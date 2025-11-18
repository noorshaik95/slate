use crate::content::{ContentManager, ContentError};
use crate::db::DatabasePool;
use crate::models::ContentType;
use crate::proto::content::{
    content_service_server::ContentService,
    // Module messages
    Module, CreateModuleRequest, UpdateModuleRequest, DeleteModuleRequest,
    ListModulesRequest, ListModulesResponse, GetModuleRequest,
    // Lesson messages
    Lesson, CreateLessonRequest, UpdateLessonRequest, DeleteLessonRequest,
    ListLessonsRequest, ListLessonsResponse, GetLessonRequest,
    // Resource messages
    Resource, CreateResourceRequest, UpdateResourceRequest, DeleteResourceRequest,
    ListResourcesRequest, ListResourcesResponse, GetResourceRequest,
    // Content structure messages
    GetContentStructureRequest, ContentStructure, ModuleWithContent, LessonWithContent,
    // Reorder and publication messages
    ReorderContentRequest, PublishContentRequest, UnpublishContentRequest,
    // Health check and connectivity test messages
    HealthRequest, HealthResponse, PingRequest, PingResponse,
};
use std::sync::Arc;
use std::collections::HashMap;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// ContentService gRPC implementation
pub struct ContentServiceImpl {
    content_manager: Arc<ContentManager>,
    db_pool: Arc<DatabasePool>,
}

impl ContentServiceImpl {
    pub fn new(content_manager: Arc<ContentManager>, db_pool: Arc<DatabasePool>) -> Self {
        Self { content_manager, db_pool }
    }

    /// Extracts user ID from request metadata
    fn get_user_id(&self, request: &Request<impl std::fmt::Debug>) -> Result<Uuid, Status> {
        let metadata = request.metadata();
        let user_id_str = metadata
            .get("user-id")
            .ok_or_else(|| Status::unauthenticated("Missing user-id in metadata"))?
            .to_str()
            .map_err(|_| Status::invalid_argument("Invalid user-id format"))?;

        Uuid::parse_str(user_id_str)
            .map_err(|_| Status::invalid_argument("Invalid user-id UUID"))
    }

    /// Checks if user is an instructor
    fn is_instructor(&self, request: &Request<impl std::fmt::Debug>) -> bool {
        let metadata = request.metadata();
        if let Some(role) = metadata.get("user-role") {
            if let Ok(role_str) = role.to_str() {
                return role_str == "instructor" || role_str == "admin";
            }
        }
        false
    }
}

#[tonic::async_trait]
impl ContentService for ContentServiceImpl {
    // ========================================================================
    // Health check and connectivity test
    // ========================================================================

    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        // Check database health
        let db_status = match self.db_pool.health_check().await {
            Ok(_) => "healthy",
            Err(_) => "unhealthy",
        };

        let mut checks = HashMap::new();
        checks.insert("database".to_string(), db_status.to_string());

