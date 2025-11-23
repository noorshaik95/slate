use std::future::Future;
use tracing::{debug, warn};

use super::config::{OperationType, RetryConfig};

/// Retry an async operation with exponential backoff
///
/// # Example
///
/// ```rust,ignore
/// let config = RetryConfig::database();
/// let result = retry_with_backoff(config, || async {
///     database_query().await
/// }).await;
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(config: RetryConfig, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!(attempt = attempt + 1, "Operation succeeded after retry");
                }
                return Ok(result);
            }
            Err(error) => {
                attempt += 1;

                if attempt >= config.max_attempts {
                    warn!(
                        attempt,
                        max_attempts = config.max_attempts,
                        error = %error,
                        "Operation failed after max retry attempts"
                    );
                    return Err(error);
                }

                let backoff = config.backoff_duration(attempt - 1);
                warn!(
                    attempt,
                    max_attempts = config.max_attempts,
                    backoff_ms = backoff.as_millis(),
                    error = %error,
                    "Operation failed, retrying after backoff"
                );

                tokio::time::sleep(backoff).await;
            }
        }
    }
}

/// Retry an operation with a preset configuration
///
/// # Example
///
/// ```rust,ignore
/// let result = retry_operation(
///     OperationType::Database,
///     || async { database_query().await }
/// ).await;
/// ```
pub async fn retry_operation<F, Fut, T, E>(op_type: OperationType, operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let config = RetryConfig::for_operation(op_type);
    retry_with_backoff(config, operation).await
}
