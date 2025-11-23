use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::router::RouterError;
use common_rust::rate_limit::RateLimitError;

/// Gateway error types
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Route not found: {0}")]
    RouteNotFound(#[from] RouterError),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("gRPC call failed: {0}")]
    GrpcCallFailed(String),

    #[error("Request timeout")]
    Timeout,

    #[error("Not found")]
    NotFound,

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<RateLimitError> for GatewayError {
    fn from(_: RateLimitError) -> Self {
        GatewayError::RateLimitExceeded
    }
}

/// Error response structure
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl GatewayError {
    /// Convert to response with trace ID
    pub fn into_response_with_trace_id(self, trace_id: String) -> Response {
        use super::constants::*;

        let status = map_grpc_error_to_status(&self);

        let error_code = match &self {
            GatewayError::RouteNotFound(_) => ERR_CODE_ROUTE_NOT_FOUND,
            GatewayError::ServiceUnavailable(_) => ERR_CODE_SERVICE_UNAVAILABLE,
            GatewayError::RateLimitExceeded => ERR_CODE_RATE_LIMIT_EXCEEDED,
            GatewayError::ConversionError(_) => ERR_CODE_CONVERSION_ERROR,
            GatewayError::GrpcCallFailed(_) => ERR_CODE_BACKEND_ERROR,
            GatewayError::Timeout => ERR_CODE_TIMEOUT,
            GatewayError::NotFound => ERR_CODE_NOT_FOUND,
            GatewayError::InternalError(_) => ERR_CODE_INTERNAL_ERROR,
        };

        let error_response =
            super::error::ErrorResponse::new(error_code, self.to_string(), trace_id);

        error_response.into_response_with_status(status)
    }
}

impl IntoResponse for GatewayError {
    fn into_response(self) -> Response {
        // Fallback implementation without trace ID (for backward compatibility)
        // In practice, this should rarely be used as the gateway handler extracts trace ID
        let trace_id = uuid::Uuid::new_v4().to_string();
        self.into_response_with_trace_id(trace_id)
    }
}

/// Map gRPC errors to HTTP status codes
pub fn map_grpc_error_to_status(error: &GatewayError) -> StatusCode {
    match error {
        GatewayError::RouteNotFound(_) => StatusCode::NOT_FOUND,
        GatewayError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        GatewayError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        GatewayError::ConversionError(_) => StatusCode::BAD_REQUEST,
        GatewayError::GrpcCallFailed(_) => StatusCode::BAD_GATEWAY,
        GatewayError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        GatewayError::NotFound => StatusCode::NOT_FOUND,
        GatewayError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
