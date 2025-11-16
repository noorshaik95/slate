use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranscodingError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Job not found: {0}")]
    JobNotFound(String),

    #[error("Invalid job data: {0}")]
    InvalidJobData(String),

    #[error("FFmpeg execution failed: {0}")]
    FFmpegFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TranscodingError>;
