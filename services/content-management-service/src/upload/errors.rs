use thiserror::Error;

#[derive(Error, Debug)]
pub enum UploadError {
    #[error("Invalid file type: {0}. Allowed types: video/*, application/pdf, application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    InvalidFileType(String),

    #[error("File size {0} bytes exceeds maximum of {1} bytes (500MB)")]
    FileSizeExceeded(i64, i64),

    #[error("Invalid filename: {0}")]
    InvalidFilename(String),

    #[error("Upload session not found: {0}")]
    SessionNotFound(String),

    #[error("Upload session expired: {0}")]
    SessionExpired(String),

    #[error("Upload session not resumable: {0}")]
    SessionNotResumable(String),

    #[error("Invalid chunk index: {0}")]
    InvalidChunkIndex(String),

    #[error("Chunk already uploaded: {0}")]
    ChunkAlreadyUploaded(i32),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Malware detected in file: {0}")]
    MalwareDetected(String),

    #[error("File header validation failed: {0}")]
    FileHeaderMismatch(String),
}

impl From<crate::storage::StorageError> for UploadError {
    fn from(err: crate::storage::StorageError) -> Self {
        UploadError::StorageError(err.to_string())
    }
}

impl From<anyhow::Error> for UploadError {
    fn from(err: anyhow::Error) -> Self {
        UploadError::DatabaseError(err.to_string())
    }
}
