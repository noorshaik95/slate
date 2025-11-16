use crate::models::{UploadSession, UploadStatus};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing UploadSession entities
pub struct UploadSessionRepository {
    pool: PgPool,
}

impl UploadSessionRepository {
    /// Creates a new UploadSessionRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new upload session
    pub async fn create(
        &self,
        user_id: Uuid,
        lesson_id: Option<Uuid>,
        filename: String,
        content_type: String,
        total_size: i64,
        chunk_size: i32,
        storage_key: String,
    ) -> Result<UploadSession> {
        UploadSession::validate_filename(&filename).map_err(|e| anyhow!(e))?;
        UploadSession::validate_file_size(total_size).map_err(|e| anyhow!(e))?;

        let total_chunks = UploadSession::calculate_total_chunks(total_size, chunk_size);
        let expires_at = Utc::now() + Duration::hours(UploadSession::EXPIRATION_HOURS);

        let session = sqlx::query_as::<_, UploadSession>(
            r#"
            INSERT INTO upload_sessions (
                user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                total_chunks, storage_key, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                      total_chunks, uploaded_chunks, storage_key, status, 
                      created_at, expires_at, completed_at
            "#,
        )
        .bind(user_id)
        .bind(lesson_id)
        .bind(filename)
        .bind(content_type)
        .bind(total_size)
        .bind(chunk_size)
        .bind(total_chunks)
        .bind(storage_key)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create upload session")?;

        Ok(session)
    }

    /// Finds an upload session by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UploadSession>> {
        let session = sqlx::query_as::<_, UploadSession>(
            r#"
            SELECT id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                   total_chunks, uploaded_chunks, storage_key, status, 
                   created_at, expires_at, completed_at
            FROM upload_sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find upload session by ID")?;

        Ok(session)
    }

    /// Lists all upload sessions for a user
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<UploadSession>> {
        let sessions = sqlx::query_as::<_, UploadSession>(
            r#"
            SELECT id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                   total_chunks, uploaded_chunks, storage_key, status, 
                   created_at, expires_at, completed_at
            FROM upload_sessions
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list upload sessions by user")?;

        Ok(sessions)
    }

    /// Lists active (in-progress) upload sessions for a user
    pub async fn list_active_by_user(&self, user_id: Uuid) -> Result<Vec<UploadSession>> {
        let sessions = sqlx::query_as::<_, UploadSession>(
            r#"
            SELECT id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                   total_chunks, uploaded_chunks, storage_key, status, 
                   created_at, expires_at, completed_at
            FROM upload_sessions
            WHERE user_id = $1 AND status = 'in_progress' AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list active upload sessions")?;

        Ok(sessions)
    }

    /// Increments the uploaded chunks counter
    pub async fn increment_uploaded_chunks(&self, id: Uuid) -> Result<UploadSession> {
        let session = sqlx::query_as::<_, UploadSession>(
            r#"
            UPDATE upload_sessions
            SET uploaded_chunks = uploaded_chunks + 1
            WHERE id = $1
            RETURNING id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                      total_chunks, uploaded_chunks, storage_key, status, 
                      created_at, expires_at, completed_at
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to increment uploaded chunks")?;

        Ok(session)
    }

    /// Updates the status of an upload session
    pub async fn update_status(&self, id: Uuid, status: UploadStatus) -> Result<UploadSession> {
        let completed_at = if status == UploadStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };

        let session = sqlx::query_as::<_, UploadSession>(
            r#"
            UPDATE upload_sessions
            SET status = $2,
                completed_at = COALESCE($3, completed_at)
            WHERE id = $1
            RETURNING id, user_id, lesson_id, filename, content_type, total_size, chunk_size, 
                      total_chunks, uploaded_chunks, storage_key, status, 
                      created_at, expires_at, completed_at
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update upload session status")?;

        Ok(session)
    }

    /// Marks expired sessions
    pub async fn mark_expired_sessions(&self) -> Result<i64> {
        let result = sqlx::query(
            r#"
            UPDATE upload_sessions
            SET status = 'expired'
            WHERE status = 'in_progress' AND expires_at <= NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to mark expired sessions")?;

        Ok(result.rows_affected() as i64)
    }

    /// Deletes an upload session
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM upload_sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete upload session")?;

        Ok(result.rows_affected() > 0)
    }

    /// Deletes expired sessions older than a certain date
    pub async fn delete_expired_before(&self, before: DateTime<Utc>) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM upload_sessions
            WHERE status = 'expired' AND expires_at < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .context("Failed to delete expired sessions")?;

        Ok(result.rows_affected() as i64)
    }

    /// Counts active upload sessions for a user
    pub async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM upload_sessions
            WHERE user_id = $1 AND status = 'in_progress' AND expires_at > NOW()
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count active upload sessions")?;

        Ok(count.0)
    }

    /// Cleanup expired sessions (mark as expired and delete old ones)
    pub async fn cleanup_expired_sessions(&self) -> Result<(i64, i64)> {
        // Mark expired sessions
        let marked = self.mark_expired_sessions().await?;

        // Delete sessions that expired more than 7 days ago
        let delete_before = Utc::now() - Duration::days(7);
        let deleted = self.delete_expired_before(delete_before).await?;

        Ok((marked, deleted))
    }
}
