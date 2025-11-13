use std::collections::VecDeque;
use std::time::Instant;

/// Tracks the rate limit state for a single client
#[derive(Debug)]
pub struct ClientRateState {
    /// Queue of request timestamps within the current window
    pub requests: VecDeque<Instant>,
    /// Last time cleanup was performed for this client
    pub last_cleanup: Instant,
}

/// Error types for rate limiting
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0} requests in {1} seconds")]
    Exceeded(u32, u64),
    
    #[error("Rate limiter is disabled")]
    Disabled,
}

impl ClientRateState {
    /// Create a new client rate state
    pub fn new() -> Self {
        Self {
            requests: VecDeque::new(),
            last_cleanup: Instant::now(),
        }
    }
}

impl Default for ClientRateState {
    fn default() -> Self {
        Self::new()
    }
}
