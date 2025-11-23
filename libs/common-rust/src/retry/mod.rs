//! Retry module for handling transient failures
//!
//! Provides retry logic with exponential backoff and preset configurations
//! for different operation types.

mod config;
mod logic;

pub use config::{OperationType, RetryConfig};
pub use logic::{retry_operation, retry_with_backoff};
