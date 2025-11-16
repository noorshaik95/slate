use thiserror::Error;

/// Content management errors
#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for ContentError {
    fn from(err: anyhow::Error) -> Self {
        ContentError::Internal(err.to_string())
    }
}

pub type ContentResult<T> = Result<T, ContentError>;
