use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnalyticsError {
    #[error("Failed to serialize event: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to send event to analytics service: {0}")]
    TransmissionError(String),

    #[error("Analytics service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Failed to queue event in Redis: {0}")]
    QueueError(String),

    #[error("Event batch is empty")]
    EmptyBatch,

    #[error("Event is too old and will be discarded")]
    EventExpired,
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
