//! Circuit breaker module for fault tolerance
//!
//! Provides a circuit breaker implementation to protect against cascading failures
//! when calling external services.

mod breaker;
mod types;

#[cfg(test)]
mod tests;

pub use breaker::CircuitBreaker;
pub use types::{CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerStats, CircuitState};
