//! Response building utilities.
//!
//! Handles conversion of results to HTTP responses with proper trace ID injection.

use axum::response::Response;
use common_rust::observability::extract_trace_id_from_span;

use crate::handlers::types::GatewayError;

/// Build HTTP response from result.
///
/// Converts a Result into an HTTP response, adding trace ID headers and
/// handling errors appropriately.
pub fn build_response(result: Result<Response, GatewayError>) -> Response {
    match result {
        Ok(response) => {
            // Add trace ID header to successful responses
            let trace_id = extract_trace_id_from_span();
            let (mut parts, body) = response.into_parts();
            if let Ok(header_value) = trace_id.parse() {
                parts.headers.insert("x-trace-id", header_value);
            }
            Response::from_parts(parts, body)
        }
        Err(error) => {
            // Convert error to response with trace ID from OpenTelemetry span
            let trace_id = extract_trace_id_from_span();
            error.into_response_with_trace_id(trace_id)
        }
    }
}
