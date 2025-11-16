use crate::models::{ProgressTracking, ProgressSummary};
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing ProgressTracking entities
pub struct ProgressRepository {
    pool: PgPool,
}

impl ProgressRepository {
    /// Creates a new ProgressRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates or updates progress tracking for a student and resource
    pub async fn upsert(
        &self,
        student_id: Uuid,
        resource_id: Uuid,
        completed: bool,
        last_position_seconds: Option<i32>,
    ) -> Result<ProgressTracking> {
        if let Some(pos) = last_position_seconds {
            ProgressTracking::validate_position(pos).map_err(|e| anyhow!(e))?;
        }

        let completed_at = if completed { Some("NOW()") } else { None };

        let progress = sqlx::query_as::<_, ProgressTracking>(
            r#"
            INSERT INTO progress_tracking (student_id, resource_id, completed, completed_at, last_position_seconds, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (student_id, resource_id)
            DO UPDATE SET
                completed = EXCLUDED.completed,
                completed_at = CASE 
                    WHEN EXCLUDED.completed = true AND progress_tracking.completed_at IS NULL 
                    THEN NOW() 
                    WHEN EXCLUDED.completed = false 
                    THEN NULL 
                    ELSE progress_tracking.completed_at 
                END,
                last_position_seconds = EXCLUDED.last_position_seconds,
                updated_at = NOW()
            RETURNING id, student_id, resource_id, completed, completed_at, last_position_seconds, updated_at
            "#,
        )
        .bind(student_id)
        .bind(resource_id)
        .bind(completed)
        .bind(completed_at)
        .bind(last_position_seconds)
        .fetch_one(&self.pool)
        .await
        .context("Failed to upsert progress tracking")?;

        Ok(progress)
    }

    /// Marks a resource as complete for a student
    pub async fn mark_complete(&self, student_id: Uuid, resource_id: Uuid) -> Result<ProgressTracking> {
        self.upsert(student_id, resource_id, true, None).await
    }

    /// Marks a resource as incomplete for a student
    pub async fn mark_incomplete(&self, student_id: Uuid, resource_id: Uuid) -> Result<ProgressTracking> {
        self.upsert(student_id, resource_id, false, None).await
    }

    /// Updates playback position for a video resource
    pub async fn update_playback_position(
        &self,
        student_id: Uuid,
        resource_id: Uuid,
        position_seconds: i32,
    ) -> Result<ProgressTracking> {
        ProgressTracking::validate_position(position_seconds).map_err(|e| anyhow!(e))?;

        let progress = sqlx::query_as::<_, ProgressTracking>(
            r#"
            INSERT INTO progress_tracking (student_id, resource_id, last_position_seconds, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (student_id, resource_id)
            DO UPDATE SET
                last_position_seconds = EXCLUDED.last_position_seconds,
                updated_at = NOW()
            RETURNING id, student_id, resource_id, completed, completed_at, last_position_seconds, updated_at
            "#,
        )
        .bind(student_id)
        .bind(resource_id)
        .bind(position_seconds)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update playback position")?;

        Ok(progress)
    }

    /// Finds progress tracking for a student and resource
    pub async fn find_by_student_and_resource(
        &self,
        student_id: Uuid,
        resource_id: Uuid,
    ) -> Result<Option<ProgressTracking>> {
        let progress = sqlx::query_as::<_, ProgressTracking>(
            r#"
            SELECT id, student_id, resource_id, completed, completed_at, last_position_seconds, updated_at
            FROM progress_tracking
            WHERE student_id = $1 AND resource_id = $2
            "#,
        )
        .bind(student_id)
        .bind(resource_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find progress tracking")?;

        Ok(progress)
    }

    /// Lists all progress for a student
    pub async fn list_by_student(&self, student_id: Uuid) -> Result<Vec<ProgressTracking>> {
        let progress = sqlx::query_as::<_, ProgressTracking>(
            r#"
            SELECT id, student_id, resource_id, completed, completed_at, last_position_seconds, updated_at
            FROM progress_tracking
            WHERE student_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list progress by student")?;

        Ok(progress)
    }

    /// Lists completed resources for a student
    pub async fn list_completed_by_student(&self, student_id: Uuid) -> Result<Vec<ProgressTracking>> {
        let progress = sqlx::query_as::<_, ProgressTracking>(
            r#"
            SELECT id, student_id, resource_id, completed, completed_at, last_position_seconds, updated_at
            FROM progress_tracking
            WHERE student_id = $1 AND completed = true
            ORDER BY completed_at DESC
            "#,
        )
        .bind(student_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list completed progress")?;

        Ok(progress)
    }

    /// Calculates progress summary for a student and course
    pub async fn calculate_course_progress(
        &self,
        student_id: Uuid,
        course_id: Uuid,
    ) -> Result<ProgressSummary> {
        let row: (i32, i32) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(DISTINCT r.id)::int as total_resources,
                COUNT(DISTINCT CASE WHEN pt.completed = true THEN r.id END)::int as completed_resources
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            LEFT JOIN progress_tracking pt ON r.id = pt.resource_id AND pt.student_id = $1
            WHERE m.course_id = $2 AND r.published = true
            "#,
        )
        .bind(student_id)
        .bind(course_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to calculate course progress")?;

        Ok(ProgressSummary::new(
            student_id,
            Some(course_id),
            None,
            None,
            row.0,
            row.1,
        ))
    }

    /// Calculates progress summary for a student and module
    pub async fn calculate_module_progress(
        &self,
        student_id: Uuid,
        module_id: Uuid,
    ) -> Result<ProgressSummary> {
        let row: (i32, i32) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(DISTINCT r.id)::int as total_resources,
                COUNT(DISTINCT CASE WHEN pt.completed = true THEN r.id END)::int as completed_resources
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            LEFT JOIN progress_tracking pt ON r.id = pt.resource_id AND pt.student_id = $1
            WHERE l.module_id = $2 AND r.published = true
            "#,
        )
        .bind(student_id)
        .bind(module_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to calculate module progress")?;

        Ok(ProgressSummary::new(
            student_id,
            None,
            Some(module_id),
            None,
            row.0,
            row.1,
        ))
    }

    /// Calculates progress summary for a student and lesson
    pub async fn calculate_lesson_progress(
        &self,
        student_id: Uuid,
        lesson_id: Uuid,
    ) -> Result<ProgressSummary> {
        let row: (i32, i32) = sqlx::query_as(
            r#"
            SELECT 
                COUNT(DISTINCT r.id)::int as total_resources,
                COUNT(DISTINCT CASE WHEN pt.completed = true THEN r.id END)::int as completed_resources
            FROM resources r
            LEFT JOIN progress_tracking pt ON r.id = pt.resource_id AND pt.student_id = $1
            WHERE r.lesson_id = $2 AND r.published = true
            "#,
        )
        .bind(student_id)
        .bind(lesson_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to calculate lesson progress")?;

        Ok(ProgressSummary::new(
            student_id,
            None,
            None,
            Some(lesson_id),
            row.0,
            row.1,
        ))
    }

    /// Gets progress report for all students in a course
    pub async fn get_course_progress_report(&self, course_id: Uuid) -> Result<Vec<ProgressSummary>> {
        let rows: Vec<(Uuid, i32, i32)> = sqlx::query_as(
            r#"
            SELECT 
                pt.student_id,
                COUNT(DISTINCT r.id)::int as total_resources,
                COUNT(DISTINCT CASE WHEN pt.completed = true THEN r.id END)::int as completed_resources
            FROM progress_tracking pt
            INNER JOIN resources r ON pt.resource_id = r.id
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1 AND r.published = true
            GROUP BY pt.student_id
            "#,
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to get course progress report")?;

        Ok(rows
            .into_iter()
            .map(|(student_id, total, completed)| {
                ProgressSummary::new(student_id, Some(course_id), None, None, total, completed)
            })
            .collect())
    }

    /// Counts completed resources for a student in a course
    pub async fn count_completed_by_course(
        &self,
        student_id: Uuid,
        course_id: Uuid,
    ) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(DISTINCT r.id)
            FROM progress_tracking pt
            INNER JOIN resources r ON pt.resource_id = r.id
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE pt.student_id = $1 AND m.course_id = $2 AND pt.completed = true AND r.published = true
            "#,
        )
        .bind(student_id)
        .bind(course_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count completed resources")?;

        Ok(count.0)
    }

    /// Deletes progress tracking for a resource
    pub async fn delete_by_resource(&self, resource_id: Uuid) -> Result<i64> {
        let result = sqlx::query("DELETE FROM progress_tracking WHERE resource_id = $1")
            .bind(resource_id)
            .execute(&self.pool)
            .await
            .context("Failed to delete progress tracking")?;

        Ok(result.rows_affected() as i64)
    }
}
