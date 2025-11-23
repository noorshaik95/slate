//! Circuit breaker integration for backend service calls.
//!
//! Handles circuit breaker protection for gRPC calls to backend services.

use std::sync::Arc;
use std::time::Instant;
use tracing::{error, warn};

use crate::handlers::types::GatewayError;
use crate::shared::state::AppState;
use common_rust::observability::extract_trace_id_from_span;

use super::backend::call_backend_service;

/// Call backend service with circuit breaker protection.
///
/// If a circuit breaker is configured for the service, wraps the call with
/// circuit breaker logic. Otherwise, calls directly.
pub async fn call_with_circuit_breaker(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    service_channel: tonic::transport::Channel,
    grpc_request: Vec<u8>,
    path: &str,
    method: &str,
    start_time: Instant,
) -> Result<Vec<u8>, GatewayError> {
    if let Some(circuit_breaker) = state
        .grpc_pool
        .get_circuit_breaker(&routing_decision.service)
    {
        handle_with_breaker(
            state,
            routing_decision,
            service_channel,
            grpc_request,
            circuit_breaker,
            path,
            method,
            start_time,
        )
        .await
    } else {
        handle_without_breaker(
            state,
            routing_decision,
            service_channel,
            grpc_request,
            path,
            method,
        )
        .await
    }
}

/// Handle call with circuit breaker enabled.
#[allow(clippy::too_many_arguments)]
async fn handle_with_breaker(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    service_channel: tonic::transport::Channel,
    grpc_request: Vec<u8>,
    circuit_breaker: common_rust::circuit_breaker::CircuitBreaker,
    path: &str,
    method: &str,
    start_time: Instant,
) -> Result<Vec<u8>, GatewayError> {
    match circuit_breaker
        .call(call_backend_service(
            service_channel,
            &routing_decision.grpc_method,
            grpc_request,
        ))
        .await
    {
        Ok(resp) => Ok(resp),
        Err(common_rust::circuit_breaker::CircuitBreakerError::Open) => {
            handle_circuit_open(state, routing_decision, path, method)
        }
        Err(common_rust::circuit_breaker::CircuitBreakerError::OperationFailed(e)) => {
            handle_operation_failed(state, routing_decision, path, method, start_time, e)
        }
    }
}

/// Handle circuit breaker open state.
fn handle_circuit_open(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    path: &str,
    method: &str,
) -> Result<Vec<u8>, GatewayError> {
    warn!(
        service = %routing_decision.service,
        "Circuit breaker is OPEN - rejecting request"
    );

    state
        .metrics
        .request_counter
        .with_label_values(&[path, method, "503"])
        .inc();

    state
        .metrics
        .grpc_call_counter
        .with_label_values(&[
            routing_decision.service.as_ref(),
            routing_decision.grpc_method.as_ref(),
            "circuit_open",
        ])
        .inc();

    Err(GatewayError::ServiceUnavailable(format!(
        "Service {} is currently unavailable (circuit breaker open)",
        &*routing_decision.service
    )))
}

/// Handle operation failed error from circuit breaker.
fn handle_operation_failed(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    path: &str,
    method: &str,
    start_time: Instant,
    error_msg: String,
) -> Result<Vec<u8>, GatewayError> {
    let duration_ms = start_time.elapsed().as_millis();
    let trace_id = extract_trace_id_from_span();

    error!(
        service = %routing_decision.service,
        grpc_method = %routing_decision.grpc_method,
        path = %path,
        method = %method,
        trace_id = %trace_id,
        duration_ms = %duration_ms,
        error_type = "grpc_error",
        error = %error_msg,
        "Backend service call failed"
    );

    state
        .metrics
        .grpc_call_counter
        .with_label_values(&[
            routing_decision.service.as_ref(),
            routing_decision.grpc_method.as_ref(),
            "error",
        ])
        .inc();

    Err(GatewayError::GrpcCallFailed(error_msg))
}

/// Handle direct call without circuit breaker.
async fn handle_without_breaker(
    state: &Arc<AppState>,
    routing_decision: &crate::router::RoutingDecision,
    service_channel: tonic::transport::Channel,
    grpc_request: Vec<u8>,
    path: &str,
    method: &str,
) -> Result<Vec<u8>, GatewayError> {
    use crate::handlers::types::map_grpc_error_to_status;

    match call_backend_service(service_channel, &routing_decision.grpc_method, grpc_request).await {
        Ok(resp) => Ok(resp),
        Err(e) => {
            let trace_id = extract_trace_id_from_span();
            error!(
                service = %routing_decision.service,
                trace_id = %trace_id,
                grpc_method = %routing_decision.grpc_method,
                error = %e,
                "Backend service call failed"
            );

            let status_code = map_grpc_error_to_status(&e);
            let status_str = status_code.as_u16().to_string();

            state
                .metrics
                .request_counter
                .with_label_values(&[path, method, &status_str])
                .inc();

            state
                .metrics
                .grpc_call_counter
                .with_label_values(&[
                    routing_decision.service.as_ref(),
                    routing_decision.grpc_method.as_ref(),
                    "error",
                ])
                .inc();

            Err(e)
        }
    }
}
