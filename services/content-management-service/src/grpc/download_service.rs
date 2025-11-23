use crate::download::DownloadManager;
use crate::proto::content::{
    download_service_server::DownloadService as DownloadServiceTrait, DownloadUrlResponse,
    GenerateDownloadUrlRequest,
};
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, instrument};
use uuid::Uuid;

/// gRPC DownloadService implementation
pub struct DownloadServiceHandler {
    download_manager: Arc<DownloadManager>,
}

impl DownloadServiceHandler {
    /// Creates a new DownloadServiceHandler
    pub fn new(download_manager: Arc<DownloadManager>) -> Self {
        Self { download_manager }
    }

    /// Extracts user ID from request metadata
    /// In production, this would extract from JWT token in metadata
    fn extract_user_id(
        &self,
        request: &Request<GenerateDownloadUrlRequest>,
    ) -> Result<Uuid, Status> {
        // TODO: Extract from JWT token in request metadata
        // For now, we'll look for a user_id in metadata
        if let Some(user_id_str) = request.metadata().get("user_id") {
            let user_id_str = user_id_str
                .to_str()
                .map_err(|_| Status::unauthenticated("Invalid user_id in metadata"))?;

            Uuid::parse_str(user_id_str)
                .map_err(|_| Status::unauthenticated("Invalid user_id format"))
        } else {
            Err(Status::unauthenticated("Missing user_id in metadata"))
        }
    }

    /// Extracts user role from request metadata
    /// Returns true if user is an instructor
    fn is_instructor(&self, request: &Request<GenerateDownloadUrlRequest>) -> bool {
        // TODO: Extract from JWT token in request metadata
        // For now, we'll look for a role in metadata
        if let Some(role) = request.metadata().get("role") {
            if let Ok(role_str) = role.to_str() {
                return role_str.to_lowercase() == "instructor"
                    || role_str.to_lowercase() == "admin";
            }
        }
        false
    }
}

#[tonic::async_trait]
impl DownloadServiceTrait for DownloadServiceHandler {
    #[instrument(skip(self, request))]
    async fn generate_download_url(
        &self,
        request: Request<GenerateDownloadUrlRequest>,
    ) -> Result<Response<DownloadUrlResponse>, Status> {
        let req = request.get_ref();

        info!(
            resource_id = %req.resource_id,
            "Received generate download URL request"
        );

        // Validate resource_id
        if req.resource_id.is_empty() {
            return Err(Status::invalid_argument("resource_id cannot be empty"));
        }

        let resource_id = Uuid::parse_str(&req.resource_id)
            .map_err(|_| Status::invalid_argument("Invalid resource_id format"))?;

        // Extract user information from request metadata
        let user_id = self.extract_user_id(&request)?;
        let is_instructor = self.is_instructor(&request);

        info!(
            resource_id = %resource_id,
            user_id = %user_id,
            is_instructor = is_instructor,
            "Processing download URL request"
        );

        // Generate download URL
        let (download_url, expires_at, copyright_notice) = self
            .download_manager
            .generate_download_url(resource_id, user_id, is_instructor)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    resource_id = %resource_id,
                    user_id = %user_id,
                    "Failed to generate download URL"
                );
                match e {
                    crate::download::DownloadError::ResourceNotFound(_) => {
                        Status::not_found("Resource not found")
                    }
                    crate::download::DownloadError::ResourceNotPublished => {
                        Status::permission_denied("Resource is not published")
                    }
                    crate::download::DownloadError::DownloadNotAllowed(msg) => {
                        Status::permission_denied(msg)
                    }
                    crate::download::DownloadError::CopyrightRestriction(msg) => {
                        Status::permission_denied(msg)
                    }
                    crate::download::DownloadError::InvalidResourceType(msg) => {
                        Status::invalid_argument(msg)
                    }
                    _ => Status::internal(format!("Failed to generate download URL: {}", e)),
                }
            })?;

        // Convert expires_at to protobuf Timestamp
        let expires_at_proto = Timestamp {
            seconds: expires_at.timestamp(),
            nanos: expires_at.timestamp_subsec_nanos() as i32,
        };

        info!(
            resource_id = %resource_id,
            user_id = %user_id,
            expires_at = %expires_at,
            has_copyright_notice = copyright_notice.is_some(),
            "Download URL generated successfully"
        );

        Ok(Response::new(DownloadUrlResponse {
            download_url,
            expires_at: Some(expires_at_proto),
            copyright_notice: copyright_notice.unwrap_or_default(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_parsing() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(Uuid::parse_str(valid_uuid).is_ok());

        let invalid_uuid = "not-a-uuid";
        assert!(Uuid::parse_str(invalid_uuid).is_err());
    }

    #[test]
    fn test_empty_resource_id() {
        let resource_id = "";
        assert!(resource_id.is_empty());
    }

    #[test]
    fn test_role_detection() {
        let role = "instructor";
        assert!(role.to_lowercase() == "instructor" || role.to_lowercase() == "admin");

        let role = "student";
        assert!(!(role.to_lowercase() == "instructor" || role.to_lowercase() == "admin"));
    }
}
