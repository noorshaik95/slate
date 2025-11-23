use crate::models::{ContentType, CopyrightSetting, Resource};
use anyhow::{anyhow, Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing Resource entities
pub struct ResourceRepository {
    pool: PgPool,
}

impl ResourceRepository {
    /// Creates a new ResourceRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new resource
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        lesson_id: Uuid,
        name: String,
        description: Option<String>,
        content_type: ContentType,
        file_size: i64,
        storage_key: String,
        display_order: i32,
    ) -> Result<Resource> {
        Resource::validate_name(&name).map_err(|e| anyhow!(e))?;
        Resource::validate_file_size(file_size).map_err(|e| anyhow!(e))?;
        Resource::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let resource = sqlx::query_as::<_, Resource>(
            r#"
            INSERT INTO resources (
                lesson_id, name, description, content_type, file_size, 
                storage_key, display_order
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(lesson_id)
        .bind(name)
        .bind(description)
        .bind(content_type)
        .bind(file_size)
        .bind(storage_key)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create resource")?;

        Ok(resource)
    }

    /// Finds a resource by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Resource>> {
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            SELECT id, lesson_id, name, description, content_type, file_size, 
                   storage_key, manifest_url, duration_seconds, published, 
                   downloadable, copyright_setting, display_order, created_at, updated_at
            FROM resources
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to find resource by ID")?;

        Ok(resource)
    }

    /// Lists all resources for a lesson, ordered by display_order
    pub async fn list_by_lesson(
        &self,
        lesson_id: Uuid,
        published_only: bool,
    ) -> Result<Vec<Resource>> {
        let query = if published_only {
            r#"
            SELECT id, lesson_id, name, description, content_type, file_size, 
                   storage_key, manifest_url, duration_seconds, published, 
                   downloadable, copyright_setting, display_order, created_at, updated_at
            FROM resources
            WHERE lesson_id = $1 AND published = true
            ORDER BY display_order ASC
            "#
        } else {
            r#"
            SELECT id, lesson_id, name, description, content_type, file_size, 
                   storage_key, manifest_url, duration_seconds, published, 
                   downloadable, copyright_setting, display_order, created_at, updated_at
            FROM resources
            WHERE lesson_id = $1
            ORDER BY display_order ASC
            "#
        };

        let resources = sqlx::query_as::<_, Resource>(query)
            .bind(lesson_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to list resources by lesson")?;

        Ok(resources)
    }

    /// Lists all resources for a module (across all lessons)
    pub async fn list_by_module(
        &self,
        module_id: Uuid,
        published_only: bool,
    ) -> Result<Vec<Resource>> {
        let query = if published_only {
            r#"
            SELECT r.id, r.lesson_id, r.name, r.description, r.content_type, r.file_size, 
                   r.storage_key, r.manifest_url, r.duration_seconds, r.published, 
                   r.downloadable, r.copyright_setting, r.display_order, r.created_at, r.updated_at
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            WHERE l.module_id = $1 AND r.published = true
            ORDER BY l.display_order ASC, r.display_order ASC
            "#
        } else {
            r#"
            SELECT r.id, r.lesson_id, r.name, r.description, r.content_type, r.file_size, 
                   r.storage_key, r.manifest_url, r.duration_seconds, r.published, 
                   r.downloadable, r.copyright_setting, r.display_order, r.created_at, r.updated_at
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            WHERE l.module_id = $1
            ORDER BY l.display_order ASC, r.display_order ASC
            "#
        };

        let resources = sqlx::query_as::<_, Resource>(query)
            .bind(module_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to list resources by module")?;

        Ok(resources)
    }

    /// Lists all resources for a course (across all modules and lessons)
    pub async fn list_by_course(
        &self,
        course_id: Uuid,
        published_only: bool,
    ) -> Result<Vec<Resource>> {
        let query = if published_only {
            r#"
            SELECT r.id, r.lesson_id, r.name, r.description, r.content_type, r.file_size, 
                   r.storage_key, r.manifest_url, r.duration_seconds, r.published, 
                   r.downloadable, r.copyright_setting, r.display_order, r.created_at, r.updated_at
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1 AND r.published = true
            ORDER BY m.display_order ASC, l.display_order ASC, r.display_order ASC
            "#
        } else {
            r#"
            SELECT r.id, r.lesson_id, r.name, r.description, r.content_type, r.file_size, 
                   r.storage_key, r.manifest_url, r.duration_seconds, r.published, 
                   r.downloadable, r.copyright_setting, r.display_order, r.created_at, r.updated_at
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1
            ORDER BY m.display_order ASC, l.display_order ASC, r.display_order ASC
            "#
        };

        let resources = sqlx::query_as::<_, Resource>(query)
            .bind(course_id)
            .fetch_all(&self.pool)
            .await
            .context("Failed to list resources by course")?;

        Ok(resources)
    }

    /// Updates a resource
    pub async fn update(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<Resource> {
        if let Some(ref n) = name {
            Resource::validate_name(n).map_err(|e| anyhow!(e))?;
        }

        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET name = COALESCE($2, name),
                description = COALESCE($3, description),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update resource")?;

        Ok(resource)
    }

    /// Updates video metadata (manifest URL and duration)
    pub async fn update_video_metadata(
        &self,
        id: Uuid,
        manifest_url: String,
        duration_seconds: i32,
    ) -> Result<Resource> {
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET manifest_url = $2,
                duration_seconds = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(manifest_url)
        .bind(duration_seconds)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update video metadata")?;

        Ok(resource)
    }

    /// Updates the display order of a resource
    pub async fn update_display_order(&self, id: Uuid, display_order: i32) -> Result<Resource> {
        Resource::validate_display_order(display_order).map_err(|e| anyhow!(e))?;

        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET display_order = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(display_order)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update resource display order")?;

        Ok(resource)
    }

    /// Updates publication status
    pub async fn update_publication_status(&self, id: Uuid, published: bool) -> Result<Resource> {
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET published = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(published)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update publication status")?;

        Ok(resource)
    }

    /// Updates downloadable flag
    pub async fn update_downloadable(&self, id: Uuid, downloadable: bool) -> Result<Resource> {
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET downloadable = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(downloadable)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update downloadable flag")?;

        Ok(resource)
    }

    /// Updates copyright setting
    pub async fn update_copyright_setting(
        &self,
        id: Uuid,
        copyright_setting: CopyrightSetting,
    ) -> Result<Resource> {
        let resource = sqlx::query_as::<_, Resource>(
            r#"
            UPDATE resources
            SET copyright_setting = $2,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, lesson_id, name, description, content_type, file_size, 
                      storage_key, manifest_url, duration_seconds, published, 
                      downloadable, copyright_setting, display_order, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(copyright_setting)
        .fetch_one(&self.pool)
        .await
        .context("Failed to update copyright setting")?;

        Ok(resource)
    }

    /// Deletes a resource
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM resources WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete resource")?;

        Ok(result.rows_affected() > 0)
    }

    /// Counts total resources for a lesson
    pub async fn count_by_lesson(&self, lesson_id: Uuid, published_only: bool) -> Result<i64> {
        let query = if published_only {
            "SELECT COUNT(*) FROM resources WHERE lesson_id = $1 AND published = true"
        } else {
            "SELECT COUNT(*) FROM resources WHERE lesson_id = $1"
        };

        let count: (i64,) = sqlx::query_as(query)
            .bind(lesson_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to count resources")?;

        Ok(count.0)
    }

    /// Counts total resources for a course
    pub async fn count_by_course(&self, course_id: Uuid, published_only: bool) -> Result<i64> {
        let query = if published_only {
            r#"
            SELECT COUNT(*)
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1 AND r.published = true
            "#
        } else {
            r#"
            SELECT COUNT(*)
            FROM resources r
            INNER JOIN lessons l ON r.lesson_id = l.id
            INNER JOIN modules m ON l.module_id = m.id
            WHERE m.course_id = $1
            "#
        };

        let count: (i64,) = sqlx::query_as(query)
            .bind(course_id)
            .fetch_one(&self.pool)
            .await
            .context("Failed to count resources by course")?;

        Ok(count.0)
    }
}
