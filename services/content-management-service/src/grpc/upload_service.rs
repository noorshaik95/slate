use crate::upload::{UploadError, UploadHandler};
use bytes::Bytes;
use std::sync::Arc;
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

// Import generated protobuf types
use crate::proto::content::{
    upload_service_server::UploadService, CancelUploadRequest, CompleteUploadRequest,
    InitiateUploadRequest, Resource as ProtoResource, UploadChunkRequest, UploadChunkResponse,
    UploadSession as ProtoUploadSession,
};

/// Implementation of the UploadService gRPC service
pub struct UploadServiceImpl {
    handler: Arc<UploadHandler>,
}

impl UploadServiceImpl {
    /// Creates a new UploadServiceImpl
    pub fn new(handler: Arc<UploadHandler>) -> Self {
        Self { handler }
    }
}

#[tonic::async_trait]
impl UploadService for UploadServiceImpl {
    /// Initiates a new upload session
    #[instrument(skip(self, request))]
    async fn initiate_upload(
        &self,
        request: Request<InitiateUploadRequest>,
    ) -> Result<Response<ProtoUploadSession>, Status> {
        // Extract user_id from metadata (set by API Gateway auth middleware)
        let user_id = extract_user_id_from_metadata(&request)?;

        let req = request.into_inner();

        info!(
            "Initiating upload: lesson={}, filename={}, size={}",
            req.lesson_id, req.filename, req.total_size
        );

        // Parse lesson_id
        let lesson_id = Uuid::parse_str(&req.lesson_id).map_err(|e| {
            warn!("Invalid lesson_id: {}", e);
            Status::invalid_argument(format!("Invalid lesson_id: {}", e))
        })?;

        // Initiate upload
        let session = self
            .handler
            .initiate_upload(
                user_id,
                lesson_id,
                req.filename,
                req.content_type,
                req.total_size,
            )
            .await
            .map_err(|e| {
                error!("Failed to initiate upload: {}", e);
                upload_error_to_status(e)
            })?;

        // Convert to proto
        let proto_session = ProtoUploadSession {
            session_id: session.id.to_string(),
            storage_key: session.storage_key,
            chunk_size: session.chunk_size,
            total_chunks: session.total_chunks,
            uploaded_chunks: session.uploaded_chunks,
            expires_at: Some(prost_types::Timestamp {
                seconds: session.expires_at.timestamp(),
                nanos: session.expires_at.timestamp_subsec_nanos() as i32,
            }),
        };

        info!("Upload session created: {}", session.id);

        Ok(Response::new(proto_session))
    }

    /// Handles streaming chunk uploads
    #[instrument(skip(self, request))]
    async fn upload_chunk(
        &self,
        request: Request<Streaming<UploadChunkRequest>>,
    ) -> Result<Response<UploadChunkResponse>, Status> {
        let mut stream = request.into_inner();

        let mut session_id: Option<Uuid> = None;
        let mut total_uploaded = 0;
        let mut total_chunks = 0;
        let mut progress = 0.0;

        // Process each chunk in the stream
        while let Some(chunk_req) = stream.message().await.map_err(|e| {
            error!("Error reading chunk stream: {}", e);
            Status::internal(format!("Stream error: {}", e))
        })? {
            // Parse session_id on first chunk
            if session_id.is_none() {
                session_id = Some(Uuid::parse_str(&chunk_req.session_id).map_err(|e| {
                    warn!("Invalid session_id: {}", e);
                    Status::invalid_argument(format!("Invalid session_id: {}", e))
                })?);
            }

            let sid = session_id.unwrap();

            // Process chunk
            let result = self
                .handler
                .process_chunk(
                    sid,
                    chunk_req.chunk_index,
                    Bytes::from(chunk_req.chunk_data),
                )
                .await
                .map_err(|e| {
                    error!("Failed to process chunk: {}", e);
                    upload_error_to_status(e)
                })?;

            total_uploaded = result.uploaded_chunks;
            total_chunks = result.total_chunks;
            progress = result.progress_percentage;

            info!(
                "Chunk processed: session={}, chunk={}, progress={:.1}%",
                sid, chunk_req.chunk_index, progress
            );
        }

        // Return final progress
        let response = UploadChunkResponse {
            uploaded_chunks: total_uploaded,
            total_chunks,
            progress_percentage: progress as f32,
        };

        Ok(Response::new(response))
    }

