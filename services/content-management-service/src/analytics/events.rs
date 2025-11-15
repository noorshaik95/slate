use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Analytics event types for tracking content interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum AnalyticsEvent {
    VideoPlay(VideoPlayEvent),
    VideoPause(VideoPauseEvent),
    VideoSeek(VideoSeekEvent),
    VideoComplete(VideoCompleteEvent),
    Download(DownloadEvent),
}

/// Event published when a student starts video playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPlayEvent {
    pub student_id: Uuid,
    pub video_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

/// Event published when a student pauses video playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPauseEvent {
    pub student_id: Uuid,
    pub video_id: Uuid,
    pub position_seconds: i32,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

/// Event published when a student seeks within a video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSeekEvent {
    pub student_id: Uuid,
    pub video_id: Uuid,
    pub old_position_seconds: i32,
    pub new_position_seconds: i32,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

/// Event published when a student completes a video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCompleteEvent {
    pub student_id: Uuid,
    pub video_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

/// Event published when a student downloads content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEvent {
    pub student_id: Uuid,
    pub resource_id: Uuid,
    pub resource_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub timestamp: DateTime<Utc>,
}

impl AnalyticsEvent {
    /// Get the timestamp of the event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AnalyticsEvent::VideoPlay(e) => e.timestamp,
            AnalyticsEvent::VideoPause(e) => e.timestamp,
            AnalyticsEvent::VideoSeek(e) => e.timestamp,
            AnalyticsEvent::VideoComplete(e) => e.timestamp,
            AnalyticsEvent::Download(e) => e.timestamp,
        }
    }

    /// Get the student ID associated with the event
    pub fn student_id(&self) -> Uuid {
        match self {
            AnalyticsEvent::VideoPlay(e) => e.student_id,
            AnalyticsEvent::VideoPause(e) => e.student_id,
            AnalyticsEvent::VideoSeek(e) => e.student_id,
            AnalyticsEvent::VideoComplete(e) => e.student_id,
            AnalyticsEvent::Download(e) => e.student_id,
        }
    }

    /// Check if the event is older than the specified duration
    pub fn is_older_than(&self, duration: chrono::Duration) -> bool {
        let now = Utc::now();
        now.signed_duration_since(self.timestamp()) > duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_timestamp() {
        let timestamp = Utc::now();
        let event = AnalyticsEvent::VideoPlay(VideoPlayEvent {
            student_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            timestamp,
            session_id: None,
        });

        assert_eq!(event.timestamp(), timestamp);
    }

    #[test]
    fn test_event_student_id() {
        let student_id = Uuid::new_v4();
        let event = AnalyticsEvent::VideoPlay(VideoPlayEvent {
            student_id,
            video_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: None,
        });

        assert_eq!(event.student_id(), student_id);
    }

    #[test]
    fn test_event_is_older_than() {
        let old_timestamp = Utc::now() - chrono::Duration::hours(25);
        let event = AnalyticsEvent::VideoPlay(VideoPlayEvent {
            student_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            timestamp: old_timestamp,
            session_id: None,
        });

        assert!(event.is_older_than(chrono::Duration::hours(24)));
    }

    #[test]
    fn test_event_serialization() {
        let event = AnalyticsEvent::VideoPlay(VideoPlayEvent {
            student_id: Uuid::new_v4(),
            video_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_id: Some("session123".to_string()),
        });

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AnalyticsEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.student_id(), deserialized.student_id());
    }
}
