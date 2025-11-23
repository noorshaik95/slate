//! # Common Rust Library
//!
//! Shared utilities for Rust microservices including:
//! - Circuit breaker for fault tolerance
//! - Rate limiting for request throttling
//! - Health checks for service monitoring
//! - Retry logic with exponential backoff
//! - Error response handling
//! - Observability utilities (tracing, logging)

pub mod circuit_breaker;
pub mod error;
pub mod health;
pub mod observability;
pub mod rate_limit;
pub mod retry;

// Re-export commonly used types
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState,
};
pub use error::{ErrorDetail, ErrorResponse};
pub use health::{ComponentHealth, HealthChecker, HealthStatus, ServiceHealth};
pub use rate_limit::{IpRateLimiter, RateLimitConfig, RateLimitError, RateLimiter};
pub use retry::{retry_operation, retry_with_backoff, OperationType, RetryConfig};
