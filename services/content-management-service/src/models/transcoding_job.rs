use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// TranscodingStatus represents the current state of a transcoding job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum TranscodingStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "processing")]
    Processing,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
}

impl std::fmt::Display for TranscodingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscodingStatus::Pending => write!(f, "pending"),
            TranscodingStatus::Processing => write!(f, "processing"),
            TranscodingStatus::Completed => write!(f, "completed"),
            TranscodingStatus::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for TranscodingStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TranscodingStatus::Pending),
            "processing" => Ok(TranscodingStatus::Processing),
            "completed" => Ok(TranscodingStatus::Completed),
            "failed" => Ok(TranscodingStatus::Failed),
            _ => Err(format!("Invalid transcoding status: {}", s)),
        }
    }
}

/// TranscodingJob manages video transcoding operations
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TranscodingJob {
    pub id: Uuid,
    pub resource_id: Uuid,
    pub status: TranscodingStatus,
    pub retry_count: i32,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl TranscodingJob {
    /// Maximum number of retry attempts
    pub const MAX_RETRIES: i32 = 3;

    /// Checks if the job can be retried
    pub fn can_retry(&self) -> bool {
        self.status == TranscodingStatus::Failed && self.retry_count < Self::MAX_RETRIES
    }

    /// Checks if the job has exhausted all retries
    pub fn retries_exhausted(&self) -> bool {
        self.retry_count >= Self::MAX_RETRIES
    }

    /// Checks if the job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TranscodingStatus::Completed | TranscodingStatus::Failed
        ) && !self.can_retry()
    }

    /// Checks if the job is currently processing
    pub fn is_processing(&self) -> bool {
        self.status == TranscodingStatus::Processing
    }

    /// Calculates processing duration if job has started
    pub fn processing_duration(&self) -> Option<chrono::Duration> {
        self.started_at.map(|start| {
            let end = self.completed_at.unwrap_or_else(Utc::now);
            end.signed_duration_since(start)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcoding_status_from_str() {
        assert_eq!(
            "pending".parse::<TranscodingStatus>().unwrap(),
            TranscodingStatus::Pending
        );
        assert_eq!(
            "processing".parse::<TranscodingStatus>().unwrap(),
            TranscodingStatus::Processing
        );
        assert_eq!(
            "completed".parse::<TranscodingStatus>().unwrap(),
            TranscodingStatus::Completed
        );
        assert_eq!(
            "failed".parse::<TranscodingStatus>().unwrap(),
            TranscodingStatus::Failed
        );
        assert!("invalid".parse::<TranscodingStatus>().is_err());
    }

    #[test]
    fn test_can_retry() {
        let mut job = TranscodingJob {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            status: TranscodingStatus::Failed,
            retry_count: 0,
            error_message: Some("Test error".to_string()),
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        assert!(job.can_retry());

        job.retry_count = 3;
        assert!(!job.can_retry());

        job.retry_count = 0;
        job.status = TranscodingStatus::Completed;
        assert!(!job.can_retry());
    }

    #[test]
    fn test_retries_exhausted() {
        let mut job = TranscodingJob {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            status: TranscodingStatus::Failed,
            retry_count: 0,
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        assert!(!job.retries_exhausted());

        job.retry_count = 3;
        assert!(job.retries_exhausted());

        job.retry_count = 4;
        assert!(job.retries_exhausted());
    }

    #[test]
    fn test_is_terminal() {
        let mut job = TranscodingJob {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            status: TranscodingStatus::Completed,
            retry_count: 0,
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: Some(Utc::now()),
        };

        assert!(job.is_terminal());

        job.status = TranscodingStatus::Failed;
        job.retry_count = 3;
        assert!(job.is_terminal());

        job.retry_count = 0;
        assert!(!job.is_terminal()); // Can still retry

        job.status = TranscodingStatus::Processing;
        assert!(!job.is_terminal());
    }

    #[test]
    fn test_is_processing() {
        let mut job = TranscodingJob {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            status: TranscodingStatus::Processing,
            retry_count: 0,
            error_message: None,
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: None,
        };

        assert!(job.is_processing());

        job.status = TranscodingStatus::Completed;
        assert!(!job.is_processing());
    }

    #[test]
    fn test_processing_duration() {
        let now = Utc::now();
        let start = now - chrono::Duration::minutes(5);
        
        let mut job = TranscodingJob {
            id: Uuid::new_v4(),
            resource_id: Uuid::new_v4(),
            status: TranscodingStatus::Processing,
            retry_count: 0,
            error_message: None,
            created_at: now,
            started_at: Some(start),
            completed_at: None,
        };

        let duration = job.processing_duration();
        assert!(duration.is_some());
        assert!(duration.unwrap().num_seconds() >= 300); // At least 5 minutes

        job.started_at = None;
        assert!(job.processing_duration().is_none());
    }
}
