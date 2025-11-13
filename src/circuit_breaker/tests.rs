use super::*;
use std::time::Duration;
use tokio::time::sleep;

async fn always_succeed() -> Result<String, String> {
    Ok("success".to_string())
}

async fn always_fail() -> Result<String, String> {
    Err("failure".to_string())
}

#[tokio::test]
async fn test_circuit_breaker_starts_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Closed));
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // First 2 failures should keep circuit closed
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
        let state = breaker.get_state().await;
        assert!(matches!(state, CircuitState::Closed));
    }

    // Third failure should open the circuit
    let _ = breaker.call(always_fail()).await;
    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Open { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_rejects_when_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    // Circuit should be open
    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Open { .. }));

    // Next call should be rejected immediately
    let result = breaker.call(always_succeed()).await;
    assert!(matches!(
        result,
        Err(CircuitBreakerError::Open)
    ));
}

#[tokio::test]
async fn test_circuit_breaker_transitions_to_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    // Wait for timeout
    sleep(Duration::from_secs(2)).await;

    // Next call should transition to half-open
    let result = breaker.call(always_succeed()).await;
    assert!(result.is_ok());

    // Should be in half-open or closed state
    let state = breaker.get_state().await;
    assert!(
        matches!(state, CircuitState::HalfOpen) || matches!(state, CircuitState::Closed)
    );
}

#[tokio::test]
async fn test_circuit_breaker_closes_after_successes_in_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    // Wait for timeout to transition to half-open
    sleep(Duration::from_secs(2)).await;

    // First success in half-open
    let _ = breaker.call(always_succeed()).await;

    // Second success should close the circuit
    let _ = breaker.call(always_succeed()).await;

    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Closed));
}

#[tokio::test]
async fn test_circuit_breaker_reopens_on_failure_in_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    // Wait for timeout
    sleep(Duration::from_secs(2)).await;

    // Try to succeed but fail - should reopen circuit
    let _ = breaker.call(always_fail()).await;

    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Open { .. }));
}

#[tokio::test]
async fn test_circuit_breaker_tracks_statistics() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // 3 successes
    for _ in 0..3 {
        let _ = breaker.call(always_succeed()).await;
    }

    // 2 failures
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    let (consecutive_failures, consecutive_successes, total_failures, total_successes) =
        breaker.get_stats().await;

    assert_eq!(consecutive_failures, 2);
    assert_eq!(consecutive_successes, 0); // Reset by failures
    assert_eq!(total_failures, 2);
    assert_eq!(total_successes, 3);
}

#[tokio::test]
async fn test_circuit_breaker_manual_reset() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_seconds: 60,
    };
    let breaker = CircuitBreaker::new(config);

    // Open the circuit
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Open { .. }));

    // Manual reset
    breaker.reset().await;

    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Closed));

    let (consecutive_failures, consecutive_successes, _, _) = breaker.get_stats().await;
    assert_eq!(consecutive_failures, 0);
    assert_eq!(consecutive_successes, 0);
}

#[tokio::test]
async fn test_circuit_breaker_success_resets_failure_count() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_seconds: 1,
    };
    let breaker = CircuitBreaker::new(config);

    // 2 failures
    for _ in 0..2 {
        let _ = breaker.call(always_fail()).await;
    }

    // 1 success - should reset consecutive failures
    let _ = breaker.call(always_succeed()).await;

    let (consecutive_failures, _, _, _) = breaker.get_stats().await;
    assert_eq!(consecutive_failures, 0);

    // Circuit should still be closed
    let state = breaker.get_state().await;
    assert!(matches!(state, CircuitState::Closed));
}
