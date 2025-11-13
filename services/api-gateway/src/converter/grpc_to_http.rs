use axum::body::Body;
use axum::http::{Response, StatusCode};
use serde_json::{json, Value};
use tracing::{debug, warn};

use crate::grpc::GrpcResponse;

use super::constants::CLIENT_PROPAGATE_HEADERS;
use super::types::ConversionError;

/// Converter for gRPC to HTTP transformations
pub struct GrpcToHttpConverter;

impl GrpcToHttpConverter {
    /// Convert a gRPC response to HTTP format
    /// 
    /// This function:
    /// - Converts the gRPC status code to HTTP status code
    /// - Parses the response payload as JSON
    /// - Extracts metadata and converts relevant items to HTTP headers
    /// - Builds the HTTP response
    pub fn convert_response(
        grpc_resp: GrpcResponse,
    ) -> Result<Response<Body>, ConversionError> {
        debug!(
            status = ?grpc_resp.status,
            payload_size = grpc_resp.payload.len(),
            "Converting gRPC response to HTTP"
        );

        // Map gRPC status to HTTP status
        let http_status = Self::map_grpc_status_to_http(grpc_resp.status);

        // Parse the payload as JSON
        let json_body: Value = if grpc_resp.payload.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&grpc_resp.payload)
                .map_err(|e| {
                    warn!(error = %e, "Failed to parse gRPC response as JSON, returning raw bytes");
                    // If JSON parsing fails, try to return as string
                    ConversionError::JsonParseError(e.to_string())
                })?
        };

        // Build the response
        let mut response = Response::builder()
            .status(http_status)
            .header("content-type", "application/json");

        // Add metadata as headers
        for (key, value) in &grpc_resp.metadata {
            // Only propagate certain metadata back to client
            if Self::should_propagate_to_client(key) {
                response = response.header(key, value);
                debug!(header = %key, value = %value, "Propagating metadata to HTTP header");
            }
        }

        // Serialize JSON body
        let body_bytes = serde_json::to_vec(&json_body)
            .map_err(|e| ConversionError::JsonSerializeError(e.to_string()))?;

        let response = response
            .body(Body::from(body_bytes))
            .map_err(|e| ConversionError::BodyReadError(e.to_string()))?;

        debug!(status = ?http_status, "gRPC to HTTP conversion complete");

        Ok(response)
    }

    /// Map gRPC status codes to HTTP status codes
    pub(super) fn map_grpc_status_to_http(grpc_status: tonic::Code) -> StatusCode {
        match grpc_status {
            tonic::Code::Ok => StatusCode::OK,
            tonic::Code::Cancelled => StatusCode::REQUEST_TIMEOUT,
            tonic::Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            tonic::Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            tonic::Code::NotFound => StatusCode::NOT_FOUND,
            tonic::Code::AlreadyExists => StatusCode::CONFLICT,
            tonic::Code::PermissionDenied => StatusCode::FORBIDDEN,
            tonic::Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            tonic::Code::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
            tonic::Code::Aborted => StatusCode::CONFLICT,
            tonic::Code::OutOfRange => StatusCode::BAD_REQUEST,
            tonic::Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            tonic::Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            tonic::Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            tonic::Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }

    /// Determine if a metadata key should be propagated back to the client
    fn should_propagate_to_client(key: &str) -> bool {
        CLIENT_PROPAGATE_HEADERS.contains(&key)
    }
}
