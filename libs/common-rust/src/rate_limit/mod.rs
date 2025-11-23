//! Rate limiting module for request throttling
//!
//! Provides a generic rate limiter with sliding window algorithm and LRU cache
//! for bounded memory usage.

mod constants;
mod limiter;
mod types;

#[cfg(test)]
mod tests;

pub use constants::{CLEANUP_THRESHOLD_MULTIPLIER, DEFAULT_EXCLUDED_PATHS, MAX_TRACKED_CLIENTS};
pub use limiter::{should_exclude_path, IpRateLimiter, RateLimiter};
pub use types::{ClientRateState, RateLimitConfig, RateLimitError};
