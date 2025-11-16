use crate::models::Lesson;
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing Lesson entities
pub struct LessonRepository {
    pool: PgPool,
}

impl LessonRepository {
    /// Creates a new LessonRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new lesson
    pub async fn create(
        &self,
        module_id: Uuid,
        name: String,
        description: Option<String>,
        display_order: i32,
    ) -> Result<Lesson> {
        Lesson::validate_name(&name).map_err(|e| anyhow!(e))?;
        Lesson::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let lesson = sqlx::query_as::<_, Lesson>(
            r#"
            INSERT INTO lessons (module_id, name, description, display_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id, module_id, name, description, display_order, created_at, updated_at
            "#,
        )
        .bind(module_id)
        .bind(name)
        .bind(description)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create lesson")?;

        Ok(lesson)
    }

    /// Finds a lesson by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Lesson>> {
        let lesson = sqlx::query_as::<_, Lesson>(
            r#"
            SELECT id, module_id, name, description, display_order, created_at, updated_at
            FROM lessons
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find lesson by ID")?;

        Ok(lesson)
    }

    /// Lists all lessons for a module, ordered by display_order
    pub async fn list_by_module(&self, module_id: Uuid) -> Result<Vec<Lesson>> {
        let lessons = sqlx::query_as::<_, Lesson>(
            r#"
            SELECT id, module_id, name, description, display_order, created_at, updated_at
            FROM lessons
            WHERE module_id = $1
            ORDER BY display_order ASC
            "#,
        )
        .bind(module_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list lessons by module")?;

        Ok(lessons)
    }

    /// Lists all lessons for a course (across all modules)
    pub async fn list_by_course(&self, course_id: Uuid) -> Result<Vec<Lesson>> {
        let lessons = sqlx::query_as::<_, Lesson>(
            r#"
            SELECT l.id, l.module_id, l.name, l.description, l.display_order, l.created_at, l.updated_at
            FROM lessons l
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1
            ORDER BY m.display_order ASC, l.display_order ASC
            "#,
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list lessons by course")?;

        Ok(lessons)
    }

    /// Updates a lesson
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Lesson> {
        if let Some(ref n) = name {
            Lesson::validate_name(n).map_err(|e| anyhow!(e))?;
        }

        let lesson = sqlx::query_as::<_, Lesson>(
            r#"
            UPDATE lessons
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, module_id, name, description, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update lesson")?;

        Ok(lesson)
    }

    /// Updates the display order of a lesson
    pub async fn update_display_order(&self, id: Uuid, display_order: i32) -> Result<Lesson> {
        Lesson::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let lesson = sqlx::query_as::<_, Lesson>(
            r#"
            UPDATE lessons
            SET display_order = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, module_id, name, description, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update lesson display order")?;

        Ok(lesson)
    }

    /// Deletes a lesson (will fail if it has resources due to foreign key constraint)
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM lessons WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete lesson")?;

        Ok(result.rows_affected() > 0)
    }

    /// Checks if a lesson has any resources
    pub async fn has_resources(&self, id: Uuid) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM resources WHERE lesson_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if lesson has resources")?;

        Ok(count.0 > 0)
    }

    /// Counts total lessons for a module
    pub async fn count_by_module(&self, module_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM lessons WHERE module_id = $1
            "#,
        )
        .bind(module_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count lessons")?;

        Ok(count.0)
    }
}
