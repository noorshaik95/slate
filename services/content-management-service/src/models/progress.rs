use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// ProgressTracking records student progress on resources
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProgressTracking {
    pub id: Uuid,
    pub student_id: Uuid,
    pub resource_id: Uuid,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_position_seconds: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

impl ProgressTracking {
    /// Video completion threshold (90% of duration)
    pub const VIDEO_COMPLETION_THRESHOLD: f64 = 0.9;

    /// Validates playback position is non-negative
    pub fn validate_position(position: i32) -> Result<(), String> {
        if position < 0 {
            Err("Playback position must be non-negative".to_string())
        } else {
            Ok(())
        }
    }

    /// Checks if video should be auto-completed based on position and duration
    pub fn should_auto_complete(position: i32, duration: i32) -> bool {
        if duration <= 0 {
            return false;
        }
        let progress = position as f64 / duration as f64;
        progress >= Self::VIDEO_COMPLETION_THRESHOLD
    }

    /// Validates position is within video duration
    pub fn validate_position_within_duration(position: i32, duration: i32) -> Result<(), String> {
        if position > duration {
            Err(format!(
                "Position {} exceeds video duration {}",
                position, duration
            ))
        } else {
            Ok(())
        }
    }
}

/// ProgressSummary aggregates progress data for a student
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSummary {
    pub student_id: Uuid,
    pub course_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    pub lesson_id: Option<Uuid>,
    pub total_resources: i32,
    pub completed_resources: i32,
    pub progress_percentage: i32,
}

impl ProgressSummary {
    /// Calculates progress percentage from completed and total resources
    pub fn calculate_percentage(completed: i32, total: i32) -> i32 {
        if total == 0 {
            return 0;
        }
        ((completed as f64 / total as f64) * 100.0).round() as i32
    }

    /// Creates a new progress summary
    pub fn new(
        student_id: Uuid,
        course_id: Option<Uuid>,
        module_id: Option<Uuid>,
        lesson_id: Option<Uuid>,
        total_resources: i32,
        completed_resources: i32,
    ) -> Self {
        let progress_percentage = Self::calculate_percentage(completed_resources, total_resources);
        Self {
            student_id,
            course_id,
            module_id,
            lesson_id,
            total_resources,
            completed_resources,
            progress_percentage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_position() {
        assert!(ProgressTracking::validate_position(0).is_ok());
        assert!(ProgressTracking::validate_position(100).is_ok());
        assert!(ProgressTracking::validate_position(-1).is_err());
    }

    #[test]
    fn test_should_auto_complete() {
        // 90% threshold
        assert!(ProgressTracking::should_auto_complete(90, 100));
        assert!(ProgressTracking::should_auto_complete(95, 100));
        assert!(ProgressTracking::should_auto_complete(100, 100));
        assert!(!ProgressTracking::should_auto_complete(89, 100));
        assert!(!ProgressTracking::should_auto_complete(50, 100));

        // Edge cases
        assert!(!ProgressTracking::should_auto_complete(0, 0));
        assert!(!ProgressTracking::should_auto_complete(10, -1));
    }

    #[test]
    fn test_validate_position_within_duration() {
        assert!(ProgressTracking::validate_position_within_duration(50, 100).is_ok());
        assert!(ProgressTracking::validate_position_within_duration(100, 100).is_ok());
        assert!(ProgressTracking::validate_position_within_duration(101, 100).is_err());
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(ProgressSummary::calculate_percentage(0, 10), 0);
        assert_eq!(ProgressSummary::calculate_percentage(5, 10), 50);
        assert_eq!(ProgressSummary::calculate_percentage(10, 10), 100);
        assert_eq!(ProgressSummary::calculate_percentage(3, 10), 30);
        assert_eq!(ProgressSummary::calculate_percentage(7, 10), 70);
        
        // Rounding
        assert_eq!(ProgressSummary::calculate_percentage(1, 3), 33);
        assert_eq!(ProgressSummary::calculate_percentage(2, 3), 67);

        // Edge case: no resources
        assert_eq!(ProgressSummary::calculate_percentage(0, 0), 0);
    }

    #[test]
    fn test_progress_summary_new() {
        let student_id = Uuid::new_v4();
        let course_id = Uuid::new_v4();
        
        let summary = ProgressSummary::new(
            student_id,
            Some(course_id),
            None,
            None,
            10,
            5,
        );

        assert_eq!(summary.student_id, student_id);
        assert_eq!(summary.course_id, Some(course_id));
        assert_eq!(summary.total_resources, 10);
        assert_eq!(summary.completed_resources, 5);
        assert_eq!(summary.progress_percentage, 50);
    }
}