        Ok(Response::new(HealthResponse {
            status: db_status.to_string(),
            service: "content-management-service".to_string(),
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            checks,
        }))
    }

    async fn ping(
        &self,
        request: Request<PingRequest>,
    ) -> Result<Response<PingResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(PingResponse {
            message: if req.message.is_empty() {
                "pong".to_string()
            } else {
                req.message
            },
            service: "content-management-service".to_string(),
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        }))
    }

    // ========================================================================
    // Module Operations
    // ========================================================================

    async fn create_module(
        &self,
        request: Request<CreateModuleRequest>,
    ) -> Result<Response<Module>, Status> {
        let user_id = self.get_user_id(&request)?;
        let req = request.into_inner();

        let course_id = Uuid::parse_str(&req.course_id)
            .map_err(|_| Status::invalid_argument("Invalid course_id"))?;

        let module = self
            .content_manager
            .create_module(
                course_id,
                req.name,
                if req.description.is_empty() { None } else { Some(req.description) },
                req.display_order,
                user_id,
            )
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(module_to_proto(module)))
    }

    async fn update_module(
        &self,
        request: Request<UpdateModuleRequest>,
    ) -> Result<Response<Module>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid module id"))?;

        let name = if req.name.is_empty() { None } else { Some(req.name) };
        let description = if req.description.is_empty() { None } else { Some(req.description) };

        let module = self
            .content_manager
            .update_module(id, name, description)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(module_to_proto(module)))
    }

    async fn delete_module(
        &self,
        request: Request<DeleteModuleRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid module id"))?;

        self.content_manager
            .delete_module(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(()))
    }

    async fn list_modules(
        &self,
        request: Request<ListModulesRequest>,
    ) -> Result<Response<ListModulesResponse>, Status> {
        let req = request.into_inner();

        let course_id = Uuid::parse_str(&req.course_id)
            .map_err(|_| Status::invalid_argument("Invalid course_id"))?;

        let modules = self
            .content_manager
            .list_modules(course_id)
            .await
            .map_err(map_content_error)?;

        let proto_modules = modules.into_iter().map(module_to_proto).collect();

        Ok(Response::new(ListModulesResponse {
            modules: proto_modules,
        }))
    }

    async fn get_module(
        &self,
        request: Request<GetModuleRequest>,
    ) -> Result<Response<Module>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid module id"))?;

        let module = self
            .content_manager
            .get_module(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(module_to_proto(module)))
    }

    // ========================================================================
    // Lesson Operations
    // ========================================================================

    async fn create_lesson(
        &self,
        request: Request<CreateLessonRequest>,
    ) -> Result<Response<Lesson>, Status> {
        let req = request.into_inner();

        let module_id = Uuid::parse_str(&req.module_id)
            .map_err(|_| Status::invalid_argument("Invalid module_id"))?;

        let lesson = self
            .content_manager
            .create_lesson(
                module_id,
                req.name,
                if req.description.is_empty() { None } else { Some(req.description) },
                req.display_order,
            )
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(lesson_to_proto(lesson)))
    }

    async fn update_lesson(
        &self,
        request: Request<UpdateLessonRequest>,
    ) -> Result<Response<Lesson>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid lesson id"))?;

        let name = if req.name.is_empty() { None } else { Some(req.name) };
        let description = if req.description.is_empty() { None } else { Some(req.description) };

        let lesson = self
            .content_manager
            .update_lesson(id, name, description)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(lesson_to_proto(lesson)))
    }

    async fn delete_lesson(
        &self,
        request: Request<DeleteLessonRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid lesson id"))?;

        self.content_manager
            .delete_lesson(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(()))
    }

    async fn list_lessons(
        &self,
        request: Request<ListLessonsRequest>,
    ) -> Result<Response<ListLessonsResponse>, Status> {
        let req = request.into_inner();

        let module_id = Uuid::parse_str(&req.module_id)
            .map_err(|_| Status::invalid_argument("Invalid module_id"))?;

        let lessons = self
            .content_manager
            .list_lessons(module_id)
            .await
            .map_err(map_content_error)?;

        let proto_lessons = lessons.into_iter().map(lesson_to_proto).collect();

        Ok(Response::new(ListLessonsResponse {
            lessons: proto_lessons,
        }))
    }

    async fn get_lesson(
        &self,
        request: Request<GetLessonRequest>,
    ) -> Result<Response<Lesson>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid lesson id"))?;

        let lesson = self
            .content_manager
            .get_lesson(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(lesson_to_proto(lesson)))
    }

    // ========================================================================
    // Resource Operations
    // ========================================================================

    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        let req = request.into_inner();

        let lesson_id = Uuid::parse_str(&req.lesson_id)
            .map_err(|_| Status::invalid_argument("Invalid lesson_id"))?;

        let content_type: ContentType = req.content_type.parse()
            .map_err(|e: String| Status::invalid_argument(e))?;

        let resource = self
            .content_manager
            .create_resource(
                lesson_id,
                req.name,
                if req.description.is_empty() { None } else { Some(req.description) },
                content_type,
                req.file_size,
                req.storage_key,
                req.display_order,
            )
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(resource_to_proto(resource)))
    }

    async fn update_resource(
        &self,
        request: Request<UpdateResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid resource id"))?;

        let name = if req.name.is_empty() { None } else { Some(req.name) };
        let description = if req.description.is_empty() { None } else { Some(req.description) };
        let downloadable = Some(req.downloadable);
        let copyright_setting = if req.copyright_setting.is_empty() {
            None
        } else {
            Some(req.copyright_setting.parse()
                .map_err(|e: String| Status::invalid_argument(e))?)
        };

        let resource = self
            .content_manager
            .update_resource(id, name, description, downloadable, copyright_setting)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(resource_to_proto(resource)))
    }

    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid resource id"))?;

        self.content_manager
            .delete_resource(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(()))
    }

    async fn list_resources(
        &self,
        request: Request<ListResourcesRequest>,
    ) -> Result<Response<ListResourcesResponse>, Status> {
        let req = request.into_inner();

        let lesson_id = Uuid::parse_str(&req.lesson_id)
            .map_err(|_| Status::invalid_argument("Invalid lesson_id"))?;

        let resources = self
            .content_manager
            .list_resources(lesson_id)
            .await
            .map_err(map_content_error)?;

        let proto_resources = resources.into_iter().map(resource_to_proto).collect();

        Ok(Response::new(ListResourcesResponse {
            resources: proto_resources,
        }))
    }

    async fn get_resource(
        &self,
        request: Request<GetResourceRequest>,
    ) -> Result<Response<Resource>, Status> {
        let req = request.into_inner();

        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid resource id"))?;

        let resource = self
            .content_manager
            .get_resource(id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(resource_to_proto(resource)))
    }

    // ========================================================================
    // Content Structure
    // ========================================================================

    async fn get_content_structure(
        &self,
        request: Request<GetContentStructureRequest>,
    ) -> Result<Response<ContentStructure>, Status> {
        let is_instructor = self.is_instructor(&request);
        let req = request.into_inner();

        let course_id = Uuid::parse_str(&req.course_id)
            .map_err(|_| Status::invalid_argument("Invalid course_id"))?;

        let structure = self
            .content_manager
            .get_content_structure(course_id, is_instructor)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(content_structure_to_proto(structure)))
    }

    // ========================================================================
    // Reordering
    // ========================================================================

    async fn reorder_content(
        &self,
        request: Request<ReorderContentRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let reorder_items: Result<Vec<(Uuid, i32)>, Status> = req
            .items
            .into_iter()
            .map(|item| {
                let id = Uuid::parse_str(&item.id)
                    .map_err(|_| Status::invalid_argument("Invalid item id"))?;
                Ok((id, item.new_position))
            })
            .collect();

        let reorder_items = reorder_items?;

        match req.content_type.as_str() {
            "module" => {
                // For modules, we need the course_id from the first module
                if let Some((first_id, _)) = reorder_items.first() {
                    let module = self.content_manager.get_module(*first_id).await
                        .map_err(map_content_error)?;
                    self.content_manager
                        .reorder_modules(module.course_id, reorder_items)
                        .await
                        .map_err(map_content_error)?;
                }
            }
            "lesson" => {
                // For lessons, we need the module_id from the first lesson
                if let Some((first_id, _)) = reorder_items.first() {
                    let lesson = self.content_manager.get_lesson(*first_id).await
                        .map_err(map_content_error)?;
                    self.content_manager
                        .reorder_lessons(lesson.module_id, reorder_items)
                        .await
                        .map_err(map_content_error)?;
                }
            }
            "resource" => {
                // For resources, we need the lesson_id from the first resource
                if let Some((first_id, _)) = reorder_items.first() {
                    let resource = self.content_manager.get_resource(*first_id).await
                        .map_err(map_content_error)?;
                    self.content_manager
                        .reorder_resources(resource.lesson_id, reorder_items)
                        .await
                        .map_err(map_content_error)?;
                }
            }
            _ => return Err(Status::invalid_argument("Invalid content_type")),
        }

        Ok(Response::new(()))
    }

    // ========================================================================
    // Publication Control
    // ========================================================================

    async fn publish_content(
        &self,
        request: Request<PublishContentRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let resource_id = Uuid::parse_str(&req.resource_id)
            .map_err(|_| Status::invalid_argument("Invalid resource_id"))?;

        self.content_manager
            .publish_resource(resource_id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(()))
    }

    async fn unpublish_content(
        &self,
        request: Request<UnpublishContentRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        let resource_id = Uuid::parse_str(&req.resource_id)
            .map_err(|_| Status::invalid_argument("Invalid resource_id"))?;

        self.content_manager
            .unpublish_resource(resource_id)
            .await
            .map_err(map_content_error)?;

        Ok(Response::new(()))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Maps ContentError to gRPC Status
fn map_content_error(err: ContentError) -> Status {
    match err {
        ContentError::Validation(msg) => Status::invalid_argument(msg),
        ContentError::NotFound(msg) => Status::not_found(msg),
        ContentError::Conflict(msg) => Status::already_exists(msg),
        ContentError::Authorization(msg) => Status::permission_denied(msg),
        ContentError::Database(err) => Status::internal(format!("Database error: {}", err)),
        ContentError::Internal(msg) => Status::internal(msg),
    }
}

/// Converts domain Module to protobuf Module
fn module_to_proto(module: crate::models::Module) -> Module {
    Module {
        id: module.id.to_string(),
        course_id: module.course_id.to_string(),
        name: module.name,
        description: module.description.unwrap_or_default(),
        display_order: module.display_order,
        created_at: Some(prost_types::Timestamp {
            seconds: module.created_at.timestamp(),
            nanos: module.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: module.updated_at.timestamp(),
            nanos: module.updated_at.timestamp_subsec_nanos() as i32,
        }),
        created_by: module.created_by.to_string(),
    }
}

/// Converts domain Lesson to protobuf Lesson
fn lesson_to_proto(lesson: crate::models::Lesson) -> Lesson {
    Lesson {
        id: lesson.id.to_string(),
        module_id: lesson.module_id.to_string(),
        name: lesson.name,
        description: lesson.description.unwrap_or_default(),
        display_order: lesson.display_order,
        created_at: Some(prost_types::Timestamp {
            seconds: lesson.created_at.timestamp(),
            nanos: lesson.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: lesson.updated_at.timestamp(),
            nanos: lesson.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

/// Converts domain Resource to protobuf Resource
fn resource_to_proto(resource: crate::models::Resource) -> Resource {
    Resource {
        id: resource.id.to_string(),
        lesson_id: resource.lesson_id.to_string(),
        name: resource.name,
        description: resource.description.unwrap_or_default(),
        content_type: resource.content_type.to_string(),
        file_size: resource.file_size,
        storage_key: resource.storage_key,
        manifest_url: resource.manifest_url.unwrap_or_default(),
        duration_seconds: resource.duration_seconds.unwrap_or(0),
        published: resource.published,
        downloadable: resource.downloadable,
        copyright_setting: resource.copyright_setting.to_string(),
        display_order: resource.display_order,
        created_at: Some(prost_types::Timestamp {
            seconds: resource.created_at.timestamp(),
            nanos: resource.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: resource.updated_at.timestamp(),
            nanos: resource.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

/// Converts domain ContentStructure to protobuf ContentStructure
fn content_structure_to_proto(structure: crate::content::ContentStructure) -> ContentStructure {
    ContentStructure {
        modules: structure
            .modules
            .into_iter()
            .map(module_with_content_to_proto)
            .collect(),
    }
}

/// Converts domain ModuleWithContent to protobuf ModuleWithContent
fn module_with_content_to_proto(module_content: crate::content::ModuleWithContent) -> ModuleWithContent {
    ModuleWithContent {
        module: Some(module_to_proto(module_content.module)),
        lessons: module_content
            .lessons
            .into_iter()
            .map(lesson_with_content_to_proto)
            .collect(),
    }
}

/// Converts domain LessonWithContent to protobuf LessonWithContent
fn lesson_with_content_to_proto(lesson_content: crate::content::LessonWithContent) -> LessonWithContent {
    LessonWithContent {
        lesson: Some(lesson_to_proto(lesson_content.lesson)),
        resources: lesson_content
            .resources
            .into_iter()
            .map(resource_to_proto)
            .collect(),
    }
}
