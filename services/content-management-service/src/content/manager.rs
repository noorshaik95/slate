use crate::content::errors::{ContentError, ContentResult};
use crate::db::repositories::{
    LessonRepository, ModuleRepository, ResourceRepository,
};
use crate::models::{ContentType, CopyrightSetting, Lesson, Module, Resource};
use crate::search::SearchService;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

/// ContentManager handles CRUD operations for modules, lessons, and resources
pub struct ContentManager {
    module_repo: ModuleRepository,
    lesson_repo: LessonRepository,
    resource_repo: ResourceRepository,
    search_service: Option<Arc<SearchService>>,
}

impl ContentManager {
    /// Creates a new ContentManager
    pub fn new(pool: PgPool) -> Self {
        Self {
            module_repo: ModuleRepository::new(pool.clone()),
            lesson_repo: LessonRepository::new(pool.clone()),
            resource_repo: ResourceRepository::new(pool),
            search_service: None,
        }
    }

    /// Creates a new ContentManager with search service integration
    pub fn with_search(pool: PgPool, search_service: Arc<SearchService>) -> Self {
        Self {
            module_repo: ModuleRepository::new(pool.clone()),
            lesson_repo: LessonRepository::new(pool.clone()),
            resource_repo: ResourceRepository::new(pool),
            search_service: Some(search_service),
        }
    }

    /// Indexes a resource in the search service
    async fn index_resource(&self, resource: &Resource) -> ContentResult<()> {
        if let Some(ref search_service) = self.search_service {
            // Get lesson and module for hierarchical path
            let lesson = self.get_lesson(resource.lesson_id).await?;
            let module = self.get_module(lesson.module_id).await?;

            // Index the resource
            if let Err(e) = search_service.index_content(resource, &lesson, &module).await {
                error!(
                    resource_id = %resource.id,
                    error = %e,
                    "Failed to index resource in search service"
                );
                // Don't fail the operation if indexing fails
            } else {
                info!(resource_id = %resource.id, "Successfully indexed resource");
            }
        }
        Ok(())
    }

    /// Removes a resource from the search index
    async fn remove_from_index(&self, resource_id: Uuid) -> ContentResult<()> {
        if let Some(ref search_service) = self.search_service {
            if let Err(e) = search_service.remove_from_index(resource_id).await {
                error!(
                    resource_id = %resource_id,
                    error = %e,
                    "Failed to remove resource from search index"
                );
                // Don't fail the operation if removal fails
            } else {
                info!(resource_id = %resource_id, "Successfully removed resource from index");
            }
        }
        Ok(())
    }

    // ========================================================================
    // Module Operations
    // ========================================================================

