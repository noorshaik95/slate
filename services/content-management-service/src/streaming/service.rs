use crate::analytics::{AnalyticsEvent, AnalyticsPublisher, VideoPlayEvent, VideoPauseEvent, VideoSeekEvent, VideoCompleteEvent};
use crate::db::repositories::{ProgressRepository, ResourceRepository};
use crate::models::{ContentType, ProgressTracking};
use crate::streaming::errors::StreamingError;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Type of playback event for analytics
#[derive(Debug, Clone)]
pub enum PlaybackEventType {
    /// Video was paused
    Pause,
    /// User seeked to a new position
    Seek { old_position: i32 },
    /// Regular position update (no specific event)
    Update,
}

/// Quality levels for adaptive bitrate streaming
#[derive(Debug, Clone)]
pub struct QualityLevel {
    pub height: i32,
    pub bitrate: i32,
}

impl QualityLevel {
    /// Standard quality levels for transcoded videos
    pub fn standard_levels() -> Vec<QualityLevel> {
        vec![
            QualityLevel {
                height: 360,
                bitrate: 800,
            },
            QualityLevel {
                height: 480,
                bitrate: 1400,
            },
            QualityLevel {
                height: 720,
                bitrate: 2800,
            },
            QualityLevel {
                height: 1080,
                bitrate: 5000,
            },
        ]
    }
}

/// Video manifest containing streaming URLs
#[derive(Debug, Clone)]
pub struct VideoManifest {
    pub hls_url: Option<String>,
    pub dash_url: Option<String>,
    pub quality_levels: Vec<QualityLevel>,
}

/// Playback state for a video
#[derive(Debug, Clone)]
pub struct PlaybackState {
    pub duration_seconds: i32,
    pub current_position_seconds: i32,
    pub playback_speeds: Vec<f32>,
    pub available_qualities: Vec<QualityLevel>,
}

impl PlaybackState {
    /// Standard playback speed options
    pub fn standard_playback_speeds() -> Vec<f32> {
        vec![0.5, 0.75, 1.0, 1.25, 1.5, 2.0]
    }
}

/// StreamingService manages video streaming and playback tracking
pub struct StreamingService {
    resource_repo: ResourceRepository,
    progress_repo: ProgressRepository,
    analytics_publisher: Option<Arc<AnalyticsPublisher>>,
}

impl StreamingService {
    /// Creates a new StreamingService
    pub fn new(resource_repo: ResourceRepository, progress_repo: ProgressRepository) -> Self {
        Self {
            resource_repo,
            progress_repo,
            analytics_publisher: None,
        }
    }

    /// Creates a new StreamingService with analytics publisher
    pub fn with_analytics(
        resource_repo: ResourceRepository,
        progress_repo: ProgressRepository,
        analytics_publisher: Arc<AnalyticsPublisher>,
    ) -> Self {
        Self {
            resource_repo,
            progress_repo,
            analytics_publisher: Some(analytics_publisher),
        }
    }

    /// Publishes a video play event to analytics
    async fn publish_play_event(&self, student_id: Uuid, video_id: Uuid, session_id: Option<String>) {
        if let Some(publisher) = &self.analytics_publisher {
            let event = AnalyticsEvent::VideoPlay(VideoPlayEvent {
                student_id,
                video_id,
                timestamp: Utc::now(),
                session_id,
            });
            
            if let Err(e) = publisher.publish_event(event).await {
                tracing::warn!("Failed to publish video play event: {}", e);
            }
        }
    }

    /// Publishes a video pause event to analytics
    async fn publish_pause_event(&self, student_id: Uuid, video_id: Uuid, position_seconds: i32, session_id: Option<String>) {
        if let Some(publisher) = &self.analytics_publisher {
            let event = AnalyticsEvent::VideoPause(VideoPauseEvent {
                student_id,
                video_id,
                position_seconds,
                timestamp: Utc::now(),
                session_id,
            });
            
            if let Err(e) = publisher.publish_event(event).await {
                tracing::warn!("Failed to publish video pause event: {}", e);
            }
        }
    }

