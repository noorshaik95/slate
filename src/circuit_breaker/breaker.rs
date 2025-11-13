use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{CircuitBreakerConfig, CircuitBreakerError, CircuitState, CircuitStats};

/// Circuit breaker implementation for protecting backend service calls
///
/// The circuit breaker has three states:
/// - Closed: Normal operation, requests pass through
/// - Open: Too many failures detected, reject requests immediately
/// - Half-Open: Testing if service has recovered
///
/// # Example
///
/// ```rust,ignore
/// let breaker = CircuitBreaker::new(config);
///
/// match breaker.call(|| async { backend_service_call().await }).await {
///     Ok(result) => { /* success */ },
///     Err(CircuitBreakerError::Open) => { /* circuit is open */ },
///     Err(CircuitBreakerError::OperationFailed(e)) => { /* operation failed */ },
/// }
/// ```
#[derive(Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitStats>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitStats::default())),
        }
    }

    /// Execute an async operation through the circuit breaker
    ///
    /// Returns:
    /// - Ok(T) if operation succeeds
    /// - Err(CircuitBreakerError::Open) if circuit is open
    /// - Err(CircuitBreakerError::OperationFailed) if operation fails
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit is open
        if !self.can_execute().await {
            return Err(CircuitBreakerError::Open);
        }

        // Execute the operation
        match operation.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(e.to_string()))
            }
        }
    }

    /// Check if a request can be executed (circuit is not open)
    async fn can_execute(&self) -> bool {
        let mut state = self.state.write().await;

        match *state {
            CircuitState::Closed => true,
            CircuitState::HalfOpen => true,
            CircuitState::Open { opened_at } => {
                let timeout = Duration::from_secs(self.config.timeout_seconds);

                if opened_at.elapsed() >= timeout {
                    // Transition to half-open state
                    info!("Circuit breaker transitioning from OPEN to HALF_OPEN");
                    *state = CircuitState::HalfOpen;

                    // Reset consecutive counters for half-open testing
                    let mut stats = self.stats.write().await;
                    stats.consecutive_successes = 0;
                    stats.consecutive_failures = 0;

                    true
                } else {
                    debug!("Circuit breaker is OPEN - rejecting request");
                    false
                }
            }
        }
    }

    /// Record a successful operation
    async fn record_success(&self) {
        let mut stats = self.stats.write().await;
        let mut state = self.state.write().await;

        stats.consecutive_successes += 1;
        stats.consecutive_failures = 0;
        stats.total_successes += 1;

        match *state {
            CircuitState::HalfOpen => {
                if stats.consecutive_successes >= self.config.success_threshold {
                    info!(
                        consecutive_successes = stats.consecutive_successes,
                        threshold = self.config.success_threshold,
                        "Circuit breaker transitioning from HALF_OPEN to CLOSED"
                    );
                    *state = CircuitState::Closed;
                    stats.consecutive_successes = 0;
                }
            }
            _ => {}
        }
    }

    /// Record a failed operation
    async fn record_failure(&self) {
        let mut stats = self.stats.write().await;
        let mut state = self.state.write().await;

        stats.consecutive_failures += 1;
        stats.consecutive_successes = 0;
        stats.total_failures += 1;

        match *state {
            CircuitState::Closed => {
                if stats.consecutive_failures >= self.config.failure_threshold {
                    warn!(
                        consecutive_failures = stats.consecutive_failures,
                        threshold = self.config.failure_threshold,
                        "Circuit breaker transitioning from CLOSED to OPEN"
                    );
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker transitioning from HALF_OPEN back to OPEN");
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
            }
            CircuitState::Open { .. } => {
                // Already open, no state change needed
            }
        }
    }

    /// Get current circuit state (for monitoring/debugging)
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Get current statistics (for monitoring/debugging)
    pub async fn get_stats(&self) -> (u32, u32, u64, u64) {
        let stats = self.stats.read().await;
        (
            stats.consecutive_failures,
            stats.consecutive_successes,
            stats.total_failures,
            stats.total_successes,
        )
    }

    /// Manually reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        info!("Manually resetting circuit breaker to CLOSED state");
        *state = CircuitState::Closed;
        stats.consecutive_failures = 0;
        stats.consecutive_successes = 0;
    }
}
