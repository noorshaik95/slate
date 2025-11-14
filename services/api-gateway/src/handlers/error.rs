use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use tracing::warn;

/// Error response structure with trace ID support
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub trace_id: String,
}

impl ErrorResponse {
    /// Create a new error response with trace ID
    pub fn new(code: impl Into<String>, message: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                trace_id: trace_id.into(),
            },
        }
    }

    /// Convert to HTTP response with trace ID in header and body
    pub fn into_response_with_status(self, status: StatusCode) -> Response {
        let trace_id = self.error.trace_id.clone();
        
        let mut headers = HeaderMap::new();
        if let Ok(header_value) = trace_id.parse() {
            headers.insert("x-trace-id", header_value);
        }
        
        (status, headers, Json(self)).into_response()
    }
}

/// Extract trace ID from request extensions or headers
///
/// Priority order:
/// 1. OpenTelemetry trace context from request extensions
/// 2. W3C traceparent header
/// 3. x-trace-id header
/// 4. x-request-id header
/// 5. Generate new UUID
pub fn extract_trace_id<B>(request: &Request<B>) -> String {
    // Try to get trace ID from OpenTelemetry context (set by tracing middleware)
    if let Some(trace_ctx) = request.extensions().get::<crate::observability::tracing_utils::TraceContext>() {
        return trace_ctx.trace_id.clone();
    }
    
    // Try to extract from headers
    let headers = request.headers();
    
    // Try W3C traceparent header (format: 00-{trace-id}-{parent-id}-{trace-flags})
    if let Some(traceparent) = headers.get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            if let Some(trace_id) = parse_trace_id_from_traceparent(traceparent_str) {
                return trace_id;
            }
        }
    }
    
    // Try x-trace-id header
    if let Some(trace_id) = headers.get("x-trace-id") {
        if let Ok(trace_id_str) = trace_id.to_str() {
            if !trace_id_str.is_empty() {
                return trace_id_str.to_string();
            }
        }
    }
    
    // Try x-request-id header as fallback
    if let Some(request_id) = headers.get("x-request-id") {
        if let Ok(request_id_str) = request_id.to_str() {
            if !request_id_str.is_empty() {
                return request_id_str.to_string();
            }
        }
    }
    
    // Generate new UUID if no trace ID found
    let new_trace_id = uuid::Uuid::new_v4().to_string();
    warn!(
        trace_id = %new_trace_id,
        "No trace ID found in request, generated new UUID"
    );
    new_trace_id
}

/// Parse trace ID from W3C traceparent header
///
/// Format: 00-{trace-id}-{parent-id}-{trace-flags}
/// Returns the trace-id portion (32 hex characters)
fn parse_trace_id_from_traceparent(traceparent: &str) -> Option<String> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    
    if parts.len() != 4 {
        return None;
    }
    
    // parts[0] is version (should be "00")
    // parts[1] is trace-id (32 hex chars)
    if parts[0] != "00" {
        return None;
    }
    
    Some(parts[1].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;

    #[test]
    fn test_error_response_creation() {
        let error = ErrorResponse::new("NOT_FOUND", "Resource not found", "trace-123");
        
        assert_eq!(error.error.code, "NOT_FOUND");
        assert_eq!(error.error.message, "Resource not found");
        assert_eq!(error.error.trace_id, "trace-123");
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new("INTERNAL_ERROR", "Something went wrong", "trace-456");
        let json = serde_json::to_string(&error).unwrap();
        
        assert!(json.contains("\"code\":\"INTERNAL_ERROR\""));
        assert!(json.contains("\"message\":\"Something went wrong\""));
        assert!(json.contains("\"trace_id\":\"trace-456\""));
    }

    #[test]
    fn test_parse_trace_id_from_traceparent_valid() {
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let trace_id = parse_trace_id_from_traceparent(traceparent);
        
        assert_eq!(trace_id, Some("0af7651916cd43dd8448eb211c80319c".to_string()));
    }

    #[test]
    fn test_parse_trace_id_from_traceparent_invalid_version() {
        let traceparent = "01-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let trace_id = parse_trace_id_from_traceparent(traceparent);
        
        assert_eq!(trace_id, None);
    }

    #[test]
    fn test_parse_trace_id_from_traceparent_invalid_format() {
        let traceparent = "invalid-trace-parent";
        let trace_id = parse_trace_id_from_traceparent(traceparent);
        
        assert_eq!(trace_id, None);
    }

    #[test]
    fn test_extract_trace_id_from_traceparent_header() {
        let request = Request::builder()
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .body(Body::empty())
            .unwrap();
        
        let trace_id = extract_trace_id(&request);
        assert_eq!(trace_id, "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_extract_trace_id_from_x_trace_id_header() {
        let request = Request::builder()
            .header("x-trace-id", "custom-trace-123")
            .body(Body::empty())
            .unwrap();
        
        let trace_id = extract_trace_id(&request);
        assert_eq!(trace_id, "custom-trace-123");
    }

    #[test]
    fn test_extract_trace_id_from_x_request_id_header() {
        let request = Request::builder()
            .header("x-request-id", "request-456")
            .body(Body::empty())
            .unwrap();
        
        let trace_id = extract_trace_id(&request);
        assert_eq!(trace_id, "request-456");
    }

    #[test]
    fn test_extract_trace_id_generates_uuid_when_missing() {
        let request = Request::builder()
            .body(Body::empty())
            .unwrap();
        
        let trace_id = extract_trace_id(&request);
        
        // Should be a valid UUID format
        assert!(uuid::Uuid::parse_str(&trace_id).is_ok());
    }

    #[test]
    fn test_extract_trace_id_priority_traceparent_over_x_trace_id() {
        let request = Request::builder()
            .header("traceparent", "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01")
            .header("x-trace-id", "should-not-use-this")
            .body(Body::empty())
            .unwrap();
        
        let trace_id = extract_trace_id(&request);
        assert_eq!(trace_id, "0af7651916cd43dd8448eb211c80319c");
    }
}
