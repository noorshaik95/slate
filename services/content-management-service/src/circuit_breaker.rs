use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of successes in half-open state before closing
    pub success_threshold: u32,
    /// Duration to wait before transitioning from open to half-open
    pub timeout: Duration,
    /// Window duration for counting failures
    pub window_duration: Duration,
}

impl CircuitBreakerConfig {
    /// Configuration for ElasticSearch circuit breaker
    pub fn elasticsearch() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            window_duration: Duration::from_secs(60),
        }
    }

    /// Configuration for Analytics Service circuit breaker
    pub fn analytics_service() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            window_duration: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
struct CircuitStats {
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    state_changed_at: Instant,
}

impl CircuitStats {
    fn new() -> Self {
        Self {
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            state_changed_at: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }

    fn record_success(&mut self) {
        self.success_count += 1;
        self.failure_count = 0;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        self.last_failure_time = Some(Instant::now());
    }
}

/// Circuit breaker for protecting external service calls
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<RwLock<CircuitStats>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        info!(
            "Creating circuit breaker '{}' with failure_threshold={}, success_threshold={}, timeout={:?}",
            name, config.failure_threshold, config.success_threshold, config.timeout
        );

        Self {
            name,
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(RwLock::new(CircuitStats::new())),
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit is open
        let current_state = self.get_state().await;
        
        match current_state {
            CircuitState::Open => {
                // Check if timeout has elapsed
                let stats = self.stats.read().await;
                let elapsed = stats.state_changed_at.elapsed();
                
                if elapsed >= self.config.timeout {
                    drop(stats);
                    // Transition to half-open
                    self.transition_to_half_open().await;
                } else {
                    debug!(
                        "Circuit breaker '{}' is open, rejecting request",
                        self.name
                    );
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                debug!(
                    "Circuit breaker '{}' is half-open, allowing test request",
                    self.name
                );
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(err))
            }
        }
    }

    /// Get the current circuit state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.clone()
    }

    /// Get circuit breaker statistics
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await.clone();
        let stats = self.stats.read().await;
        
        CircuitBreakerStats {
            name: self.name.clone(),
            state,
            failure_count: stats.failure_count,
            success_count: stats.success_count,
            last_failure_time: stats.last_failure_time,
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let mut stats = self.stats.write().await;
        stats.record_success();

        let current_state = self.state.read().await.clone();
        
        match current_state {
            CircuitState::HalfOpen => {
                if stats.success_count >= self.config.success_threshold {
                    drop(stats);
                    drop(current_state);
                    self.transition_to_closed().await;
                }
            }
            _ => {}
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let mut stats = self.stats.write().await;
        stats.record_failure();

        let current_state = self.state.read().await.clone();
        
        match current_state {
            CircuitState::Closed => {
                if stats.failure_count >= self.config.failure_threshold {
                    drop(stats);
                    drop(current_state);
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                drop(stats);
                drop(current_state);
                self.transition_to_open().await;
            }
            _ => {}
        }
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Open;
        
        let mut stats = self.stats.write().await;
        stats.state_changed_at = Instant::now();
        
        warn!(
            "Circuit breaker '{}' transitioned to OPEN after {} failures",
            self.name, stats.failure_count
        );
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        
        let mut stats = self.stats.write().await;
        stats.reset();
        stats.state_changed_at = Instant::now();
        
        info!(
            "Circuit breaker '{}' transitioned to HALF_OPEN for testing",
            self.name
        );
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        
        let mut stats = self.stats.write().await;
        stats.reset();
        stats.state_changed_at = Instant::now();
        
        info!(
            "Circuit breaker '{}' transitioned to CLOSED, service recovered",
            self.name
        );
    }

    /// Reset the circuit breaker (for testing/admin purposes)
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::Closed;
        
        let mut stats = self.stats.write().await;
        stats.reset();
        stats.state_changed_at = Instant::now();
        
        info!("Circuit breaker '{}' manually reset", self.name);
    }
}

/// Circuit breaker error
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open, request rejected
    CircuitOpen,
    /// Operation failed
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::fmt::Display + std::fmt::Debug> std::error::Error for CircuitBreakerError<E> {}

/// Circuit breaker statistics for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        
        // Successful calls should keep circuit closed
        let result = cb.call(|| async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Trigger failures to open circuit
        for _ in 0..3 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(cb.get_state().await, CircuitState::Open);
        
        // Next call should be rejected
        let result = cb.call(|| async { Ok::<_, String>("success") }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_transition() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(cb.get_state().await, CircuitState::Open);
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Next call should transition to half-open
        let _ = cb.call(|| async { Ok::<_, String>("success") }).await;
        
        let state = cb.get_state().await;
        assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_successes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        // Wait for timeout to transition to half-open
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Successful calls should close the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Ok::<_, String>("success") }).await;
        }
        
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_reopens_on_half_open_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;
        
        // Fail in half-open state
        let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Record some failures
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        let stats = cb.get_stats().await;
        assert_eq!(stats.name, "test");
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 2);
        assert!(stats.last_failure_time.is_some());
    }

    #[tokio::test]
    async fn test_circuit_breaker_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_secs(1),
            window_duration: Duration::from_secs(60),
        };
        
        let cb = CircuitBreaker::new("test".to_string(), config);
        
        // Open the circuit
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(cb.get_state().await, CircuitState::Open);
        
        // Reset
        cb.reset().await;
        
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 0);
    }
}