    /// Publishes a video seek event to analytics
    async fn publish_seek_event(&self, student_id: Uuid, video_id: Uuid, old_position: i32, new_position: i32, session_id: Option<String>) {
        if let Some(publisher) = &self.analytics_publisher {
            let event = AnalyticsEvent::VideoSeek(VideoSeekEvent {
                student_id,
                video_id,
                old_position_seconds: old_position,
                new_position_seconds: new_position,
                timestamp: Utc::now(),
                session_id,
            });
            
            if let Err(e) = publisher.publish_event(event).await {
                tracing::warn!("Failed to publish video seek event: {}", e);
            }
        }
    }

    /// Publishes a video complete event to analytics
    async fn publish_complete_event(&self, student_id: Uuid, video_id: Uuid, session_id: Option<String>) {
        if let Some(publisher) = &self.analytics_publisher {
            let event = AnalyticsEvent::VideoComplete(VideoCompleteEvent {
                student_id,
                video_id,
                timestamp: Utc::now(),
                session_id,
            });
            
            if let Err(e) = publisher.publish_event(event).await {
                tracing::warn!("Failed to publish video complete event: {}", e);
            }
        }
    }

    /// Gets video manifest for streaming
    /// 
    /// Requirements: 6.3, 7.1, 12.1
    /// - Validates video resource exists and is published
    /// - Checks user access permissions (students can only access published content)
    /// - Returns HLS/DASH manifest URLs
    /// - Publishes play event to analytics
    pub async fn get_video_manifest(
        &self,
        video_id: Uuid,
        user_id: Uuid,
        is_instructor: bool,
        session_id: Option<String>,
    ) -> Result<VideoManifest, StreamingError> {
        // Find the resource
        let resource = self
            .resource_repo
            .find_by_id(video_id)
            .await?
            .ok_or_else(|| StreamingError::VideoNotFound(video_id.to_string()))?;

        // Validate it's a video
        if resource.content_type != ContentType::Video {
            return Err(StreamingError::NotAVideo);
        }

        // Check publication status and access permissions
        if !resource.published && !is_instructor {
            return Err(StreamingError::AccessDenied(
                "Video is not published".to_string(),
            ));
        }

        // Check if video has been transcoded
        let manifest_url = resource
            .manifest_url
            .ok_or(StreamingError::NotTranscoded)?;

        // Parse manifest URL to generate HLS and DASH URLs
        // Assuming manifest_url is the base path, we append format-specific paths
        let hls_url = Some(format!("{}/master.m3u8", manifest_url));
        let dash_url = Some(format!("{}/manifest.mpd", manifest_url));

        // Publish play event to analytics (only for students, not instructors previewing)
        if !is_instructor {
            self.publish_play_event(user_id, video_id, session_id).await;
        }

        Ok(VideoManifest {
            hls_url,
            dash_url,
            quality_levels: QualityLevel::standard_levels(),
        })
    }

