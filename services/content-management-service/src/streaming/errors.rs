use thiserror::Error;

/// Errors that can occur in the streaming service
#[derive(Debug, Error)]
pub enum StreamingError {
    #[error("Video not found: {0}")]
    VideoNotFound(String),

    #[error("Video not published")]
    VideoNotPublished,

    #[error("Resource is not a video")]
    NotAVideo,

    #[error("Video not yet transcoded")]
    NotTranscoded,

    #[error("Invalid playback position: {0}")]
    InvalidPosition(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] anyhow::Error),
}
