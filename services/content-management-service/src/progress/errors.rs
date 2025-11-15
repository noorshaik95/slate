use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Resource is not published")]
    ResourceNotPublished,

    #[error("Invalid progress data: {0}")]
    InvalidData(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] sqlx::Error),
}
