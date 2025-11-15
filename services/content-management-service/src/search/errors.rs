use thiserror::Error;

/// Errors that can occur during search operations
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("ElasticSearch connection error: {0}")]
    ConnectionError(String),

    #[error("ElasticSearch index error: {0}")]
    IndexError(String),

    #[error("ElasticSearch query error: {0}")]
    QueryError(String),

    #[error("Search query too short: minimum 2 characters required")]
    QueryTooShort,

    #[error("Invalid search parameters: {0}")]
    InvalidParameters(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Search service unavailable (circuit breaker open)")]
    ServiceUnavailable,
}

impl From<elasticsearch::Error> for SearchError {
    fn from(err: elasticsearch::Error) -> Self {
        SearchError::InternalError(err.to_string())
    }
}

impl From<serde_json::Error> for SearchError {
    fn from(err: serde_json::Error) -> Self {
        SearchError::SerializationError(err.to_string())
    }
}
