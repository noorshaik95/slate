use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration for different operation types
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum backoff duration
    pub max_backoff: Duration,
}

impl RetryConfig {
    /// Configuration for database operations (3 retries: 100ms, 200ms, 400ms)
    pub fn database() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_millis(400),
        }
    }

    /// Configuration for S3 operations (3 retries with exponential backoff)
    pub fn storage() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(2),
        }
    }

    /// Configuration for ElasticSearch operations (2 retries)
    pub fn search() -> Self {
        Self {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(200),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(1),
        }
    }

    /// Configuration for analytics events (retry every 5 minutes for up to 24 hours)
    pub fn analytics() -> Self {
        Self {
            max_attempts: 288, // 24 hours / 5 minutes = 288 attempts
            initial_backoff: Duration::from_secs(300), // 5 minutes
            backoff_multiplier: 1.0, // No exponential backoff, constant interval
            max_backoff: Duration::from_secs(300),
        }
    }

    /// Calculate backoff duration for a given attempt number
    fn backoff_duration(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }

        let backoff_ms = self.initial_backoff.as_millis() as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        
        let backoff = Duration::from_millis(backoff_ms as u64);
        
        // Cap at max_backoff
        if backoff > self.max_backoff {
            self.max_backoff
        } else {
            backoff
        }
    }
}

/// Retry a future with exponential backoff
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation_name` - Name of the operation for logging
/// * `f` - Async function that returns a Result
///
/// # Returns
/// Result of the operation or the last error encountered
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: RetryConfig,
    operation_name: &str,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        match f().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(err) => {
                if attempt >= config.max_attempts {
                    warn!(
                        operation = operation_name,
                        attempt = attempt,
                        error = %err,
                        "Operation failed after all retry attempts"
                    );
                    return Err(err);
                }

                let backoff = config.backoff_duration(attempt);
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    backoff_ms = backoff.as_millis(),
                    error = %err,
                    "Operation failed, retrying after backoff"
                );

                sleep(backoff).await;
            }
        }
    }
}

/// Retry a database operation with standard database retry configuration
pub async fn retry_database<F, Fut, T>(
    operation_name: &str,
    f: F,
) -> Result<T, sqlx::Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    retry_with_backoff(RetryConfig::database(), operation_name, f).await
}

/// Retry a storage operation with standard storage retry configuration
pub async fn retry_storage<F, Fut, T, E>(
    operation_name: &str,
    f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry_with_backoff(RetryConfig::storage(), operation_name, f).await
}

/// Retry a search operation with standard search retry configuration
pub async fn retry_search<F, Fut, T, E>(
    operation_name: &str,
    f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    retry_with_backoff(RetryConfig::search(), operation_name, f).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_database_retry_config() {
        let config = RetryConfig::database();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff, Duration::from_millis(100));
        
        // Test backoff progression: 100ms, 200ms, 400ms
        assert_eq!(config.backoff_duration(1), Duration::from_millis(100));
        assert_eq!(config.backoff_duration(2), Duration::from_millis(200));
        assert_eq!(config.backoff_duration(3), Duration::from_millis(400));
    }

    #[test]
    fn test_storage_retry_config() {
        let config = RetryConfig::storage();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff, Duration::from_millis(100));
        
        // Test exponential backoff with max cap
        assert_eq!(config.backoff_duration(1), Duration::from_millis(100));
        assert_eq!(config.backoff_duration(2), Duration::from_millis(200));
        assert_eq!(config.backoff_duration(3), Duration::from_millis(400));
    }

    #[test]
    fn test_search_retry_config() {
        let config = RetryConfig::search();
        assert_eq!(config.max_attempts, 2);
        assert_eq!(config.initial_backoff, Duration::from_millis(200));
    }

    #[test]
    fn test_analytics_retry_config() {
        let config = RetryConfig::analytics();
        assert_eq!(config.max_attempts, 288); // 24 hours / 5 minutes
        assert_eq!(config.initial_backoff, Duration::from_secs(300)); // 5 minutes
        
        // Test constant backoff (no exponential growth)
        assert_eq!(config.backoff_duration(1), Duration::from_secs(300));
        assert_eq!(config.backoff_duration(2), Duration::from_secs(300));
        assert_eq!(config.backoff_duration(10), Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_first_attempt() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            RetryConfig::database(),
            "test_operation",
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>("success")
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            RetryConfig::database(),
            "test_operation",
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok::<_, &str>("success")
                    }
                }
            },
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            RetryConfig::database(),
            "test_operation",
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<String, _>("persistent failure")
                }
            },
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "persistent failure");
        assert_eq!(counter.load(Ordering::SeqCst), 3); // max_attempts
    }

    #[test]
    fn test_backoff_duration_caps_at_max() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_backoff: Duration::from_secs(1),
        };

        // After a few attempts, backoff should be capped at max_backoff
        assert!(config.backoff_duration(5) <= Duration::from_secs(1));
        assert!(config.backoff_duration(10) <= Duration::from_secs(1));
    }
}
