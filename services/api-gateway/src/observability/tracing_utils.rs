use tracing::{span, Level, Span, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opentelemetry::trace::TraceContextExt;

/// Extract trace ID from the current OpenTelemetry span
///
/// Returns the trace ID as a hex string (32 characters) from the current span context.
/// If no span is active or trace ID is invalid, generates a UUID as fallback.
///
/// # Examples
///
/// ```
/// use crate::observability::tracing_utils::extract_trace_id_from_span;
///
/// let trace_id = extract_trace_id_from_span();
/// info!(trace_id = %trace_id, "Processing request");
/// ```
///
/// # Fallback Behavior
///
/// This function will generate a UUID v4 as fallback in the following cases:
/// - No active OpenTelemetry span exists
/// - The span context has an invalid trace ID (all zeros)
/// - Any error occurs during trace ID extraction
///
/// A warning is logged when fallback is used to help with debugging.
pub fn extract_trace_id_from_span() -> String {
    let current_span = tracing::Span::current();
    let context = current_span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();
    
    // Check if we have a valid trace ID
    let trace_id = span_context.trace_id();
    
    // OpenTelemetry trace ID is invalid if it's all zeros or empty
    let trace_id_str = trace_id.to_string();
    if !trace_id_str.is_empty() && trace_id_str != "00000000000000000000000000000000" {
        // Convert trace ID to lowercase hex string (32 characters)
        trace_id_str
    } else {
        // Generate fallback UUID
        let fallback_trace_id = uuid::Uuid::new_v4().to_string();
        
        warn!(
            trace_id = %fallback_trace_id,
            "No active OpenTelemetry span or invalid trace ID, using fallback trace ID"
        );
        
        fallback_trace_id
    }
}

/// Attributes for creating structured spans
#[derive(Debug, Clone)]
pub struct SpanAttributes {
    pub http_method: String,
    pub http_target: String,
    pub http_route: Option<String>,
    pub rpc_service: Option<String>,
    pub rpc_method: Option<String>,
    pub user_id: Option<String>,
}

/// Create a structured span for a request with standard attributes
///
/// This creates spans following OpenTelemetry semantic conventions:
/// - http.method: HTTP method (GET, POST, etc.)
/// - http.target: Full request path
/// - http.route: Route pattern (e.g., /api/users/:id)
/// - rpc.service: Backend gRPC service name
/// - rpc.method: Backend gRPC method name
/// - user.id: Authenticated user ID (if available)
pub fn create_request_span(attrs: SpanAttributes) -> Span {
    let span = span!(
        Level::INFO,
        "gateway_request",
        http.method = %attrs.http_method,
        http.target = %attrs.http_target,
        http.route = attrs.http_route.as_deref().unwrap_or("unknown"),
        rpc.service = attrs.rpc_service.as_deref().unwrap_or("unknown"),
        rpc.method = attrs.rpc_method.as_deref().unwrap_or("unknown"),
        user.id = attrs.user_id.as_deref().unwrap_or("anonymous"),
    );

    span
}

/// Record the response status on a span
pub fn record_response_status(span: &Span, status_code: u16, error: Option<&str>) {
    span.record("http.status_code", status_code);
    
    if let Some(err) = error {
        span.record("error", true);
        span.record("error.message", err);
    } else {
        span.record("error", false);
    }
}

/// Extract trace context from HTTP headers
///
/// Supports W3C Trace Context (traceparent, tracestate)
pub fn extract_trace_context(headers: &axum::http::HeaderMap) -> Option<TraceContext> {
    // Try to extract W3C traceparent header
    if let Some(traceparent) = headers.get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            return parse_traceparent(traceparent_str);
        }
    }

    // Fallback to x-trace-id if present
    if let Some(trace_id) = headers.get("x-trace-id") {
        if let Ok(trace_id_str) = trace_id.to_str() {
            return Some(TraceContext {
                trace_id: trace_id_str.to_string(),
                span_id: None,
                trace_flags: None,
            });
        }
    }

    None
}

