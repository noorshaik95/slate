use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{
    CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerStats, CircuitState, CircuitStats,
};

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
/// let config = CircuitBreakerConfig::default();
/// let breaker = CircuitBreaker::with_name("my-service".to_string(), config);
///
/// match breaker.call(|| async { backend_service_call().await }).await {
///     Ok(result) => { /* success */ },
///     Err(CircuitBreakerError::Open) => { /* circuit is open */ },
///     Err(CircuitBreakerError::OperationFailed(e)) => { /* operation failed */ },
/// }
/// ```
#[derive(Clone)]
pub struct CircuitBreaker {
    name: Option<String>,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitStats>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            name: None,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitStats::default())),
        }
    }

    /// Create a new circuit breaker with a name for logging
    pub fn with_name(name: String, config: CircuitBreakerConfig) -> Self {
        info!(
            name = %name,
            failure_threshold = config.failure_threshold,
            success_threshold = config.success_threshold,
            timeout_seconds = config.timeout_seconds,
            "Creating circuit breaker"
        );

        Self {
            name: Some(name),
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
                    self.log_transition("OPEN", "HALF_OPEN");
                    *state = CircuitState::HalfOpen;

                    // Reset consecutive counters for half-open testing
                    let mut stats = self.stats.write().await;
                    stats.consecutive_successes = 0;
                    stats.consecutive_failures = 0;

                    true
                } else {
                    debug!(
                        name = ?self.name,
                        "Circuit breaker is OPEN - rejecting request"
                    );
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
                    self.log_transition("HALF_OPEN", "CLOSED");
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
                        name = ?self.name,
                        consecutive_failures = stats.consecutive_failures,
                        threshold = self.config.failure_threshold,
                        timeout_seconds = self.config.timeout_seconds,
                        "Circuit breaker transitioning from CLOSED to OPEN"
                    );
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                }
            }
            CircuitState::HalfOpen => {
                warn!(
                    name = ?self.name,
                    timeout_seconds = self.config.timeout_seconds,
                    "Circuit breaker transitioning from HALF_OPEN back to OPEN"
                );
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
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await.clone();
        let stats = self.stats.read().await;

        CircuitBreakerStats {
            name: self.name.clone(),
            state,
            consecutive_failures: stats.consecutive_failures,
            consecutive_successes: stats.consecutive_successes,
            total_failures: stats.total_failures,
            total_successes: stats.total_successes,
        }
    }

    /// Manually reset the circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut stats = self.stats.write().await;

        info!(name = ?self.name, "Manually resetting circuit breaker to CLOSED state");
        *state = CircuitState::Closed;
        stats.consecutive_failures = 0;
        stats.consecutive_successes = 0;
    }

    /// Helper to log state transitions
    fn log_transition(&self, from: &str, to: &str) {
        info!(
            name = ?self.name,
            from = from,
            to = to,
            "Circuit breaker state transition"
        );
    }
}
