use super::{DownloadError, Result};
use crate::analytics::{AnalyticsEvent, AnalyticsPublisher, DownloadEvent};
use crate::db::repositories::{DownloadTrackingRepository, ResourceRepository};
use crate::models::{ContentType, CopyrightSetting, Resource};
use crate::storage::S3Client;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// DownloadManager handles content download operations with permission validation
pub struct DownloadManager {
    resource_repo: ResourceRepository,
    download_tracking_repo: DownloadTrackingRepository,
    s3_client: S3Client,
    analytics_publisher: Arc<AnalyticsPublisher>,
}

impl DownloadManager {
    /// Creates a new DownloadManager
    pub fn new(
        resource_repo: ResourceRepository,
        download_tracking_repo: DownloadTrackingRepository,
        s3_client: S3Client,
        analytics_publisher: Arc<AnalyticsPublisher>,
    ) -> Self {
        Self {
            resource_repo,
            download_tracking_repo,
            s3_client,
            analytics_publisher,
        }
    }

    /// Validates download permission for a resource
    /// 
    /// Checks:
    /// - Resource exists
    /// - Resource is published
    /// - Copyright settings allow download
    /// - Resource is marked as downloadable (for videos)
    #[instrument(skip(self))]
    pub async fn validate_download_permission(
        &self,
        resource_id: Uuid,
        user_id: Uuid,
        is_instructor: bool,
    ) -> Result<Resource> {
        debug!(
            "Validating download permission for resource {} by user {}",
            resource_id, user_id
        );

        // Fetch the resource
        let resource = self
            .resource_repo
            .find_by_id(resource_id)
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?
            .ok_or_else(|| DownloadError::ResourceNotFound(resource_id.to_string()))?;

        // Instructors can download any content
        if is_instructor {
            info!(
                "Instructor {} granted download access to resource {}",
                user_id, resource_id
            );
            return Ok(resource);
        }

        // Students can only download published content
        if !resource.published {
            warn!(
                "User {} attempted to download unpublished resource {}",
                user_id, resource_id
            );
            return Err(DownloadError::ResourceNotPublished);
        }

        // Check copyright restrictions
        match resource.copyright_setting {
            CopyrightSetting::NoDownload => {
                warn!(
                    "User {} attempted to download no-download resource {}",
                    user_id, resource_id
                );
                return Err(DownloadError::CopyrightRestriction(
                    "This content cannot be downloaded due to copyright restrictions".to_string(),
                ));
            }
            CopyrightSetting::EducationalUseOnly => {
                info!(
                    "User {} downloading educational-use-only resource {}",
                    user_id, resource_id
                );
                // Allow download but will include copyright notice
            }
            CopyrightSetting::Unrestricted => {
                debug!("Resource {} has unrestricted copyright", resource_id);
            }
        }

        // For videos, check the downloadable flag
        if resource.content_type == ContentType::Video && !resource.downloadable {
            warn!(
                "User {} attempted to download non-downloadable video {}",
                user_id, resource_id
            );
            return Err(DownloadError::DownloadNotAllowed(
                "This video is not available for download. Please stream it online.".to_string(),
            ));
        }

        // Log access attempt to copyrighted materials
        if resource.copyright_setting != CopyrightSetting::Unrestricted {
            info!(
                "Access to copyrighted material: user={}, resource={}, copyright={:?}",
                user_id, resource_id, resource.copyright_setting
            );
        }

        info!(
            "User {} granted download access to resource {}",
            user_id, resource_id
        );
        Ok(resource)
    }

    /// Generates a presigned download URL for a resource
    /// 
    /// Requirements:
    /// - 1-hour expiration for documents (PDF, DOCX)
    /// - 2-hour expiration for downloadable videos
    #[instrument(skip(self))]
    pub async fn generate_download_url(
        &self,
        resource_id: Uuid,
        user_id: Uuid,
        is_instructor: bool,
    ) -> Result<(String, chrono::DateTime<Utc>, Option<String>)> {
        info!(
            "Generating download URL for resource {} by user {}",
            resource_id, user_id
        );

        // Validate download permission
        let resource = self
            .validate_download_permission(resource_id, user_id, is_instructor)
            .await?;

        // Determine expiration based on content type
        let expiration = match resource.content_type {
            ContentType::Video => {
                debug!("Setting 2-hour expiration for video download");
                Duration::from_secs(2 * 60 * 60) // 2 hours
            }
            ContentType::Pdf | ContentType::Docx => {
                debug!("Setting 1-hour expiration for document download");
                Duration::from_secs(60 * 60) // 1 hour
            }
        };

        // Generate presigned URL
        let download_url = self
            .s3_client
            .generate_presigned_url(&resource.storage_key, expiration)
            .await
            .map_err(|e| {
                error!(
                    "Failed to generate presigned URL for resource {}: {}",
                    resource_id, e
                );
                DownloadError::StorageError(e.to_string())
            })?;

        let expires_at = Utc::now() + chrono::Duration::from_std(expiration).unwrap();

        // Get copyright notice if applicable
        let copyright_notice = resource.copyright_notice().map(|s| s.to_string());

        // Track the download
        if let Err(e) = self.track_download(user_id, resource_id, &resource).await {
            // Log error but don't fail the download
            error!("Failed to track download: {}", e);
        }

        info!(
            "Generated download URL for resource {} (expires at {})",
            resource_id, expires_at
        );

        Ok((download_url, expires_at, copyright_notice))
    }

    /// Tracks a download event
    /// 
    /// Records in database and sends to analytics service
    #[instrument(skip(self, resource))]
    async fn track_download(
        &self,
        user_id: Uuid,
        resource_id: Uuid,
        resource: &Resource,
    ) -> Result<()> {
        debug!(
            "Tracking download: user={}, resource={}",
            user_id, resource_id
        );

        // Record in database
        self.download_tracking_repo
            .record_download(user_id, resource_id)
            .await
            .map_err(|e| {
                error!("Failed to record download in database: {}", e);
                DownloadError::DatabaseError(e.to_string())
            })?;

        // Send to analytics service
        let event = AnalyticsEvent::Download(DownloadEvent {
            student_id: user_id,
            resource_id,
            resource_name: resource.name.clone(),
            content_type: resource.content_type.to_string(),
            file_size: resource.file_size,
            timestamp: Utc::now(),
        });

        self.analytics_publisher
            .publish_event(event)
            .await
            .map_err(|e| {
                warn!("Failed to publish download event to analytics: {}", e);
                // Don't fail the download if analytics fails
                e
            })?;

        info!("Download tracked successfully");
        Ok(())
    }

    /// Gets download statistics for a resource
    pub async fn get_download_stats(&self, resource_id: Uuid) -> Result<(i64, i64)> {
        let total_downloads = self
            .download_tracking_repo
            .count_by_resource(resource_id)
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        let unique_users = self
            .download_tracking_repo
            .count_unique_students_by_resource(resource_id)
            .await
            .map_err(|e| DownloadError::DatabaseError(e.to_string()))?;

        Ok((total_downloads, unique_users))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_error_display() {
        let err = DownloadError::ResourceNotFound("123".to_string());
        assert_eq!(err.to_string(), "Resource not found: 123");

        let err = DownloadError::DownloadNotAllowed("test".to_string());
        assert_eq!(err.to_string(), "Download not allowed: test");

        let err = DownloadError::CopyrightRestriction("restricted".to_string());
        assert_eq!(err.to_string(), "Copyright restriction: restricted");
    }
}
