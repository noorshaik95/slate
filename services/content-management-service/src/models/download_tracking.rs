use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// DownloadTracking records content download events
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DownloadTracking {
    pub id: Uuid,
    pub student_id: Uuid,
    pub resource_id: Uuid,
    pub downloaded_at: DateTime<Utc>,
}

impl DownloadTracking {
    /// Creates a new download tracking record
    pub fn new(student_id: Uuid, resource_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            student_id,
            resource_id,
            downloaded_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_download_tracking() {
        let student_id = Uuid::new_v4();
        let resource_id = Uuid::new_v4();
        
        let tracking = DownloadTracking::new(student_id, resource_id);
        
        assert_eq!(tracking.student_id, student_id);
        assert_eq!(tracking.resource_id, resource_id);
        assert!(tracking.downloaded_at <= Utc::now());
    }
}