    /// Completes an upload and creates a resource
    #[instrument(skip(self, request))]
    async fn complete_upload(
        &self,
        request: Request<CompleteUploadRequest>,
    ) -> Result<Response<ProtoResource>, Status> {
        let req = request.into_inner();

        info!("Completing upload: session={}", req.session_id);

        // Parse session_id
        let session_id = Uuid::parse_str(&req.session_id).map_err(|e| {
            warn!("Invalid session_id: {}", e);
            Status::invalid_argument(format!("Invalid session_id: {}", e))
        })?;

        // Complete upload (lesson_id is retrieved from session)
        let resource = self
            .handler
            .complete_upload(session_id, req.name, Some(req.description))
            .await
            .map_err(|e| {
                error!("Failed to complete upload: {}", e);
                upload_error_to_status(e)
            })?;

        // Convert to proto
        let proto_resource = ProtoResource {
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
        };

        info!("Upload completed: resource={}", resource.id);

        Ok(Response::new(proto_resource))
    }

    /// Cancels an upload session
    #[instrument(skip(self, request))]
    async fn cancel_upload(
        &self,
        request: Request<CancelUploadRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();

        info!("Cancelling upload: session={}", req.session_id);

        // Parse session_id
        let session_id = Uuid::parse_str(&req.session_id).map_err(|e| {
            warn!("Invalid session_id: {}", e);
            Status::invalid_argument(format!("Invalid session_id: {}", e))
        })?;

        // Cancel upload
        self.handler.cancel_upload(session_id).await.map_err(|e| {
            error!("Failed to cancel upload: {}", e);
            upload_error_to_status(e)
        })?;

        info!("Upload cancelled: session={}", session_id);

        Ok(Response::new(()))
    }
}

/// Extracts user_id from gRPC metadata
fn extract_user_id_from_metadata<T>(request: &Request<T>) -> Result<Uuid, Status> {
    let metadata = request.metadata();

    let user_id_str = metadata
        .get("x-user-id")
        .ok_or_else(|| Status::unauthenticated("Missing user_id in metadata"))?
        .to_str()
        .map_err(|e| Status::internal(format!("Invalid user_id metadata: {}", e)))?;

    Uuid::parse_str(user_id_str)
        .map_err(|e| Status::internal(format!("Invalid user_id format: {}", e)))
}

/// Converts UploadError to gRPC Status
fn upload_error_to_status(error: UploadError) -> Status {
    match error {
        UploadError::InvalidFileType(_) => Status::invalid_argument(error.to_string()),
        UploadError::FileSizeExceeded(_, _) => Status::invalid_argument(error.to_string()),
        UploadError::InvalidFilename(_) => Status::invalid_argument(error.to_string()),
        UploadError::SessionNotFound(_) => Status::not_found(error.to_string()),
        UploadError::SessionExpired(_) => Status::failed_precondition(error.to_string()),
        UploadError::SessionNotResumable(_) => Status::failed_precondition(error.to_string()),
        UploadError::InvalidChunkIndex(_) => Status::invalid_argument(error.to_string()),
        UploadError::ChunkAlreadyUploaded(_) => Status::already_exists(error.to_string()),
        UploadError::MalwareDetected(_) => Status::permission_denied(error.to_string()),
        UploadError::FileHeaderMismatch(_) => Status::invalid_argument(error.to_string()),
        UploadError::StorageError(_) => Status::internal(error.to_string()),
        UploadError::DatabaseError(_) => Status::internal(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_error_to_status() {
        let err = UploadError::InvalidFileType("text/plain".to_string());
        let status = upload_error_to_status(err);
        assert_eq!(status.code(), tonic::Code::InvalidArgument);

        let err = UploadError::SessionNotFound("123".to_string());
        let status = upload_error_to_status(err);
        assert_eq!(status.code(), tonic::Code::NotFound);

        let err = UploadError::StorageError("S3 error".to_string());
        let status = upload_error_to_status(err);
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