/// Parse W3C traceparent header
///
/// Format: 00-{trace-id}-{parent-id}-{trace-flags}
fn parse_traceparent(traceparent: &str) -> Option<TraceContext> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    
    if parts.len() != 4 {
        return None;
    }

    // parts[0] is version (should be "00")
    // parts[1] is trace-id (32 hex chars)
    // parts[2] is parent-id/span-id (16 hex chars)
    // parts[3] is trace-flags (2 hex chars)

    if parts[0] != "00" {
        return None; // Only support version 00
    }

    Some(TraceContext {
        trace_id: parts[1].to_string(),
        span_id: Some(parts[2].to_string()),
        trace_flags: Some(parts[3].to_string()),
    })
}

/// Trace context extracted from headers
#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: Option<String>,
    pub trace_flags: Option<String>,
}

impl TraceContext {
    /// Create a new root trace context
    pub fn new_root() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: None,
            trace_flags: Some("01".to_string()), // Sampled
        }
    }

    /// Format as W3C traceparent header
    pub fn to_traceparent(&self) -> String {
        let span_id = self.span_id.as_deref().unwrap_or("0000000000000000");
        let flags = self.trace_flags.as_deref().unwrap_or("01");
        
        format!("00-{}-{}-{}", self.trace_id, span_id, flags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_traceparent_valid() {
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let context = parse_traceparent(traceparent);
        
        assert!(context.is_some());
        let ctx = context.unwrap();
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.span_id.unwrap(), "b7ad6b7169203331");
        assert_eq!(ctx.trace_flags.unwrap(), "01");
    }

    #[test]
    fn test_parse_traceparent_invalid_version() {
        let traceparent = "01-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let context = parse_traceparent(traceparent);
        
        assert!(context.is_none());
    }

    #[test]
    fn test_parse_traceparent_invalid_format() {
        let traceparent = "invalid-trace-parent";
        let context = parse_traceparent(traceparent);
        
        assert!(context.is_none());
    }

    #[test]
    fn test_trace_context_to_traceparent() {
        let context = TraceContext {
            trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
            span_id: Some("b7ad6b7169203331".to_string()),
            trace_flags: Some("01".to_string()),
        };

        let traceparent = context.to_traceparent();
        assert_eq!(traceparent, "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01");
    }

    #[test]
    fn test_new_root_trace_context() {
        let context = TraceContext::new_root();
        
        assert!(!context.trace_id.is_empty());
        assert_eq!(context.trace_flags, Some("01".to_string()));
    }

    #[test]
    fn test_span_attributes() {
        let attrs = SpanAttributes {
            http_method: "POST".to_string(),
            http_target: "/api/users".to_string(),
            http_route: Some("/api/users".to_string()),
            rpc_service: Some("user.UserService".to_string()),
            rpc_method: Some("CreateUser".to_string()),
            user_id: Some("user-123".to_string()),
        };

        assert_eq!(attrs.http_method, "POST");
        assert_eq!(attrs.rpc_service.unwrap(), "user.UserService");
    }

    #[test]
    fn test_extract_trace_id_fallback_no_span() {
        // When no active span exists, should generate UUID fallback
        let trace_id = super::extract_trace_id_from_span();
        
        // UUID format: 8-4-4-4-12 characters with hyphens (36 total)
        assert_eq!(trace_id.len(), 36);
        assert_eq!(trace_id.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn test_trace_id_format() {
        // Test that the returned trace ID is a valid format
        let trace_id = super::extract_trace_id_from_span();
        
        // Should be either:
        // - 32 hex characters (OpenTelemetry trace ID)
        // - 36 characters with hyphens (UUID fallback)
        assert!(
            trace_id.len() == 32 || trace_id.len() == 36,
            "Trace ID should be 32 hex chars or 36 char UUID, got length: {}",
            trace_id.len()
        );
        
        // Should be lowercase
        assert_eq!(trace_id, trace_id.to_lowercase());
    }
}
