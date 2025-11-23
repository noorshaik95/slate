use uuid::Uuid;

/// W3C Trace Context representation
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
            trace_id: Uuid::new_v4().to_string(),
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

/// Extract trace ID from the current OpenTelemetry span
///
/// Returns the trace ID as a hex string from the current span context.
/// If no span is active or trace ID is invalid, generates a UUID as fallback.
#[cfg(feature = "observability")]
pub fn extract_trace_id_from_span() -> String {
    use opentelemetry::trace::TraceContextExt;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let current_span = tracing::Span::current();
    let context = current_span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();

    // Check if we have a valid trace ID
    let trace_id = span_context.trace_id();

    // OpenTelemetry trace ID is invalid if it's all zeros or empty
    let trace_id_str = trace_id.to_string();
    if !trace_id_str.is_empty() && trace_id_str != "00000000000000000000000000000000" {
        trace_id_str
    } else {
        // Generate fallback UUID
        let fallback_trace_id = Uuid::new_v4().to_string();

        tracing::warn!(
            trace_id = %fallback_trace_id,
            "No active OpenTelemetry span or invalid trace ID, using fallback trace ID"
        );

        fallback_trace_id
    }
}

/// Extract trace context from HTTP headers
#[cfg(feature = "http")]
pub fn extract_trace_context_from_headers(headers: &axum::http::HeaderMap) -> Option<TraceContext> {
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
pub fn parse_traceparent(traceparent: &str) -> Option<TraceContext> {
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
