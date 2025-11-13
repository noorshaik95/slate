use std::time::Duration;

/// Maximum number of retry attempts for gRPC calls
pub const MAX_RETRIES: u32 = 3;

/// Initial backoff duration in milliseconds for retry logic
pub const INITIAL_BACKOFF_MS: u64 = 100;

/// Backoff multiplier for exponential backoff (base for power calculation)
pub const BACKOFF_MULTIPLIER: u64 = 2;

/// Default connection timeout in seconds
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 10;

/// TCP keepalive interval in seconds
pub const TCP_KEEPALIVE_SECS: u64 = 60;

/// HTTP/2 keepalive interval in seconds
pub const HTTP2_KEEPALIVE_INTERVAL_SECS: u64 = 30;

/// Keepalive timeout in seconds
pub const KEEPALIVE_TIMEOUT_SECS: u64 = 20;

/// Default connection timeout duration
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(DEFAULT_CONNECT_TIMEOUT_SECS);

/// TCP keepalive duration
pub const TCP_KEEPALIVE: Duration = Duration::from_secs(TCP_KEEPALIVE_SECS);

/// HTTP/2 keepalive interval duration
pub const HTTP2_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(HTTP2_KEEPALIVE_INTERVAL_SECS);

/// Keepalive timeout duration
pub const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(KEEPALIVE_TIMEOUT_SECS);

/// Error message for generic gRPC calls not implemented
pub const ERR_GENERIC_CALLS_NOT_IMPLEMENTED: &str = 
    "Generic gRPC calls not yet implemented. Use get_channel() and service-specific clients.";

/// Error message for max retries exceeded
pub const ERR_MAX_RETRIES_EXCEEDED: &str = "Max retries exceeded";

/// Retryable gRPC status codes (as string patterns)
pub const RETRYABLE_STATUS_UNAVAILABLE: &str = "Unavailable";
pub const RETRYABLE_STATUS_DEADLINE_EXCEEDED: &str = "DeadlineExceeded";
pub const RETRYABLE_STATUS_RESOURCE_EXHAUSTED: &str = "ResourceExhausted";
