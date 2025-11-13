use api_gateway::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;
use tokio::time::sleep;

/// Test Closed → Open transition
#[tokio::test]
async fn test_closed_to_open_transition() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 5,
    };
    let breaker = CircuitBreaker::new(config);

    // Initially in Closed state - successful calls should work
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());

    // Trigger failures to reach threshold
    for i in 0..3 {
        let result = breaker
            .call(async { Err::<(), _>(format!("failure {}", i)) })
            .await;
        assert!(result.is_err());
    }

    // Circuit should now be Open - requests should be rejected immediately
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), api_gateway::circuit_breaker::CircuitBreakerError::Open));
}

/// Test Open → Half-Open transition
#[tokio::test]
async fn test_open_to_half_open_transition() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1, // Short timeout for faster test
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Verify it's open
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());

    // Wait for timeout to expire
    sleep(Duration::from_millis(1100)).await;

    // Next request should be allowed (Half-Open state)
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());
}

/// Test Half-Open → Closed transition (successful recovery)
#[tokio::test]
async fn test_half_open_to_closed_transition() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(1100)).await;

    // Make successful calls to reach success threshold
    for _ in 0..2 {
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }

    // Circuit should now be Closed - multiple requests should succeed
    for _ in 0..5 {
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }
}

/// Test Half-Open → Open transition (failed recovery)
#[tokio::test]
async fn test_half_open_to_open_transition() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(1100)).await;

    // First request in Half-Open succeeds
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());

    // But next request fails - should go back to Open
    let result = breaker.call(async { Err::<(), _>("failure") }).await;
    assert!(result.is_err());

    // Subsequent requests should be rejected immediately (Open state)
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), api_gateway::circuit_breaker::CircuitBreakerError::Open));
}

/// Test multiple state transitions in sequence
#[tokio::test]
async fn test_multiple_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Cycle 1: Closed → Open
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());

    // Cycle 2: Open → Half-Open → Closed
    sleep(Duration::from_millis(1100)).await;
    for _ in 0..2 {
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }

    // Cycle 3: Closed → Open again
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());

    // Cycle 4: Open → Half-Open → Open (failed recovery)
    sleep(Duration::from_millis(1100)).await;
    let _ = breaker.call(async { Ok::<_, String>("success") }).await;
    let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());
}

/// Test partial success in Half-Open state
#[tokio::test]
async fn test_half_open_partial_success() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 3, // Need 3 successes to close
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(1100)).await;

    // Make 2 successful calls (not enough to close)
    for _ in 0..2 {
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }

    // One more success should close the circuit
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());

    // Now multiple requests should succeed (Closed state)
    for _ in 0..5 {
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }
}

/// Test concurrent requests in different states
#[tokio::test]
async fn test_concurrent_requests_during_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..3 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(1100)).await;

    // Make concurrent requests in Half-Open state
    let breaker_clone1 = breaker.clone();
    let breaker_clone2 = breaker.clone();
    let breaker_clone3 = breaker.clone();

    let handle1 = tokio::spawn(async move {
        breaker_clone1.call(async { Ok::<_, String>("success") }).await
    });

    let handle2 = tokio::spawn(async move {
        breaker_clone2.call(async { Ok::<_, String>("success") }).await
    });

    let handle3 = tokio::spawn(async move {
        breaker_clone3.call(async { Ok::<_, String>("success") }).await
    });

    let results = tokio::join!(handle1, handle2, handle3);
    
    // At least some requests should succeed
    let success_count = results.0.unwrap().is_ok() as i32
        + results.1.unwrap().is_ok() as i32
        + results.2.unwrap().is_ok() as i32;
    
    assert!(success_count >= 2);
}

/// Test edge case: exactly at failure threshold
#[tokio::test]
async fn test_exactly_at_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 5,
    };
    let breaker = CircuitBreaker::new(config);

    // Make exactly threshold - 1 failures
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Circuit should still be closed
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());

    // One more failure should open it
    let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    let _ = breaker.call(async { Err::<(), _>("failure") }).await;

    let result = breaker.call(async { Ok::<_, String>("test") }).await;
    assert!(result.is_err());
}
