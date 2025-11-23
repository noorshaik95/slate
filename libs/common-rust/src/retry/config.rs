use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Operation type for preset retry configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Database,
    Storage,
    Search,
    ExternalApi,
    Analytics,
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,

    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,

    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl RetryConfig {
    /// Database operations: 3 retries, 100ms initial, 2s max
    pub fn database() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 2000,
            backoff_multiplier: 2.0,
        }
    }

    /// Storage operations (S3): 5 retries, 200ms initial, 5s max
    pub fn storage() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff_ms: 200,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }

    /// Search operations (Elasticsearch): 3 retries, 500ms initial, 3s max
    pub fn search() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 500,
            max_backoff_ms: 3000,
            backoff_multiplier: 2.0,
        }
    }

    /// External API calls: 3 retries, 1s initial, 10s max
    pub fn external_api() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 10000,
            backoff_multiplier: 2.0,
        }
    }

    /// Analytics service: 2 retries, 500ms initial, 2s max
    pub fn analytics() -> Self {
        Self {
            max_attempts: 2,
            initial_backoff_ms: 500,
            max_backoff_ms: 2000,
            backoff_multiplier: 2.0,
        }
    }

    /// Get preset configuration for operation type
    pub fn for_operation(op_type: OperationType) -> Self {
        match op_type {
            OperationType::Database => Self::database(),
            OperationType::Storage => Self::storage(),
            OperationType::Search => Self::search(),
            OperationType::ExternalApi => Self::external_api(),
            OperationType::Analytics => Self::analytics(),
        }
    }

    /// Calculate backoff duration for a given attempt
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let backoff_ms =
            (self.initial_backoff_ms as f64 * self.backoff_multiplier.powi(attempt as i32)) as u64;

        let capped_ms = backoff_ms.min(self.max_backoff_ms);
        Duration::from_millis(capped_ms)
    }
}
