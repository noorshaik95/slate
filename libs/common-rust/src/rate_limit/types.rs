use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Instant;

/// Rate limiter configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    /// Whether rate limiting is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum number of requests allowed per window
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,

    /// Time window in seconds for counting requests
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_requests_per_minute() -> u32 {
    60
}

fn default_window_seconds() -> u64 {
    60
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            requests_per_minute: default_requests_per_minute(),
            window_seconds: default_window_seconds(),
        }
    }
}

/// Rate limit error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0} requests in {1} seconds")]
    Exceeded(u32, u64),

    #[error("Rate limiter is disabled")]
    Disabled,
}

/// Client rate state for tracking request timestamps
#[derive(Debug)]
pub struct ClientRateState {
    pub requests: VecDeque<Instant>,
    pub last_cleanup: Instant,
}
