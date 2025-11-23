use serde::{Deserialize, Serialize};
use thiserror::Error;
use tonic::{Code, Status};
use tracing::error;

/// Unified error type for the Content Management Service
#[derive(Debug, Error)]
pub enum ServiceError {
    // Validation errors (400 Bad Request)
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File size exceeds limit: {0}")]
    FileSizeExceeded(String),

    #[error("Invalid hierarchy: {0}")]
    InvalidHierarchy(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    // Authorization errors (403 Forbidden)
    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Copyright restriction: {0}")]
    CopyrightRestriction(String),

    // Not found errors (404 Not Found)
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Lesson not found: {0}")]
    LessonNotFound(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Upload session not found: {0}")]
    SessionNotFound(String),

    // Conflict errors (409 Conflict)
    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Module has children and cannot be deleted")]
    ModuleHasChildren,

    #[error("Lesson has children and cannot be deleted")]
    LessonHasChildren,

    #[error("Duplicate display order: {0}")]
    DuplicateDisplayOrder(String),

    // Service errors (500 Internal Server Error)
    #[error("Database error: {0}")]
    Database(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Search service error: {0}")]
    SearchService(String),

    #[error("Transcoding error: {0}")]
    Transcoding(String),

    #[error("Analytics service error: {0}")]
    AnalyticsService(String),

    #[error("Internal error: {0}")]
    Internal(String),

    // Service unavailable (503 Service Unavailable)
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Database unavailable")]
    DatabaseUnavailable,

    #[error("Storage unavailable")]
    StorageUnavailable,

    #[error("Search service unavailable")]
    SearchServiceUnavailable,
}

/// Error response structure for gRPC and HTTP responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code for client-side handling
    pub error_code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Trace ID for debugging
    pub trace_id: String,
}

impl ErrorResponse {
    pub fn new(error_code: String, message: String, trace_id: String) -> Self {
        Self {
            error_code,
            message,
            details: None,
            trace_id,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl ServiceError {
    /// Convert ServiceError to gRPC Status
    pub fn to_status(&self, trace_id: &str) -> Status {
        let (code, error_code) = self.get_grpc_code_and_error_code();

        let error_response = ErrorResponse::new(error_code, self.to_string(), trace_id.to_string());

        let details = serde_json::to_string(&error_response).unwrap_or_else(|_| self.to_string());

        error!(
            error = %self,
            trace_id = %trace_id,
            grpc_code = ?code,
            "Service error occurred"
        );

        Status::new(code, details)
    }

    /// Get gRPC status code and error code string
    fn get_grpc_code_and_error_code(&self) -> (Code, String) {
        match self {
            // Validation errors -> INVALID_ARGUMENT
            ServiceError::Validation(_) => (Code::InvalidArgument, "VALIDATION_ERROR".to_string()),
            ServiceError::InvalidFileType(_) => {
                (Code::InvalidArgument, "INVALID_FILE_TYPE".to_string())
            }
            ServiceError::FileSizeExceeded(_) => {
                (Code::InvalidArgument, "FILE_SIZE_EXCEEDED".to_string())
            }
            ServiceError::InvalidHierarchy(_) => {
                (Code::InvalidArgument, "INVALID_HIERARCHY".to_string())
            }
            ServiceError::InvalidInput(_) => (Code::InvalidArgument, "INVALID_INPUT".to_string()),

            // Authorization errors -> PERMISSION_DENIED
            ServiceError::Authorization(_) => {
                (Code::PermissionDenied, "AUTHORIZATION_ERROR".to_string())
            }
            ServiceError::AccessDenied(_) => (Code::PermissionDenied, "ACCESS_DENIED".to_string()),
            ServiceError::CopyrightRestriction(_) => {
                (Code::PermissionDenied, "COPYRIGHT_RESTRICTION".to_string())
            }

            // Not found errors -> NOT_FOUND
            ServiceError::NotFound(_) => (Code::NotFound, "NOT_FOUND".to_string()),
            ServiceError::ModuleNotFound(_) => (Code::NotFound, "MODULE_NOT_FOUND".to_string()),
            ServiceError::LessonNotFound(_) => (Code::NotFound, "LESSON_NOT_FOUND".to_string()),
            ServiceError::ResourceNotFound(_) => (Code::NotFound, "RESOURCE_NOT_FOUND".to_string()),
            ServiceError::SessionNotFound(_) => (Code::NotFound, "SESSION_NOT_FOUND".to_string()),

            // Conflict errors -> ALREADY_EXISTS or FAILED_PRECONDITION
            ServiceError::Conflict(_) => (Code::FailedPrecondition, "CONFLICT".to_string()),
            ServiceError::ModuleHasChildren => {
                (Code::FailedPrecondition, "MODULE_HAS_CHILDREN".to_string())
            }
            ServiceError::LessonHasChildren => {
                (Code::FailedPrecondition, "LESSON_HAS_CHILDREN".to_string())
            }
            ServiceError::DuplicateDisplayOrder(_) => {
                (Code::AlreadyExists, "DUPLICATE_DISPLAY_ORDER".to_string())
            }

            // Service errors -> INTERNAL
            ServiceError::Database(_) => (Code::Internal, "DATABASE_ERROR".to_string()),
            ServiceError::Storage(_) => (Code::Internal, "STORAGE_ERROR".to_string()),
            ServiceError::SearchService(_) => (Code::Internal, "SEARCH_SERVICE_ERROR".to_string()),
            ServiceError::Transcoding(_) => (Code::Internal, "TRANSCODING_ERROR".to_string()),
            ServiceError::AnalyticsService(_) => {
                (Code::Internal, "ANALYTICS_SERVICE_ERROR".to_string())
            }
            ServiceError::Internal(_) => (Code::Internal, "INTERNAL_ERROR".to_string()),

            // Service unavailable -> UNAVAILABLE
            ServiceError::ServiceUnavailable(_) => {
                (Code::Unavailable, "SERVICE_UNAVAILABLE".to_string())
            }
            ServiceError::DatabaseUnavailable => {
                (Code::Unavailable, "DATABASE_UNAVAILABLE".to_string())
            }
            ServiceError::StorageUnavailable => {
                (Code::Unavailable, "STORAGE_UNAVAILABLE".to_string())
            }
            ServiceError::SearchServiceUnavailable => {
                (Code::Unavailable, "SEARCH_SERVICE_UNAVAILABLE".to_string())
            }
        }
    }
}

// Conversions from domain-specific errors to ServiceError

impl From<crate::content::errors::ContentError> for ServiceError {
    fn from(err: crate::content::errors::ContentError) -> Self {
        match err {
            crate::content::errors::ContentError::Validation(msg) => ServiceError::Validation(msg),
            crate::content::errors::ContentError::NotFound(msg) => ServiceError::NotFound(msg),
            crate::content::errors::ContentError::Conflict(msg) => ServiceError::Conflict(msg),
            crate::content::errors::ContentError::Authorization(msg) => {
                ServiceError::Authorization(msg)
            }
            crate::content::errors::ContentError::Database(e) => {
                ServiceError::Database(e.to_string())
            }
            crate::content::errors::ContentError::Internal(msg) => ServiceError::Internal(msg),
        }
    }
}

impl From<crate::upload::errors::UploadError> for ServiceError {
    fn from(err: crate::upload::errors::UploadError) -> Self {
        match err {
            crate::upload::errors::UploadError::InvalidFileType(msg) => {
                ServiceError::InvalidFileType(msg)
            }
            crate::upload::errors::UploadError::FileSizeExceeded(_, _) => {
                ServiceError::FileSizeExceeded(err.to_string())
            }
            crate::upload::errors::UploadError::InvalidFilename(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::upload::errors::UploadError::SessionNotFound(msg) => {
                ServiceError::SessionNotFound(msg)
            }
            crate::upload::errors::UploadError::SessionExpired(msg) => ServiceError::Conflict(msg),
            crate::upload::errors::UploadError::SessionNotResumable(msg) => {
                ServiceError::Conflict(msg)
            }
            crate::upload::errors::UploadError::InvalidChunkIndex(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::upload::errors::UploadError::ChunkAlreadyUploaded(_) => {
                ServiceError::Conflict(err.to_string())
            }
            crate::upload::errors::UploadError::StorageError(msg) => ServiceError::Storage(msg),
            crate::upload::errors::UploadError::DatabaseError(msg) => ServiceError::Database(msg),
            crate::upload::errors::UploadError::MalwareDetected(msg) => {
                ServiceError::Validation(msg)
            }
            crate::upload::errors::UploadError::FileHeaderMismatch(msg) => {
                ServiceError::Validation(msg)
            }
        }
    }
}

impl From<crate::storage::StorageError> for ServiceError {
    fn from(err: crate::storage::StorageError) -> Self {
        match err {
            crate::storage::StorageError::ObjectNotFound(msg) => ServiceError::NotFound(msg),
            crate::storage::StorageError::InvalidStorageKey(msg) => ServiceError::InvalidInput(msg),
            _ => ServiceError::Storage(err.to_string()),
        }
    }
}

impl From<crate::streaming::errors::StreamingError> for ServiceError {
    fn from(err: crate::streaming::errors::StreamingError) -> Self {
        match err {
            crate::streaming::errors::StreamingError::VideoNotFound(msg) => {
                ServiceError::ResourceNotFound(msg)
            }
            crate::streaming::errors::StreamingError::VideoNotPublished => {
                ServiceError::AccessDenied("Video not published".to_string())
            }
            crate::streaming::errors::StreamingError::NotAVideo => {
                ServiceError::InvalidInput("Resource is not a video".to_string())
            }
            crate::streaming::errors::StreamingError::NotTranscoded => {
                ServiceError::Internal("Video not yet transcoded".to_string())
            }
            crate::streaming::errors::StreamingError::InvalidPosition(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::streaming::errors::StreamingError::AccessDenied(msg) => {
                ServiceError::AccessDenied(msg)
            }
            crate::streaming::errors::StreamingError::DatabaseError(e) => {
                ServiceError::Database(e.to_string())
            }
        }
    }
}

impl From<crate::progress::errors::ProgressError> for ServiceError {
    fn from(err: crate::progress::errors::ProgressError) -> Self {
        match err {
            crate::progress::errors::ProgressError::ResourceNotFound(msg) => {
                ServiceError::ResourceNotFound(msg)
            }
            crate::progress::errors::ProgressError::ResourceNotPublished => {
                ServiceError::AccessDenied("Resource not published".to_string())
            }
            crate::progress::errors::ProgressError::InvalidData(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::progress::errors::ProgressError::DatabaseError(e) => {
                ServiceError::Database(e.to_string())
            }
            crate::progress::errors::ProgressError::RepositoryError(e) => {
                ServiceError::Database(e.to_string())
            }
        }
    }
}

impl From<crate::search::errors::SearchError> for ServiceError {
    fn from(err: crate::search::errors::SearchError) -> Self {
        match err {
            crate::search::errors::SearchError::ConnectionError(_) => {
                ServiceError::SearchServiceUnavailable
            }
            crate::search::errors::SearchError::QueryError(msg) => ServiceError::SearchService(msg),
            crate::search::errors::SearchError::IndexError(msg) => ServiceError::SearchService(msg),
            crate::search::errors::SearchError::SerializationError(msg) => {
                ServiceError::Internal(msg)
            }
            crate::search::errors::SearchError::QueryTooShort => {
                ServiceError::InvalidInput("Query too short".to_string())
            }
            crate::search::errors::SearchError::InvalidParameters(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::search::errors::SearchError::ResourceNotFound(msg) => {
                ServiceError::ResourceNotFound(msg)
            }
            crate::search::errors::SearchError::InternalError(msg) => ServiceError::Internal(msg),
            crate::search::errors::SearchError::ServiceUnavailable => {
                ServiceError::SearchServiceUnavailable
            }
        }
    }
}

impl From<crate::download::errors::DownloadError> for ServiceError {
    fn from(err: crate::download::errors::DownloadError) -> Self {
        match err {
            crate::download::errors::DownloadError::ResourceNotFound(msg) => {
                ServiceError::ResourceNotFound(msg)
            }
            crate::download::errors::DownloadError::ResourceNotPublished => {
                ServiceError::AccessDenied("Resource not published".to_string())
            }
            crate::download::errors::DownloadError::DownloadNotAllowed(msg) => {
                ServiceError::CopyrightRestriction(msg)
            }
            crate::download::errors::DownloadError::CopyrightRestriction(msg) => {
                ServiceError::CopyrightRestriction(msg)
            }
            crate::download::errors::DownloadError::InvalidResourceType(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::download::errors::DownloadError::StorageError(msg) => ServiceError::Storage(msg),
            crate::download::errors::DownloadError::DatabaseError(msg) => {
                ServiceError::Database(msg)
            }
            crate::download::errors::DownloadError::AnalyticsError(msg) => {
                ServiceError::AnalyticsService(msg)
            }
            crate::download::errors::DownloadError::InternalError(msg) => {
                ServiceError::Internal(msg)
            }
        }
    }
}

impl From<crate::transcoding::errors::TranscodingError> for ServiceError {
    fn from(err: crate::transcoding::errors::TranscodingError) -> Self {
        match err {
            crate::transcoding::errors::TranscodingError::Redis(_) => {
                ServiceError::Internal(err.to_string())
            }
            crate::transcoding::errors::TranscodingError::Serialization(_) => {
                ServiceError::Internal(err.to_string())
            }
            crate::transcoding::errors::TranscodingError::JobNotFound(msg) => {
                ServiceError::NotFound(msg)
            }
            crate::transcoding::errors::TranscodingError::InvalidJobData(msg) => {
                ServiceError::InvalidInput(msg)
            }
            crate::transcoding::errors::TranscodingError::FFmpegFailed(msg) => {
                ServiceError::Transcoding(msg)
            }
            crate::transcoding::errors::TranscodingError::DownloadFailed(msg) => {
                ServiceError::Storage(msg)
            }
            crate::transcoding::errors::TranscodingError::UploadFailed(msg) => {
                ServiceError::Storage(msg)
            }
            crate::transcoding::errors::TranscodingError::Database(_) => {
                ServiceError::Database(err.to_string())
            }
            crate::transcoding::errors::TranscodingError::Io(_) => {
                ServiceError::Internal(err.to_string())
            }
        }
    }
}

impl From<crate::analytics::errors::AnalyticsError> for ServiceError {
    fn from(err: crate::analytics::errors::AnalyticsError) -> Self {
        match err {
            crate::analytics::errors::AnalyticsError::SerializationError(_) => {
                ServiceError::Internal(err.to_string())
            }
            crate::analytics::errors::AnalyticsError::TransmissionError(msg) => {
                ServiceError::AnalyticsService(msg)
            }
            crate::analytics::errors::AnalyticsError::ServiceUnavailable(msg) => {
                ServiceError::AnalyticsService(msg)
            }
            crate::analytics::errors::AnalyticsError::QueueError(msg) => {
                ServiceError::Internal(msg)
            }
            crate::analytics::errors::AnalyticsError::EmptyBatch => {
                ServiceError::Internal("Empty batch".to_string())
            }
            crate::analytics::errors::AnalyticsError::EventExpired => {
                ServiceError::Internal("Event expired".to_string())
            }
        }
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ServiceError::NotFound("Record not found".to_string()),
            sqlx::Error::PoolTimedOut => ServiceError::DatabaseUnavailable,
            _ => ServiceError::Database(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err.to_string())
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

// Tests moved to tests/error_mapping_test.rs
