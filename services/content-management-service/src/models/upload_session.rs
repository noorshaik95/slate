use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// UploadStatus represents the current state of an upload session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum UploadStatus {
    #[sqlx(rename = "in_progress")]
    InProgress,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "cancelled")]
    Cancelled,
    #[sqlx(rename = "expired")]
    Expired,
}

impl std::fmt::Display for UploadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadStatus::InProgress => write!(f, "in_progress"),
            UploadStatus::Completed => write!(f, "completed"),
            UploadStatus::Failed => write!(f, "failed"),
            UploadStatus::Cancelled => write!(f, "cancelled"),
            UploadStatus::Expired => write!(f, "expired"),
        }
    }
}

impl std::str::FromStr for UploadStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "in_progress" => Ok(UploadStatus::InProgress),
            "completed" => Ok(UploadStatus::Completed),
            "failed" => Ok(UploadStatus::Failed),
            "cancelled" => Ok(UploadStatus::Cancelled),
            "expired" => Ok(UploadStatus::Expired),
            _ => Err(format!("Invalid upload status: {}", s)),
        }
    }
}

/// UploadSession manages chunked file uploads
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub lesson_id: Option<Uuid>,
    pub filename: String,
    pub content_type: String,
    pub total_size: i64,
    pub chunk_size: i32,
    pub total_chunks: i32,
    pub uploaded_chunks: i32,
    pub storage_key: String,
    pub status: UploadStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl UploadSession {
    /// Default chunk size in bytes (5MB)
    pub const DEFAULT_CHUNK_SIZE: i32 = 5 * 1024 * 1024;

    /// Session expiration duration in hours
    pub const EXPIRATION_HOURS: i64 = 24;

    /// Maximum file size in bytes (500MB)
    pub const MAX_FILE_SIZE: i64 = 500 * 1024 * 1024;

    /// Calculates the number of chunks needed for a file
    pub fn calculate_total_chunks(file_size: i64, chunk_size: i32) -> i32 {
        ((file_size as f64) / (chunk_size as f64)).ceil() as i32
    }

    /// Calculates upload progress percentage
    pub fn progress_percentage(&self) -> f64 {
        if self.total_chunks == 0 {
            return 0.0;
        }
        (self.uploaded_chunks as f64 / self.total_chunks as f64) * 100.0
    }

    /// Checks if the session has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Checks if the session is resumable
    pub fn is_resumable(&self) -> bool {
        self.status == UploadStatus::InProgress && !self.is_expired()
    }

    /// Checks if all chunks have been uploaded
    pub fn is_complete(&self) -> bool {
        self.uploaded_chunks >= self.total_chunks
    }

    /// Validates chunk index is within valid range
    pub fn validate_chunk_index(&self, chunk_index: i32) -> Result<(), String> {
        if chunk_index < 0 {
            Err("Chunk index must be non-negative".to_string())
        } else if chunk_index >= self.total_chunks {
            Err(format!(
                "Chunk index {} exceeds total chunks {}",
                chunk_index, self.total_chunks
            ))
        } else {
            Ok(())
        }
    }

    /// Validates file size
    pub fn validate_file_size(size: i64) -> Result<(), String> {
        if size <= 0 {
            Err("File size must be positive".to_string())
        } else if size > Self::MAX_FILE_SIZE {
            Err(format!(
                "File size exceeds maximum of {} bytes (500MB)",
                Self::MAX_FILE_SIZE
            ))
        } else {
            Ok(())
        }
    }

    /// Validates filename
    pub fn validate_filename(filename: &str) -> Result<(), String> {
        if filename.trim().is_empty() {
            Err("Filename cannot be empty".to_string())
        } else if filename.len() > 255 {
            Err("Filename cannot exceed 255 characters".to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_status_from_str() {
        assert_eq!(
            "in_progress".parse::<UploadStatus>().unwrap(),
            UploadStatus::InProgress
        );
        assert_eq!(
            "completed".parse::<UploadStatus>().unwrap(),
            UploadStatus::Completed
        );
        assert_eq!(
            "failed".parse::<UploadStatus>().unwrap(),
            UploadStatus::Failed
        );
        assert!("invalid".parse::<UploadStatus>().is_err());
    }

    #[test]
    fn test_calculate_total_chunks() {
        let chunk_size = 5 * 1024 * 1024; // 5MB
        assert_eq!(UploadSession::calculate_total_chunks(5 * 1024 * 1024, chunk_size), 1);
        assert_eq!(UploadSession::calculate_total_chunks(10 * 1024 * 1024, chunk_size), 2);
        assert_eq!(UploadSession::calculate_total_chunks(12 * 1024 * 1024, chunk_size), 3);
    }

    #[test]
    fn test_progress_percentage() {
        let session = UploadSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            lesson_id: Some(Uuid::new_v4()),
            filename: "test.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            total_size: 10 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            total_chunks: 2,
            uploaded_chunks: 1,
            storage_key: "test".to_string(),
            status: UploadStatus::InProgress,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            completed_at: None,
        };

        assert_eq!(session.progress_percentage(), 50.0);
    }

    #[test]
    fn test_is_expired() {
        let mut session = UploadSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            lesson_id: Some(Uuid::new_v4()),
            filename: "test.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            total_size: 10 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            total_chunks: 2,
            uploaded_chunks: 0,
            storage_key: "test".to_string(),
            status: UploadStatus::InProgress,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            completed_at: None,
        };

        assert!(!session.is_expired());

        session.expires_at = Utc::now() - chrono::Duration::hours(1);
        assert!(session.is_expired());
    }

    #[test]
    fn test_is_resumable() {
        let mut session = UploadSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            lesson_id: Some(Uuid::new_v4()),
            filename: "test.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            total_size: 10 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            total_chunks: 2,
            uploaded_chunks: 1,
            storage_key: "test".to_string(),
            status: UploadStatus::InProgress,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            completed_at: None,
        };

        assert!(session.is_resumable());

        session.status = UploadStatus::Completed;
        assert!(!session.is_resumable());

        session.status = UploadStatus::InProgress;
        session.expires_at = Utc::now() - chrono::Duration::hours(1);
        assert!(!session.is_resumable());
    }

    #[test]
    fn test_validate_chunk_index() {
        let session = UploadSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            lesson_id: Some(Uuid::new_v4()),
            filename: "test.mp4".to_string(),
            content_type: "video/mp4".to_string(),
            total_size: 10 * 1024 * 1024,
            chunk_size: 5 * 1024 * 1024,
            total_chunks: 2,
            uploaded_chunks: 0,
            storage_key: "test".to_string(),
            status: UploadStatus::InProgress,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(24),
            completed_at: None,
        };

        assert!(session.validate_chunk_index(0).is_ok());
        assert!(session.validate_chunk_index(1).is_ok());
        assert!(session.validate_chunk_index(-1).is_err());
        assert!(session.validate_chunk_index(2).is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(UploadSession::validate_file_size(1024).is_ok());
        assert!(UploadSession::validate_file_size(500 * 1024 * 1024).is_ok());
        assert!(UploadSession::validate_file_size(0).is_err());
        assert!(UploadSession::validate_file_size(-1).is_err());
        assert!(UploadSession::validate_file_size(501 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_validate_filename() {
        assert!(UploadSession::validate_filename("test.mp4").is_ok());
        assert!(UploadSession::validate_filename("").is_err());
        assert!(UploadSession::validate_filename(&"a".repeat(256)).is_err());
    }
}
