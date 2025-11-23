#[cfg(test)]
mod tests {
    use super::super::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState};
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Successful calls should keep circuit closed
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Trigger failures to open circuit
        for _ in 0..3 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        // Should be in open state
        let state = cb.get_state().await;
        assert!(matches!(state, CircuitState::Open { .. }));

        // Next call should be rejected
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Next call should transition to half-open
        let _ = cb.call(async { Ok::<_, String>("success") }).await;

        let state = cb.get_state().await;
        // Should be either half-open or closed (if success threshold met)
        assert!(matches!(
            state,
            CircuitState::HalfOpen | CircuitState::Closed
        ));
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_successes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        // Wait for timeout to transition to half-open
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Successful calls should close the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Ok::<_, String>("success") }).await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reopens_on_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Fail in half-open state
        let _ = cb.call(async { Err::<String, _>("error") }).await;

        // Should be back to open
        let state = cb.get_state().await;
        assert!(matches!(state, CircuitState::Open { .. }));
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::with_name("test".to_string(), config);

        // Record some failures
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        let stats = cb.get_stats().await;
        assert_eq!(stats.name, Some("test".to_string()));
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.consecutive_failures, 2);
        assert_eq!(stats.total_failures, 2);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("error") }).await;
        }

        assert!(matches!(cb.get_state().await, CircuitState::Open { .. }));

        // Reset
        cb.reset().await;

        assert_eq!(cb.get_state().await, CircuitState::Closed);

        let stats = cb.get_stats().await;
        assert_eq!(stats.consecutive_failures, 0);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 2,
            timeout_seconds: 1,
        };

        let cb = CircuitBreaker::new(config);

        // Spawn multiple concurrent tasks
        let mut handles = vec![];
        for i in 0..10 {
            let cb_clone = cb.clone();
            let handle = tokio::spawn(async move {
                if i % 2 == 0 {
                    cb_clone
                        .call(async { Ok::<String, String>("success".to_string()) })
                        .await
                } else {
                    cb_clone
                        .call(async { Err::<String, String>("error".to_string()) })
                        .await
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let _ = handle.await;
        }

        // Circuit should still be operational
        let stats = cb.get_stats().await;
        assert_eq!(stats.total_successes, 5);
        assert_eq!(stats.total_failures, 5);
    }
}
