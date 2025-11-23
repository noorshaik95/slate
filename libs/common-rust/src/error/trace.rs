use uuid::Uuid;

/// Extract trace ID from HTTP headers (requires http feature)
#[cfg(feature = "http")]
pub fn extract_trace_id_from_http(headers: &axum::http::HeaderMap) -> String {
    // Try traceparent first (W3C Trace Context)
    if let Some(traceparent) = headers.get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            if let Some(trace_id) = parse_traceparent(traceparent_str) {
                return trace_id;
            }
        }
    }

    // Try x-trace-id
    if let Some(trace_id) = headers.get("x-trace-id") {
        if let Ok(trace_id_str) = trace_id.to_str() {
            return trace_id_str.to_string();
        }
    }

    // Try x-request-id
    if let Some(request_id) = headers.get("x-request-id") {
        if let Ok(request_id_str) = request_id.to_str() {
            return request_id_str.to_string();
        }
    }

    // Generate new trace ID
    generate_trace_id()
}

/// Extract trace ID from gRPC metadata (requires grpc feature)
#[cfg(feature = "grpc")]
pub fn extract_trace_id_from_grpc<T>(request: &tonic::Request<T>) -> String {
    let metadata = request.metadata();

    // Try traceparent first
    if let Some(traceparent) = metadata.get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            if let Some(trace_id) = parse_traceparent(traceparent_str) {
                return trace_id;
            }
        }
    }

    // Try x-trace-id
    if let Some(trace_id) = metadata.get("x-trace-id") {
        if let Ok(trace_id_str) = trace_id.to_str() {
            return trace_id_str.to_string();
        }
    }

    // Generate new trace ID
    generate_trace_id()
}

/// Parse W3C traceparent header and extract trace ID
/// Format: 00-{trace-id}-{parent-id}-{trace-flags}
fn parse_traceparent(traceparent: &str) -> Option<String> {
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

/// Generate a new trace ID using UUID v4
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string()
}
