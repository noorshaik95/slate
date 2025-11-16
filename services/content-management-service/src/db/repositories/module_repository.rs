use crate::models::Module;
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing Module entities
pub struct ModuleRepository {
    pool: PgPool,
}

impl ModuleRepository {
    /// Creates a new ModuleRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new module
    pub async fn create(
        &self,
        course_id: Uuid,
        name: String,
        description: Option<String>,
        display_order: i32,
        created_by: Uuid,
    ) -> Result<Module> {
        Module::validate_name(&name).map_err(|e| anyhow!(e))?;
        Module::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let module = sqlx::query_as::<_, Module>(
            r#"
            INSERT INTO modules (course_id, name, description, display_order, created_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, course_id, name, description, display_order, created_at, updated_at, created_by
            "#,
        )
        .bind(course_id)
        .bind(name)
        .bind(description)
        .bind(display_order)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create module")?;

        Ok(module)
    }

    /// Finds a module by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Module>> {
        let module = sqlx::query_as::<_, Module>(
            r#"
            SELECT id, course_id, name, description, display_order, created_at, updated_at, created_by
            FROM modules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find module by ID")?;

        Ok(module)
    }

    /// Lists all modules for a course, ordered by display_order
    pub async fn list_by_course(&self, course_id: Uuid) -> Result<Vec<Module>> {
        let modules = sqlx::query_as::<_, Module>(
            r#"
            SELECT id, course_id, name, description, display_order, created_at, updated_at, created_by
            FROM modules
            WHERE course_id = $1
            ORDER BY display_order ASC
            "#,
        )
        .bind(course_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to list modules by course")?;

        Ok(modules)
    }

    /// Updates a module
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Module> {
        if let Some(ref n) = name {
            Module::validate_name(n).map_err(|e| anyhow!(e))?;
        }

        let module = sqlx::query_as::<_, Module>(
            r#"
            UPDATE modules
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, course_id, name, description, display_order, created_at, updated_at, created_by
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update module")?;

        Ok(module)
    }

    /// Updates the display order of a module
    pub async fn update_display_order(&self, id: Uuid, display_order: i32) -> Result<Module> {
        Module::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let module = sqlx::query_as::<_, Module>(
            r#"
            UPDATE modules
            SET display_order = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, course_id, name, description, display_order, created_at, updated_at, created_by
            "#,
        )
        .bind(id)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update module display order")?;

        Ok(module)
    }

    /// Deletes a module (will fail if it has lessons due to foreign key constraint)
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM modules WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete module")?;

        Ok(result.rows_affected() > 0)
    }

    /// Checks if a module has any lessons
    pub async fn has_lessons(&self, id: Uuid) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM lessons WHERE module_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to check if module has lessons")?;

        Ok(count.0 > 0)
    }

    /// Counts total modules for a course
    pub async fn count_by_course(&self, course_id: Uuid) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM modules WHERE course_id = $1
            "#,
        )
        .bind(course_id)
        .fetch_one(&self.pool)
        .await
        .context("Failed to count modules")?;

        Ok(count.0)
    }
}