    /// Updates playback position for a video
    /// 
    /// Requirements: 7.2, 7.4, 7.5, 12.2, 12.3, 12.4
    /// - Accepts timestamp in seconds
    /// - Validates position is within video duration
    /// - Persists last playback position in progress_tracking table
    /// - Auto-completes video when reaching 90% of duration
    /// - Publishes pause/seek/complete events to analytics
    pub async fn update_playback_position(
        &self,
        video_id: Uuid,
        user_id: Uuid,
        position_seconds: i32,
        event_type: PlaybackEventType,
        session_id: Option<String>,
    ) -> Result<(), StreamingError> {
        // Validate position is non-negative
        ProgressTracking::validate_position(position_seconds)
            .map_err(|e| StreamingError::InvalidPosition(e))?;

        // Find the resource to get duration
        let resource = self
            .resource_repo
            .find_by_id(video_id)
            .await?
            .ok_or_else(|| StreamingError::VideoNotFound(video_id.to_string()))?;

        // Validate it's a video
        if resource.content_type != ContentType::Video {
            return Err(StreamingError::NotAVideo);
        }

        // Get video duration
        let duration = resource
            .duration_seconds
            .ok_or_else(|| StreamingError::InvalidPosition("Video has no duration".to_string()))?;

        // Validate position is within duration
        ProgressTracking::validate_position_within_duration(position_seconds, duration)
            .map_err(|e| StreamingError::InvalidPosition(e))?;

        // Get current position for seek event detection (used in seek events)
        let _current_progress = self
            .progress_repo
            .find_by_student_and_resource(user_id, video_id)
            .await?;

        // Publish analytics events based on event type
        match event_type {
            PlaybackEventType::Pause => {
                self.publish_pause_event(user_id, video_id, position_seconds, session_id.clone()).await;
            }
            PlaybackEventType::Seek { old_position } => {
                self.publish_seek_event(user_id, video_id, old_position, position_seconds, session_id.clone()).await;
            }
            PlaybackEventType::Update => {
                // No specific event for regular updates
            }
        }

        // Check if video should be auto-completed (90% threshold)
        let should_complete = ProgressTracking::should_auto_complete(position_seconds, duration);

        if should_complete {
            // Mark as complete
            self.progress_repo
                .mark_complete(user_id, video_id)
                .await?;
            
            // Publish completion event
            self.publish_complete_event(user_id, video_id, session_id).await;
        } else {
            // Just update position
            self.progress_repo
                .update_playback_position(user_id, video_id, position_seconds)
                .await?;
        }

        Ok(())
    }

    /// Gets playback state for a video
    /// 
    /// Requirements: 7.1, 7.3
    /// - Returns video metadata (duration, current position, available quality levels)
    /// - Includes playback speed options
    pub async fn get_playback_state(
        &self,
        video_id: Uuid,
        user_id: Uuid,
        is_instructor: bool,
    ) -> Result<PlaybackState, StreamingError> {
        // Find the resource
        let resource = self
            .resource_repo
            .find_by_id(video_id)
            .await?
            .ok_or_else(|| StreamingError::VideoNotFound(video_id.to_string()))?;

        // Validate it's a video
        if resource.content_type != ContentType::Video {
            return Err(StreamingError::NotAVideo);
        }

        // Check publication status and access permissions
        if !resource.published && !is_instructor {
            return Err(StreamingError::AccessDenied(
                "Video is not published".to_string(),
            ));
        }

        // Get video duration
        let duration_seconds = resource
            .duration_seconds
            .ok_or_else(|| StreamingError::InvalidPosition("Video has no duration".to_string()))?;

        // Get current playback position from progress tracking
        let progress = self
            .progress_repo
            .find_by_student_and_resource(user_id, video_id)
            .await?;

        let current_position_seconds = progress
            .and_then(|p| p.last_position_seconds)
            .unwrap_or(0);

        Ok(PlaybackState {
            duration_seconds,
            current_position_seconds,
            playback_speeds: PlaybackState::standard_playback_speeds(),
            available_qualities: QualityLevel::standard_levels(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_levels() {
        let levels = QualityLevel::standard_levels();
        assert_eq!(levels.len(), 4);
        assert_eq!(levels[0].height, 360);
        assert_eq!(levels[1].height, 480);
        assert_eq!(levels[2].height, 720);
        assert_eq!(levels[3].height, 1080);
    }

    #[test]
    fn test_playback_speeds() {
        let speeds = PlaybackState::standard_playback_speeds();
        assert_eq!(speeds.len(), 6);
        assert_eq!(speeds[0], 0.5);
        assert_eq!(speeds[2], 1.0);
        assert_eq!(speeds[5], 2.0);
    }
}
