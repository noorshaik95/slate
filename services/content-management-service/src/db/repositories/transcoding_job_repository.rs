use crate::models::{TranscodingJob, TranscodingStatus};
use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing TranscodingJob entities
pub struct TranscodingJobRepository {
    pool: PgPool,
}

impl TranscodingJobRepository {
    /// Creates a new TranscodingJobRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new transcoding job
    pub async fn create(&self, resource_id: Uuid) -> Result<TranscodingJob> {
        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            INSERT INTO transcoding_jobs (resource_id)
            VALUES ($1)
            RETURNING id, resource_id, status, retry_count, error_message, 
                      created_at, started_at, completed_at
            "#,
        )
        .bind(resource_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create transcoding job")?;

        Ok(job)
    }

    /// Finds a transcoding job by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TranscodingJob>> {
        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            SELECT id, resource_id, status, retry_count, error_message, 
                   created_at, started_at, completed_at
            FROM transcoding_jobs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find transcoding job by ID")?;

        Ok(job)
    }

    /// Finds a transcoding job by resource ID
    pub async fn find_by_resource(&self, resource_id: Uuid) -> Result<Option<TranscodingJob>> {
        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            SELECT id, resource_id, status, retry_count, error_message, 
                   created_at, started_at, completed_at
            FROM transcoding_jobs
            WHERE resource_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(resource_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find transcoding job by resource")?;

        Ok(job)
    }

    /// Lists pending transcoding jobs
    pub async fn list_pending(&self, limit: i64) -> Result<Vec<TranscodingJob>> {
        let jobs = sqlx::query_as::<_, TranscodingJob>(
            r#"
            SELECT id, resource_id, status, retry_count, error_message, 
                   created_at, started_at, completed_at
            FROM transcoding_jobs
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list pending transcoding jobs")?;

        Ok(jobs)
    }

    /// Lists failed jobs that can be retried
    pub async fn list_retryable(&self, limit: i64) -> Result<Vec<TranscodingJob>> {
        let jobs = sqlx::query_as::<_, TranscodingJob>(
            r#"
            SELECT id, resource_id, status, retry_count, error_message, 
                   created_at, started_at, completed_at
            FROM transcoding_jobs
            WHERE status = 'failed' AND retry_count < $1
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(TranscodingJob::MAX_RETRIES)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list retryable transcoding jobs")?;

        Ok(jobs)
    }

    /// Updates the status of a transcoding job
    pub async fn update_status(
        &self,
        id: Uuid,
        status: TranscodingStatus,
        error_message: Option<String>,
    ) -> Result<TranscodingJob> {
        let started_at = if status == TranscodingStatus::Processing {
            Some("NOW()")
        } else {
            None
        };

        let completed_at = if matches!(
            status,
            TranscodingStatus::Completed | TranscodingStatus::Failed
        ) {
            Some("NOW()")
        } else {
            None
        };

        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            UPDATE transcoding_jobs
            SET status = $2,
                error_message = $3,
                started_at = COALESCE($4, started_at),
                completed_at = COALESCE($5, completed_at)
            WHERE id = $1
            RETURNING id, resource_id, status, retry_count, error_message, 
                      created_at, started_at, completed_at
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(error_message)
        .bind(started_at)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update transcoding job status")?;

        Ok(job)
    }

    /// Marks a job as processing
    pub async fn mark_processing(&self, id: Uuid) -> Result<TranscodingJob> {
        self.update_status(id, TranscodingStatus::Processing, None)
            .await
    }

    /// Marks a job as completed
    pub async fn mark_completed(&self, id: Uuid) -> Result<TranscodingJob> {
        self.update_status(id, TranscodingStatus::Completed, None)
            .await
    }

    /// Marks a job as failed and increments retry count
    pub async fn mark_failed(&self, id: Uuid, error_message: String) -> Result<TranscodingJob> {
        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            UPDATE transcoding_jobs
            SET status = 'failed',
                error_message = $2,
                retry_count = retry_count + 1,
                completed_at = NOW()
            WHERE id = $1
            RETURNING id, resource_id, status, retry_count, error_message, 
                      created_at, started_at, completed_at
            "#,
        )
        .bind(id)
        .bind(error_message)
        .fetch_one(&self.pool)
        .await
        .context("Failed to mark transcoding job as failed")?;

        Ok(job)
    }

    /// Resets a failed job to pending for retry
    pub async fn reset_for_retry(&self, id: Uuid) -> Result<TranscodingJob> {
        let job = sqlx::query_as::<_, TranscodingJob>(
            r#"
            UPDATE transcoding_jobs
            SET status = 'pending',
                started_at = NULL,
                completed_at = NULL
            WHERE id = $1
            RETURNING id, resource_id, status, retry_count, error_message, 
                      created_at, started_at, completed_at
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to reset transcoding job for retry")?;

        Ok(job)
    }

    /// Counts jobs by status
    pub async fn count_by_status(&self, status: TranscodingStatus) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM transcoding_jobs WHERE status = $1
            "#,
        )
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count transcoding jobs by status")?;

        Ok(count.0)
    }

    /// Counts pending jobs (queue size)
    pub async fn count_pending(&self) -> Result<i64> {
        self.count_by_status(TranscodingStatus::Pending).await
    }

    /// Deletes a transcoding job
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM transcoding_jobs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete transcoding job")?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes completed jobs older than a certain number of days
    pub async fn delete_completed_older_than_days(&self, days: i32) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM transcoding_jobs
            WHERE status = 'completed' AND completed_at < NOW() - INTERVAL '1 day' * $1
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .context("Failed to delete old completed transcoding jobs")?;

        Ok(result.rows_affected() as i64)
    }
}
