use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub trace_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Standard error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

impl ErrorResponse {
    /// Create a new error response
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        trace_id: impl Into<String>,
    ) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                trace_id: trace_id.into(),
                details: None,
            },
        }
    }

    /// Create an error response with additional details
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        trace_id: impl Into<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                trace_id: trace_id.into(),
                details: Some(details),
            },
        }
    }

    /// Convert to HTTP response (requires http feature)
    #[cfg(feature = "http")]
    pub fn to_http_response(&self, status_code: u16) -> axum::response::Response {
        use axum::http::{header, StatusCode};
        use axum::response::{IntoResponse, Json};

        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (
            status,
            [(header::CONTENT_TYPE, "application/json")],
            [(
                header::HeaderName::from_static("x-trace-id"),
                self.error.trace_id.clone(),
            )],
            Json(self.clone()),
        )
            .into_response()
    }

    /// Convert to gRPC status (requires grpc feature)
    #[cfg(feature = "grpc")]
    pub fn to_grpc_status(&self, code: tonic::Code) -> tonic::Status {
        // Create status with message
        tonic::Status::new(code, &self.error.message)
        // Note: Metadata can be added via interceptors if needed
    }
}