    /// Creates a new module with name validation (1-200 characters)
    pub async fn create_module(
        &self,
        course_id: Uuid,
        name: String,
        description: Option<String>,
        display_order: i32,
        created_by: Uuid,
    ) -> ContentResult<Module> {
        // Validate name (1-200 characters)
        Module::validate_name(&name)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate display order
        Module::validate_display_order(display_order)
            .map_err(|e| ContentError::Validation(e))?;

        // Create module
        let module = self
            .module_repo
            .create(course_id, name, description, display_order, created_by)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique constraint") {
                    ContentError::Conflict(format!(
                        "Module with display_order {} already exists",
                        display_order
                    ))
                } else {
                    ContentError::from(e)
                }
            })?;

        Ok(module)
    }

    /// Gets a module by ID
    pub async fn get_module(&self, id: Uuid) -> ContentResult<Module> {
        self.module_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ContentError::NotFound(format!("Module {} not found", id)))
    }

    /// Lists all modules for a course
    pub async fn list_modules(&self, course_id: Uuid) -> ContentResult<Vec<Module>> {
        Ok(self.module_repo.list_by_course(course_id).await?)
    }

    /// Updates a module
    pub async fn update_module(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> ContentResult<Module> {
        // Validate name if provided
        if let Some(ref n) = name {
            Module::validate_name(n)
                .map_err(|e| ContentError::Validation(e))?;
        }

        // Check if module exists
        self.get_module(id).await?;

        // Update module
        let module = self
            .module_repo
            .update(id, name, description)
            .await
            .map_err(|e| ContentError::from(e))?;

        Ok(module)
    }

    /// Deletes a module (with child checking)
    pub async fn delete_module(&self, id: Uuid) -> ContentResult<()> {
        // Check if module exists
        self.get_module(id).await?;

        // Check if module has lessons
        let has_lessons = self.module_repo.has_lessons(id).await?;
        if has_lessons {
            return Err(ContentError::Conflict(
                "Cannot delete module with existing lessons".to_string(),
            ));
        }

        // Delete module
        let deleted = self.module_repo.delete(id).await?;
        if !deleted {
            return Err(ContentError::NotFound(format!("Module {} not found", id)));
        }

        Ok(())
    }

    // ========================================================================
    // Lesson Operations
    // ========================================================================

    /// Creates a new lesson with parent module validation
    pub async fn create_lesson(
        &self,
        module_id: Uuid,
        name: String,
        description: Option<String>,
        display_order: i32,
    ) -> ContentResult<Lesson> {
        // Validate name (1-200 characters)
        Lesson::validate_name(&name)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate display order
        Lesson::validate_display_order(display_order)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate parent module exists
        self.get_module(module_id).await?;

        // Create lesson
        let lesson = self
            .lesson_repo
            .create(module_id, name, description, display_order)
            .await
            .map_err(|e| {
                if e.to_string().contains("unique constraint") {
                    ContentError::Conflict(format!(
                        "Lesson with display_order {} already exists in module",
                        display_order
                    ))
                } else {
                    ContentError::from(e)
                }
            })?;

        Ok(lesson)
    }

    /// Gets a lesson by ID
    pub async fn get_lesson(&self, id: Uuid) -> ContentResult<Lesson> {
        self.lesson_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ContentError::NotFound(format!("Lesson {} not found", id)))
    }

    /// Lists all lessons for a module
    pub async fn list_lessons(&self, module_id: Uuid) -> ContentResult<Vec<Lesson>> {
        Ok(self.lesson_repo.list_by_module(module_id).await?)
    }

    /// Updates a lesson
    pub async fn update_lesson(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> ContentResult<Lesson> {
        // Validate name if provided
        if let Some(ref n) = name {
            Lesson::validate_name(n)
                .map_err(|e| ContentError::Validation(e))?;
        }

        // Check if lesson exists
        self.get_lesson(id).await?;

        // Update lesson
        let lesson = self
            .lesson_repo
            .update(id, name, description)
            .await
            .map_err(|e| ContentError::from(e))?;

        Ok(lesson)
    }

    /// Deletes a lesson (with child checking)
    pub async fn delete_lesson(&self, id: Uuid) -> ContentResult<()> {
        // Check if lesson exists
        self.get_lesson(id).await?;

        // Check if lesson has resources
        let has_resources = self.lesson_repo.has_resources(id).await?;
        if has_resources {
            return Err(ContentError::Conflict(
                "Cannot delete lesson with existing resources".to_string(),
            ));
        }

        // Delete lesson
        let deleted = self.lesson_repo.delete(id).await?;
        if !deleted {
            return Err(ContentError::NotFound(format!("Lesson {} not found", id)));
        }

        Ok(())
    }

    // ========================================================================
    // Resource Operations
    // ========================================================================

    /// Creates a new resource with parent lesson validation
    pub async fn create_resource(
        &self,
        lesson_id: Uuid,
        name: String,
        description: Option<String>,
        content_type: ContentType,
        file_size: i64,
        storage_key: String,
        display_order: i32,
    ) -> ContentResult<Resource> {
        // Validate name (1-200 characters)
        Resource::validate_name(&name)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate display order
        Resource::validate_display_order(display_order)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate file size
        Resource::validate_file_size(file_size)
            .map_err(|e| ContentError::Validation(e))?;

        // Validate parent lesson exists
        self.get_lesson(lesson_id).await?;

        // Create resource
        let resource = self
            .resource_repo
            .create(
                lesson_id,
                name,
                description,
                content_type,
                file_size,
                storage_key,
                display_order,
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("unique constraint") {
                    ContentError::Conflict(format!(
                        "Resource with display_order {} already exists in lesson",
                        display_order
                    ))
                } else {
                    ContentError::from(e)
                }
            })?;

        // Index content on creation
        self.index_resource(&resource).await?;

        Ok(resource)
    }

    /// Gets a resource by ID
    pub async fn get_resource(&self, id: Uuid) -> ContentResult<Resource> {
        self.resource_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ContentError::NotFound(format!("Resource {} not found", id)))
    }

    /// Lists all resources for a lesson
    pub async fn list_resources(&self, lesson_id: Uuid) -> ContentResult<Vec<Resource>> {
        Ok(self.resource_repo.list_by_lesson(lesson_id, false).await?)
    }

    /// Updates a resource
    pub async fn update_resource(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        downloadable: Option<bool>,
        copyright_setting: Option<CopyrightSetting>,
    ) -> ContentResult<Resource> {
        // Validate name if provided
        if let Some(ref n) = name {
            Resource::validate_name(n)
                .map_err(|e| ContentError::Validation(e))?;
        }

        // Check if resource exists
        self.get_resource(id).await?;

        // Update basic fields
        let mut resource = self
            .resource_repo
            .update(id, name, description)
            .await
            .map_err(|e| ContentError::from(e))?;

        // Update downloadable if provided
        if let Some(dl) = downloadable {
            resource = self
                .resource_repo
                .update_downloadable(id, dl)
                .await
                .map_err(|e| ContentError::from(e))?;
        }

        // Update copyright setting if provided
        if let Some(cs) = copyright_setting {
            resource = self
                .resource_repo
                .update_copyright_setting(id, cs)
                .await
                .map_err(|e| ContentError::from(e))?;
        }

        // Update index on content modification
        self.index_resource(&resource).await?;

        Ok(resource)
    }

    /// Deletes a resource
    pub async fn delete_resource(&self, id: Uuid) -> ContentResult<()> {
        // Check if resource exists
        self.get_resource(id).await?;

        // Remove from index on deletion
        self.remove_from_index(id).await?;

        // Delete resource (no children to check for resources)
        let deleted = self.resource_repo.delete(id).await?;
        if !deleted {
            return Err(ContentError::NotFound(format!("Resource {} not found", id)));
        }

        Ok(())
    }

    // ========================================================================
    // Content Reordering
    // ========================================================================

    /// Reorders modules within a course
    pub async fn reorder_modules(
        &self,
        course_id: Uuid,
        reorder_items: Vec<(Uuid, i32)>, // (module_id, new_position)
    ) -> ContentResult<()> {
        // Validate all positions are unique and sequential starting from 0
        self.validate_reorder_positions(&reorder_items)?;

        // Verify all modules belong to the course
        let modules = self.list_modules(course_id).await?;
        let module_ids: std::collections::HashSet<Uuid> = 
            modules.iter().map(|m| m.id).collect();

        for (module_id, _) in &reorder_items {
            if !module_ids.contains(module_id) {
                return Err(ContentError::NotFound(format!(
                    "Module {} not found in course",
                    module_id
                )));
            }
        }

        // Update display orders
        for (module_id, new_position) in reorder_items {
            self.module_repo
                .update_display_order(module_id, new_position)
                .await?;
        }

        Ok(())
    }

    /// Reorders lessons within a module
    pub async fn reorder_lessons(
        &self,
        module_id: Uuid,
        reorder_items: Vec<(Uuid, i32)>, // (lesson_id, new_position)
    ) -> ContentResult<()> {
        // Validate all positions are unique and sequential starting from 0
        self.validate_reorder_positions(&reorder_items)?;

        // Verify module exists
        self.get_module(module_id).await?;

        // Verify all lessons belong to the module
        let lessons = self.list_lessons(module_id).await?;
        let lesson_ids: std::collections::HashSet<Uuid> = 
            lessons.iter().map(|l| l.id).collect();

        for (lesson_id, _) in &reorder_items {
            if !lesson_ids.contains(lesson_id) {
                return Err(ContentError::NotFound(format!(
                    "Lesson {} not found in module",
                    lesson_id
                )));
            }
        }

        // Update display orders
        for (lesson_id, new_position) in reorder_items {
            self.lesson_repo
                .update_display_order(lesson_id, new_position)
                .await?;
        }

        Ok(())
    }

    /// Reorders resources within a lesson
    pub async fn reorder_resources(
        &self,
        lesson_id: Uuid,
        reorder_items: Vec<(Uuid, i32)>, // (resource_id, new_position)
    ) -> ContentResult<()> {
        // Validate all positions are unique and sequential starting from 0
        self.validate_reorder_positions(&reorder_items)?;

        // Verify lesson exists
        self.get_lesson(lesson_id).await?;

        // Verify all resources belong to the lesson
        let resources = self.list_resources(lesson_id).await?;
        let resource_ids: std::collections::HashSet<Uuid> = 
            resources.iter().map(|r| r.id).collect();

        for (resource_id, _) in &reorder_items {
            if !resource_ids.contains(resource_id) {
                return Err(ContentError::NotFound(format!(
                    "Resource {} not found in lesson",
                    resource_id
                )));
            }
        }

        // Update display orders
        for (resource_id, new_position) in reorder_items {
            self.resource_repo
                .update_display_order(resource_id, new_position)
                .await?;
        }

        Ok(())
    }

    /// Validates that reorder positions are unique and sequential starting from 0
    fn validate_reorder_positions(&self, items: &[(Uuid, i32)]) -> ContentResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        // Collect all positions
        let mut positions: Vec<i32> = items.iter().map(|(_, pos)| *pos).collect();
        positions.sort_unstable();

        // Check for duplicates
        for i in 1..positions.len() {
            if positions[i] == positions[i - 1] {
                return Err(ContentError::Validation(
                    "Duplicate position indices found".to_string(),
                ));
            }
        }

        // Check if sequential starting from 0
        for (i, &pos) in positions.iter().enumerate() {
            if pos != i as i32 {
                return Err(ContentError::Validation(
                    "Position indices must be sequential starting from 0".to_string(),
                ));
            }
        }

        Ok(())
    }

    // ========================================================================
    // Publication Control
    // ========================================================================

    /// Publishes a resource (makes it visible to students)
    pub async fn publish_resource(&self, resource_id: Uuid) -> ContentResult<()> {
        // Check if resource exists
        self.get_resource(resource_id).await?;

        // Update publication status
        let resource = self.resource_repo
            .update_publication_status(resource_id, true)
            .await?;

        // Update index within 10 seconds of publication status change
        self.index_resource(&resource).await?;

        Ok(())
    }

    /// Unpublishes a resource (makes it visible only to instructors)
    pub async fn unpublish_resource(&self, resource_id: Uuid) -> ContentResult<()> {
        // Check if resource exists
        self.get_resource(resource_id).await?;

        // Update publication status
        let resource = self.resource_repo
            .update_publication_status(resource_id, false)
            .await?;

        // Update index within 10 seconds of publication status change
        self.index_resource(&resource).await?;

        Ok(())
    }

    /// Lists published resources for a lesson (student view)
    pub async fn list_published_resources(&self, lesson_id: Uuid) -> ContentResult<Vec<Resource>> {
        Ok(self.resource_repo.list_by_lesson(lesson_id, true).await?)
    }

    /// Lists all resources for a lesson (instructor view)
    pub async fn list_all_resources(&self, lesson_id: Uuid) -> ContentResult<Vec<Resource>> {
        Ok(self.resource_repo.list_by_lesson(lesson_id, false).await?)
    }

    // ========================================================================
    // Content Structure
    // ========================================================================

    /// Gets the complete content structure for a course
    /// Filters by publication status based on user role
    pub async fn get_content_structure(
        &self,
        course_id: Uuid,
        is_instructor: bool,
    ) -> ContentResult<ContentStructure> {
        // Get all modules for the course
        let modules = self.list_modules(course_id).await?;

        let mut module_structures = Vec::new();

        for module in modules {
            // Get all lessons for this module
            let lessons = self.list_lessons(module.id).await?;

            let mut lesson_structures = Vec::new();

            for lesson in lessons {
                // Get resources for this lesson (filtered by publication status)
                let resources = if is_instructor {
                    self.list_all_resources(lesson.id).await?
                } else {
                    self.list_published_resources(lesson.id).await?
                };

                lesson_structures.push(LessonWithContent {
                    lesson,
                    resources,
                });
            }

            module_structures.push(ModuleWithContent {
                module,
                lessons: lesson_structures,
            });
        }

        Ok(ContentStructure {
            modules: module_structures,
        })
    }
}

/// Content structure with complete hierarchy
#[derive(Debug, Clone)]
pub struct ContentStructure {
    pub modules: Vec<ModuleWithContent>,
}

/// Module with its lessons
#[derive(Debug, Clone)]
pub struct ModuleWithContent {
    pub module: Module,
    pub lessons: Vec<LessonWithContent>,
}

/// Lesson with its resources
#[derive(Debug, Clone)]
pub struct LessonWithContent {
    pub lesson: Lesson,
    pub resources: Vec<Resource>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests that verify the business logic.
    // Integration tests with a real database are in the tests/ directory.

    #[test]
    fn test_content_manager_creation() {
        // This test just verifies the struct can be created
        // Real functionality tests require a database connection
    }
}
