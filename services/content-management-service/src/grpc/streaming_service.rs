use crate::proto::content::{
    streaming_service_server::StreamingService as StreamingServiceTrait, GetPlaybackStateRequest,
    GetVideoManifestRequest, PlaybackState as ProtoPlaybackState,
    QualityLevel as ProtoQualityLevel, UpdatePlaybackPositionRequest,
    VideoManifest as ProtoVideoManifest,
};
use crate::streaming::{StreamingError, StreamingService};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

/// gRPC implementation of StreamingService
pub struct StreamingServiceImpl {
    service: Arc<StreamingService>,
}

impl StreamingServiceImpl {
    /// Creates a new StreamingServiceImpl
    pub fn new(service: Arc<StreamingService>) -> Self {
        Self { service }
    }

    /// Extracts user ID from request metadata
    fn extract_user_id(request: &Request<impl std::fmt::Debug>) -> Result<Uuid, Status> {
        let metadata = request.metadata();
        let user_id_str = metadata
            .get("user-id")
            .ok_or_else(|| Status::unauthenticated("Missing user-id in metadata"))?
            .to_str()
            .map_err(|_| Status::invalid_argument("Invalid user-id format"))?;

        Uuid::parse_str(user_id_str)
            .map_err(|_| Status::invalid_argument("Invalid user-id UUID format"))
    }

    /// Extracts user role from request metadata
    fn extract_user_role(request: &Request<impl std::fmt::Debug>) -> Result<String, Status> {
        let metadata = request.metadata();
        let role = metadata
            .get("user-role")
            .ok_or_else(|| Status::unauthenticated("Missing user-role in metadata"))?
            .to_str()
            .map_err(|_| Status::invalid_argument("Invalid user-role format"))?;

        Ok(role.to_string())
    }

    /// Checks if user is an instructor
    fn is_instructor(role: &str) -> bool {
        role.eq_ignore_ascii_case("instructor") || role.eq_ignore_ascii_case("admin")
    }
}

#[tonic::async_trait]
impl StreamingServiceTrait for StreamingServiceImpl {
    /// Gets video manifest for streaming
    async fn get_video_manifest(
        &self,
        request: Request<GetVideoManifestRequest>,
    ) -> Result<Response<ProtoVideoManifest>, Status> {
        let user_id = Self::extract_user_id(&request)?;
        let role = Self::extract_user_role(&request)?;
        let is_instructor = Self::is_instructor(&role);

        let req = request.into_inner();

        // Parse video ID
        let video_id = Uuid::parse_str(&req.video_id)
            .map_err(|_| Status::invalid_argument("Invalid video_id format"))?;

        // Extract session ID for analytics
        let session_id = if req.session_id.is_empty() {
            None
        } else {
            Some(req.session_id)
        };

        // Get video manifest
        let manifest = self
            .service
            .get_video_manifest(video_id, user_id, is_instructor, session_id)
            .await
            .map_err(|e| match e {
                StreamingError::VideoNotFound(_) => Status::not_found(e.to_string()),
                StreamingError::VideoNotPublished => Status::permission_denied(e.to_string()),
                StreamingError::NotAVideo => Status::invalid_argument(e.to_string()),
                StreamingError::NotTranscoded => {
                    Status::failed_precondition("Video is still being transcoded")
                }
                StreamingError::AccessDenied(_) => Status::permission_denied(e.to_string()),
                StreamingError::InvalidPosition(_) => Status::invalid_argument(e.to_string()),
                StreamingError::DatabaseError(_) => Status::internal(e.to_string()),
            })?;

        // Convert to proto
        let proto_manifest = ProtoVideoManifest {
            hls_url: manifest.hls_url.unwrap_or_default(),
            dash_url: manifest.dash_url.unwrap_or_default(),
            quality_levels: manifest
                .quality_levels
                .into_iter()
                .map(|q| ProtoQualityLevel {
                    height: q.height,
                    bitrate: q.bitrate,
                })
                .collect(),
        };

        Ok(Response::new(proto_manifest))
    }

    /// Updates playback position
    async fn update_playback_position(
        &self,
        request: Request<UpdatePlaybackPositionRequest>,
    ) -> Result<Response<()>, Status> {
        let user_id = Self::extract_user_id(&request)?;

        let req = request.into_inner();

        // Parse video ID
        let video_id = Uuid::parse_str(&req.video_id)
            .map_err(|_| Status::invalid_argument("Invalid video_id format"))?;

        // Validate position
        if req.position_seconds < 0 {
            return Err(Status::invalid_argument("Position must be non-negative"));
        }

        // Parse event type
        use crate::streaming::PlaybackEventType;
        let event_type = match req.event_type.to_lowercase().as_str() {
            "pause" => PlaybackEventType::Pause,
            "seek" => PlaybackEventType::Seek {
                old_position: req.old_position_seconds,
            },
            _ => PlaybackEventType::Update,
        };

        // Extract session ID for analytics
        let session_id = if req.session_id.is_empty() {
            None
        } else {
            Some(req.session_id)
        };

        // Update playback position
        self.service
            .update_playback_position(
                video_id,
                user_id,
                req.position_seconds,
                event_type,
                session_id,
            )
            .await
            .map_err(|e| match e {
                StreamingError::VideoNotFound(_) => Status::not_found(e.to_string()),
                StreamingError::NotAVideo => Status::invalid_argument(e.to_string()),
                StreamingError::InvalidPosition(_) => Status::invalid_argument(e.to_string()),
                StreamingError::AccessDenied(_) => Status::permission_denied(e.to_string()),
                StreamingError::VideoNotPublished => Status::permission_denied(e.to_string()),
                StreamingError::NotTranscoded => Status::failed_precondition(e.to_string()),
                StreamingError::DatabaseError(_) => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(()))
    }

    /// Gets playback state
    async fn get_playback_state(
        &self,
        request: Request<GetPlaybackStateRequest>,
    ) -> Result<Response<ProtoPlaybackState>, Status> {
        let user_id = Self::extract_user_id(&request)?;
        let role = Self::extract_user_role(&request)?;
        let is_instructor = Self::is_instructor(&role);

        let req = request.into_inner();

        // Parse video ID
        let video_id = Uuid::parse_str(&req.video_id)
            .map_err(|_| Status::invalid_argument("Invalid video_id format"))?;

        // Get playback state
        let state = self
            .service
            .get_playback_state(video_id, user_id, is_instructor)
            .await
            .map_err(|e| match e {
                StreamingError::VideoNotFound(_) => Status::not_found(e.to_string()),
                StreamingError::VideoNotPublished => Status::permission_denied(e.to_string()),
                StreamingError::NotAVideo => Status::invalid_argument(e.to_string()),
                StreamingError::AccessDenied(_) => Status::permission_denied(e.to_string()),
                StreamingError::InvalidPosition(_) => Status::invalid_argument(e.to_string()),
                StreamingError::NotTranscoded => Status::failed_precondition(e.to_string()),
                StreamingError::DatabaseError(_) => Status::internal(e.to_string()),
            })?;

        // Convert to proto
        let proto_state = ProtoPlaybackState {
            duration_seconds: state.duration_seconds,
            current_position_seconds: state.current_position_seconds,
            playback_speeds: state.playback_speeds,
            available_qualities: state
                .available_qualities
                .into_iter()
                .map(|q| ProtoQualityLevel {
                    height: q.height,
                    bitrate: q.bitrate,
                })
                .collect(),
        };

        Ok(Response::new(proto_state))
    }
}
