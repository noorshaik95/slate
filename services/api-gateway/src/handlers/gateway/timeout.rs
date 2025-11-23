//! Request timeout handling.
//!
//! Wraps gateway requests with configurable timeouts to prevent hanging requests.

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    response::Response,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::error;

use crate::shared::state::AppState;
use common_rust::observability::extract_trace_id_from_span;

use super::response::build_response;
use super::routing::process_request;
use crate::handlers::types::GatewayError;

/// Handle request with timeout wrapper.
///
/// Wraps the actual handler with a configurable timeout to prevent hanging requests.
/// If the timeout is exceeded, returns a Gateway Timeout error.
pub async fn handle_with_timeout(
    state: Arc<AppState>,
    addr: SocketAddr,
    headers: HeaderMap,
    request: Request<Body>,
) -> Response {
    let timeout_duration = Duration::from_millis(state.config.server.request_timeout_ms);

    let start = Instant::now();
    let result = match tokio::time::timeout(
        timeout_duration,
        process_request(State(state), ConnectInfo(addr), headers, request),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            // Extract trace ID from OpenTelemetry span for timeout error logging
            let trace_id = extract_trace_id_from_span();
            let duration_ms = start.elapsed().as_millis();
            error!(
                timeout_ms = ?timeout_duration.as_millis(),
                duration_ms = %duration_ms,
                trace_id = %trace_id,
                error_type = "timeout",
                "Request timeout exceeded"
            );
            Err(GatewayError::Timeout)
        }
    };

    build_response(result)
}
