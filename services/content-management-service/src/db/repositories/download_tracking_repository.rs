use crate::models::DownloadTracking;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing DownloadTracking entities
pub struct DownloadTrackingRepository {
    pool: PgPool,
}

impl DownloadTrackingRepository {
    /// Creates a new DownloadTrackingRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Records a download event
    pub async fn record_download(
        &self,
        student_id: Uuid,
        resource_id: Uuid,
    ) -> Result<DownloadTracking> {
        let tracking = sqlx::query_as::<_, DownloadTracking>(
            r#"
            INSERT INTO download_tracking (student_id, resource_id)
            VALUES ($1, $2)
            RETURNING id, student_id, resource_id, downloaded_at
            "#,
        )
        .bind(student_id)
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to record download")?;

        Ok(tracking)
    }

    /// Lists all downloads for a student
    pub async fn list_by_student(&self, student_id: Uuid) -> Result<Vec<DownloadTracking>> {
        let downloads = sqlx::query_as::<_, DownloadTracking>(
            r#"
            SELECT id, student_id, resource_id, downloaded_at
            FROM download_tracking
            WHERE student_id = $1
            ORDER BY downloaded_at DESC
            "#,
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list downloads by student")?;

        Ok(downloads)
    }

    /// Lists all downloads for a resource
    pub async fn list_by_resource(&self, resource_id: Uuid) -> Result<Vec<DownloadTracking>> {
        let downloads = sqlx::query_as::<_, DownloadTracking>(
            r#"
            SELECT id, student_id, resource_id, downloaded_at
            FROM download_tracking
            WHERE resource_id = $1
            ORDER BY downloaded_at DESC
            "#,
        )
        .bind(resource_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list downloads by resource")?;

        Ok(downloads)
    }

    /// Lists downloads within a date range
    pub async fn list_by_date_range(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<DownloadTracking>> {
        let downloads = sqlx::query_as::<_, DownloadTracking>(
            r#"
            SELECT id, student_id, resource_id, downloaded_at
            FROM download_tracking
            WHERE downloaded_at >= $1 AND downloaded_at <= $2
            ORDER BY downloaded_at DESC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list downloads by date range")?;

        Ok(downloads)
    }

    /// Counts total downloads for a resource
    pub async fn count_by_resource(&self, resource_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM download_tracking WHERE resource_id = $1
            "#,
        )
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count downloads by resource")?;

        Ok(count.0)
    }

    /// Counts total downloads for a student
    pub async fn count_by_student(&self, student_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM download_tracking WHERE student_id = $1
            "#,
        )
        .bind(student_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count downloads by student")?;

        Ok(count.0)
    }

    /// Counts unique students who downloaded a resource
    pub async fn count_unique_students_by_resource(&self, resource_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT student_id) FROM download_tracking WHERE resource_id = $1
            "#,
        )
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count unique students")?;

        Ok(count.0)
    }

    /// Gets download statistics for a course
    pub async fn get_course_download_stats(&self, course_id: Uuid) -> Result<Vec<(Uuid, String, i64)>> {
        let stats: Vec<(Uuid, String, i64)> = sqlx::query_as(
            r#"
            SELECT r.id, r.name, COUNT(dt.id)::bigint as download_count
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            LEFT JOIN download_tracking dt ON r.id = dt.resource_id
            WHERE m.course_id = $1
            GROUP BY r.id, r.name
            ORDER BY download_count DESC
            "#,
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get course download stats")?;

        Ok(stats)
    }

    /// Deletes download tracking records older than a certain date
    pub async fn delete_older_than(&self, before: DateTime<Utc>) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM download_tracking WHERE downloaded_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .context("Failed to delete old download tracking records")?;

        Ok(result.rows_affected() as i64)
    }
}
