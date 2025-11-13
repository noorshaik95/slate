// Declare submodules
mod types;
mod constants;
mod limiter;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{ClientRateState, RateLimitError};

// Re-export public functions/structs from limiter
pub use limiter::RateLimiter;
