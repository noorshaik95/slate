//! Metrics recording utilities.
//!
//! Handles recording of request and gRPC call metrics for observability.

use std::sync::Arc;
use std::time::Instant;

use crate::shared::state::AppState;

/// Record success metrics for completed requests.
pub fn record_success_metrics(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    path: &str,
    method: &str,
    start_time: Instant,
) {
    let duration = start_time.elapsed();

    state
        .metrics
        .request_duration
        .with_label_values(&[path, method])
        .observe(duration.as_secs_f64());

    state
        .metrics
        .request_counter
        .with_label_values(&[path, method, "200"])
        .inc();

    state
        .metrics
        .grpc_call_counter
        .with_label_values(&[
            routing_decision.service.as_ref(),
            routing_decision.grpc_method.as_ref(),
            "success",
        ])
        .inc();
}

/// Record error metrics for failed requests.
#[allow(dead_code)]
pub fn record_error_metrics(state: &Arc<AppState>, path: &str, method: &str, status_code: &str) {
    state
        .metrics
        .request_counter
        .with_label_values(&[path, method, status_code])
        .inc();
}
