use common_rust::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_different_timeouts_for_different_services() {
    // Fast service with 2-second timeout
    let fast_config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 2,
    };
    let fast_breaker = CircuitBreaker::new(fast_config);

    // Slow service with 5-second timeout
    let slow_config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 5,
    };
    let slow_breaker = CircuitBreaker::new(slow_config);

    // Trigger failures to open both circuit breakers
    for _ in 0..3 {
        let _ = fast_breaker
            .call(async { Err::<(), _>("simulated failure") })
            .await;
        let _ = slow_breaker
            .call(async { Err::<(), _>("simulated failure") })
            .await;
    }

    // Both should be open now
    let fast_result = fast_breaker.call(async { Ok::<_, String>("test") }).await;
    let slow_result = slow_breaker.call(async { Ok::<_, String>("test") }).await;

    assert!(fast_result.is_err());
    assert!(slow_result.is_err());

    // Wait 2.5 seconds - fast breaker should transition to half-open, slow should still be open
    sleep(Duration::from_millis(2500)).await;

    let fast_result = fast_breaker.call(async { Ok::<_, String>("test") }).await;
    let slow_result = slow_breaker.call(async { Ok::<_, String>("test") }).await;

    // Fast breaker should allow request (half-open)
    assert!(fast_result.is_ok());

    // Slow breaker should still reject (still open)
    assert!(slow_result.is_err());

    // Wait another 3 seconds - now slow breaker should also transition to half-open
    sleep(Duration::from_secs(3)).await;

    let slow_result = slow_breaker.call(async { Ok::<_, String>("test") }).await;

    // Slow breaker should now allow request (half-open)
    assert!(slow_result.is_ok());
}

#[tokio::test]
async fn test_fast_service_not_affected_by_slow_service_timeout() {
    // Fast service with 1-second timeout
    let fast_config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout_seconds: 1,
    };
    let fast_breaker = CircuitBreaker::new(fast_config);

    // Slow service with 10-second timeout
    let slow_config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout_seconds: 10,
    };
    let slow_breaker = CircuitBreaker::new(slow_config);

    // Open both breakers
    for _ in 0..2 {
        let _ = fast_breaker.call(async { Err::<(), _>("failure") }).await;
        let _ = slow_breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait 1.5 seconds
    sleep(Duration::from_millis(1500)).await;

    // Fast breaker should recover
    let fast_result = fast_breaker
        .call(async { Ok::<_, String>("success") })
        .await;
    assert!(fast_result.is_ok());

    // Slow breaker should still be open
    let slow_result = slow_breaker
        .call(async { Ok::<_, String>("success") })
        .await;
    assert!(slow_result.is_err());
}

#[tokio::test]
async fn test_default_timeout_fallback() {
    // Create breaker with default config
    let default_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

    // Open the breaker
    for _ in 0..5 {
        let _ = default_breaker
            .call(async { Err::<(), _>("failure") })
            .await;
    }

    // Verify it's open
    let result = default_breaker
        .call(async { Ok::<_, String>("test") })
        .await;
    assert!(result.is_err());

    // Default timeout is 30 seconds, so after 1 second it should still be open
    sleep(Duration::from_secs(1)).await;
    let result = default_breaker
        .call(async { Ok::<_, String>("test") })
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_per_service_timeout_configuration() {
    // Simulate different service types with appropriate timeouts

    // Critical fast service - 15 second timeout
    let critical_config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout_seconds: 15,
    };

    // Standard service - 30 second timeout (default)
    let standard_config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout_seconds: 30,
    };

    // Slow batch service - 60 second timeout
    let batch_config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 60,
    };

    let critical_breaker = CircuitBreaker::new(critical_config);
    let standard_breaker = CircuitBreaker::new(standard_config);
    let batch_breaker = CircuitBreaker::new(batch_config);

    // Open all breakers
    for _ in 0..5 {
        let _ = critical_breaker
            .call(async { Err::<(), _>("failure") })
            .await;
        let _ = standard_breaker
            .call(async { Err::<(), _>("failure") })
            .await;
        let _ = batch_breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // All should be open
    assert!(critical_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_err());
    assert!(standard_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_err());
    assert!(batch_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_err());

    // After 16 seconds, only critical should transition to half-open
    sleep(Duration::from_secs(16)).await;

    assert!(critical_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_ok());
    assert!(standard_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_err());
    assert!(batch_breaker
        .call(async { Ok::<_, String>("test") })
        .await
        .is_err());
}

#[tokio::test]
async fn test_timeout_logged_in_state_transitions() {
    // This test verifies that timeout values are properly configured
    // The actual logging is tested through the circuit breaker implementation

    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout_seconds: 1,
    };

    let breaker = CircuitBreaker::new(config);

    // Open the breaker
    for _ in 0..2 {
        let _ = breaker.call(async { Err::<(), _>("failure") }).await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(1100)).await;

    // This should trigger the transition to half-open (with timeout logging)
    let result = breaker.call(async { Ok::<_, String>("success") }).await;
    assert!(result.is_ok());
}
