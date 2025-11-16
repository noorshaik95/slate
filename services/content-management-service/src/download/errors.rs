use thiserror::Error;

/// Errors that can occur during download operations
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Download not allowed: {0}")]
    DownloadNotAllowed(String),

    #[error("Copyright restriction: {0}")]
    CopyrightRestriction(String),

    #[error("Resource not published")]
    ResourceNotPublished,

    #[error("Invalid resource type: {0}")]
    InvalidResourceType(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Analytics error: {0}")]
    AnalyticsError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<anyhow::Error> for DownloadError {
    fn from(err: anyhow::Error) -> Self {
        DownloadError::InternalError(err.to_string())
    }
}

impl From<crate::storage::errors::StorageError> for DownloadError {
    fn from(err: crate::storage::errors::StorageError) -> Self {
        DownloadError::StorageError(err.to_string())
    }
}

impl From<crate::analytics::AnalyticsError> for DownloadError {
    fn from(err: crate::analytics::AnalyticsError) -> Self {
        DownloadError::AnalyticsError(err.to_string())
    }
}
