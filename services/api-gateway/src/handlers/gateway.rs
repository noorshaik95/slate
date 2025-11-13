use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::auth::middleware::AuthContext;
use crate::rate_limit::RateLimiter;
use crate::shared::state::AppState;

use super::constants::*;
use super::types::{GatewayError, map_grpc_error_to_status};

/// Main gateway handler with timeout wrapper
///
/// Wraps the actual handler with a configurable timeout to prevent hanging requests
pub async fn gateway_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Response, GatewayError> {
    let timeout_duration = Duration::from_millis(state.config.server.request_timeout_ms);

    let start = Instant::now();
    match tokio::time::timeout(
        timeout_duration,
        gateway_handler_inner(State(state), ConnectInfo(addr), headers, request)
    ).await {
        Ok(result) => result,
        Err(_) => {
            let duration_ms = start.elapsed().as_millis();
            error!(
                timeout_ms = ?timeout_duration.as_millis(),
                duration_ms = %duration_ms,
                error_type = "timeout",
                "Request timeout exceeded"
            );
            Err(GatewayError::Timeout)
        }
    }
}

/// Inner gateway handler that processes all incoming requests
///
/// This handler:
/// 1. Routes the request to determine target service
/// 2. Queries auth policy from backend service (done in middleware)
/// 3. Applies authorization (done in middleware)
/// 4. Applies rate limiting
/// 5. Converts HTTP request to gRPC
/// 6. Calls backend service via gRPC client pool
/// 7. Converts gRPC response to HTTP
/// 8. Handles errors and maps to appropriate HTTP status codes
/// 9. Emits traces and metrics
async fn gateway_handler_inner(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request<Body>,
) -> Result<Response, GatewayError> {
    let start_time = Instant::now();
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();
    
    info!(
        path = %path,
        method = %method,
        client_ip = %addr.ip(),
        "Processing gateway request"
    );

    // Skip gateway processing for health and metrics endpoints
    if path == SYSTEM_PATH_HEALTH || path == SYSTEM_PATH_METRICS {
        debug!(path = %path, "Skipping gateway processing for system endpoint");
        return Err(GatewayError::NotFound);
    }

    // Apply rate limiting
    if let Some(rate_limiter) = &state.rate_limiter {
        if !RateLimiter::should_exclude_path(&path) {
            if let Err(e) = rate_limiter.check_rate_limit(addr.ip()).await {
                let duration_ms = start_time.elapsed().as_millis();
                warn!(
                    client_ip = %addr.ip(),
                    path = %path,
                    method = %method,
                    duration_ms = %duration_ms,
                    error_type = "rate_limit",
                    error = %e,
                    "Rate limit exceeded"
                );
                
                state.metrics.rate_limit_counter.inc();
                
                return Err(GatewayError::RateLimitExceeded);
            }
        }
    }

    // Get routing decision from request extensions (set by auth middleware)
    // Performance: Retrieve Arc<RoutingDecision> to avoid cloning the data.
    // Cloning Arc is cheap (atomic reference count increment) vs cloning the entire struct.
    let routing_decision = request.extensions().get::<Arc<crate::router::RoutingDecision>>()
        .cloned()
        .ok_or_else(|| {
            let duration_ms = start_time.elapsed().as_millis();
            error!(
                path = %path,
                method = %method,
                duration_ms = %duration_ms,
                error_type = "internal",
                "Routing decision not found in request extensions (auth middleware may have failed)"
            );

            state.metrics.request_counter
                .with_label_values(&[path.as_str(), method.as_str(), "500"])
                .inc();

            GatewayError::InternalError("Missing routing decision".to_string())
        })?;

    debug!(
        service = %routing_decision.service,
        grpc_method = %routing_decision.grpc_method,
        path_params = ?routing_decision.path_params,
        "Request routed to backend service"
    );

    // Get auth context from request extensions (set by auth middleware)
    let auth_context = request.extensions().get::<AuthContext>().cloned();

    // Get channel for the backend service
    let service_channel = match state.grpc_pool.get_channel(&routing_decision.service) {
        Ok(channel) => channel,
        Err(e) => {
            error!(
                service = %routing_decision.service,
                error = %e,
                "Failed to get service channel"
            );
            
            state.metrics.request_counter
                .with_label_values(&[path.as_str(), method.as_str(), "503"])
                .inc();
            
            return Err(GatewayError::ServiceUnavailable(routing_decision.service.to_string()));
        }
    };

    // Convert HTTP request to gRPC format
    let grpc_request = match convert_http_to_grpc(
        request,
        &routing_decision.grpc_method,
        &routing_decision.path_params,
        &headers,
        auth_context.as_ref(),
    )
    .await
    {
        Ok(req) => req,
        Err(e) => {
            error!(
                service = %routing_decision.service,
                error = %e,
                "Failed to convert HTTP request to gRPC"
            );
            
            state.metrics.request_counter
                .with_label_values(&[path.as_str(), method.as_str(), "400"])
                .inc();
            
            return Err(e);
        }
    };

    debug!(
        service = %routing_decision.service,
        grpc_method = %routing_decision.grpc_method,
        payload_size = grpc_request.len(),
        "Calling backend service via gRPC"
    );

    // Call backend service via gRPC with circuit breaker protection
    let grpc_response = if let Some(circuit_breaker) = state.grpc_pool.get_circuit_breaker(&routing_decision.service) {
        // Use circuit breaker for this service
        match circuit_breaker.call(call_backend_service(
            service_channel.clone(),
            &routing_decision.grpc_method,
            grpc_request.clone(),
        )).await {
            Ok(resp) => resp,
            Err(crate::circuit_breaker::CircuitBreakerError::Open) => {
                warn!(
                    service = %routing_decision.service,
                    "Circuit breaker is OPEN - rejecting request"
                );

                state.metrics.request_counter
                    .with_label_values(&[path.as_str(), method.as_str(), "503"])
                    .inc();

                state.metrics.grpc_call_counter
                    .with_label_values(&[routing_decision.service.as_ref(), routing_decision.grpc_method.as_ref(), "circuit_open"])
                    .inc();

                return Err(GatewayError::ServiceUnavailable(
                    format!("Service {} is currently unavailable (circuit breaker open)", &*routing_decision.service)
                ));
            }
            Err(crate::circuit_breaker::CircuitBreakerError::OperationFailed(e)) => {
                let duration_ms = start_time.elapsed().as_millis();
                error!(
                    service = %routing_decision.service,
                    grpc_method = %routing_decision.grpc_method,
                    path = %path,
                    method = %method,
                    duration_ms = %duration_ms,
                    error_type = "grpc_error",
                    error = %e,
                    "Backend service call failed"
                );

                state.metrics.grpc_call_counter
                    .with_label_values(&[routing_decision.service.as_ref(), routing_decision.grpc_method.as_ref(), "error"])
                    .inc();

                return Err(GatewayError::GrpcCallFailed(e));
            }
        }
    } else {
        // No circuit breaker configured, call directly
        match call_backend_service(
            service_channel,
            &routing_decision.grpc_method,
            grpc_request,
        )
        .await
        {
            Ok(resp) => resp,
            Err(e) => {
                error!(
                    service = %routing_decision.service,
                    grpc_method = %routing_decision.grpc_method,
                    error = %e,
                    "Backend service call failed"
                );

                let status_code = map_grpc_error_to_status(&e);

                let status_str = status_code.as_u16().to_string();
                state.metrics.request_counter
                    .with_label_values(&[path.as_str(), method.as_str(), &status_str])
                    .inc();

                state.metrics.grpc_call_counter
                    .with_label_values(&[routing_decision.service.as_ref(), routing_decision.grpc_method.as_ref(), "error"])
                    .inc();

                return Err(e);
            }
        }
    };

    debug!(
        service = %routing_decision.service,
        response_size = grpc_response.len(),
        "Received response from backend service"
    );

    // Convert gRPC response to HTTP
    let http_response = match convert_grpc_to_http(grpc_response).await {
        Ok(resp) => resp,
        Err(e) => {
            error!(
                service = %routing_decision.service,
                error = %e,
                "Failed to convert gRPC response to HTTP"
            );
            
            state.metrics.request_counter
                .with_label_values(&[path.as_str(), method.as_str(), "500"])
                .inc();
            
            return Err(e);
        }
    };

    // Record metrics
    let duration = start_time.elapsed();
    
    state.metrics.request_duration
        .with_label_values(&[path.as_str(), method.as_str()])
        .observe(duration.as_secs_f64());
    
    state.metrics.request_counter
        .with_label_values(&[path.as_str(), method.as_str(), "200"])
        .inc();
    
    state.metrics.grpc_call_counter
        .with_label_values(&[routing_decision.service.as_ref(), routing_decision.grpc_method.as_ref(), "success"])
        .inc();

    info!(
        path = %path,
        method = %method,
        service = %routing_decision.service,
        duration_ms = duration.as_millis(),
        "Request completed successfully"
    );

    Ok(http_response)
}

/// Convert HTTP request to gRPC format
///
/// This function:
/// - Validates and sanitizes path parameters
/// - Extracts the request body
/// - Includes path parameters in the payload
/// - Propagates trace headers as gRPC metadata
/// - Includes auth context in metadata
pub(crate) async fn convert_http_to_grpc(
    request: Request<Body>,
    grpc_method: &str,
    path_params: &std::collections::HashMap<String, String>,
    headers: &HeaderMap,
    auth_context: Option<&AuthContext>,
) -> Result<Vec<u8>, GatewayError> {
    use axum::body::to_bytes;
    use crate::security::PathValidator;
    
    debug!(
        grpc_method = %grpc_method,
        path_params = ?path_params,
        "Converting HTTP request to gRPC"
    );

    // Validate and sanitize path parameters to prevent directory traversal attacks
    let sanitized_params = if !path_params.is_empty() {
        PathValidator::sanitize_path_params(path_params)
            .map_err(|e| {
                error!(error = %e, "Path parameter validation failed");
                GatewayError::ConversionError(format!("Invalid path parameter: {}", e))
            })?
    } else {
        path_params.clone()
    };

    // Extract request body with size limit to prevent memory exhaustion attacks
    let body_bytes = to_bytes(request.into_body(), super::constants::MAX_REQUEST_BODY_SIZE)
        .await
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_READ_BODY, e)))?;

    // Parse JSON body if present
    let mut payload: serde_json::Value = if body_bytes.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body_bytes)
            .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_INVALID_JSON, e)))?
    };

    // Merge sanitized path parameters into payload
    if !sanitized_params.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            for (key, value) in &sanitized_params {
                obj.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }
    }

    // Add auth context to payload if present
    if let Some(ctx) = auth_context {
        if ctx.authenticated {
            if let Some(obj) = payload.as_object_mut() {
                if let Some(user_id) = &ctx.user_id {
                    obj.insert(METADATA_AUTH_USER_ID.to_string(), serde_json::Value::String(user_id.clone()));
                }
                if !ctx.roles.is_empty() {
                    obj.insert(METADATA_AUTH_ROLES.to_string(), serde_json::json!(ctx.roles));
                }
            }
        }
    }

    // Note: Trace headers are now extracted and injected into gRPC metadata
    // by the DynamicGrpcClient, not added to the payload
    // This ensures proper W3C trace context propagation

    // Serialize to bytes
    let payload_bytes = serde_json::to_vec(&payload)
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_SERIALIZE_PAYLOAD, e)))?;

    debug!(
        grpc_method = %grpc_method,
        payload_size = payload_bytes.len(),
        "HTTP request converted to gRPC format"
    );

    Ok(payload_bytes)
}

/// Convert gRPC response to HTTP format
pub(crate) async fn convert_grpc_to_http(grpc_response: Vec<u8>) -> Result<Response, GatewayError> {
    debug!(
        response_size = grpc_response.len(),
        "Converting gRPC response to HTTP"
    );

    // Parse the response as JSON
    let json_value: serde_json::Value = serde_json::from_slice(&grpc_response)
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_PARSE_GRPC_RESPONSE, e)))?;

    // Create HTTP response with JSON body
    let response = (StatusCode::OK, Json(json_value)).into_response();

    debug!("gRPC response converted to HTTP successfully");

    Ok(response)
}

/// Call backend service via gRPC using dynamic client
///
/// This function:
/// 1. Routes to appropriate typed client based on service
/// 2. Makes the actual gRPC call with proper protobuf types
/// 3. Returns the response
pub(crate) async fn call_backend_service(
    channel: tonic::transport::Channel,
    grpc_method: &str,
    request_payload: Vec<u8>,
) -> Result<Vec<u8>, GatewayError> {
    debug!(
        grpc_method = %grpc_method,
        payload_size = request_payload.len(),
        "Calling backend service with typed client"
    );

    // Extract service name from grpc_method (format: "service.ServiceName/Method")
    let service_name = grpc_method
        .split('/')
        .next()
        .ok_or_else(|| GatewayError::ConversionError("Invalid gRPC method format".to_string()))?;

    let method_name = grpc_method
        .split('/')
        .nth(1)
        .ok_or_else(|| GatewayError::ConversionError("Invalid gRPC method format".to_string()))?;

    // Route to appropriate typed client based on service
    match service_name {
        "user.UserService" => {
            use crate::handlers::user_service::call_user_service;
            call_user_service(channel, method_name, request_payload).await
        }
        _ => {
            // Fall back to dynamic client for other services
            use crate::grpc::DynamicGrpcClient;
            use std::collections::HashMap;

            debug!(
                service = %service_name,
                "Using dynamic client for non-user service"
            );

            let mut trace_headers = HashMap::new();
            let mut clean_payload = request_payload.clone();

            if let Ok(mut json_value) = serde_json::from_slice::<serde_json::Value>(&request_payload) {
                if let Some(obj) = json_value.as_object_mut() {
                    if let Some(trace_data) = obj.remove(METADATA_TRACE) {
                        if let Some(trace_obj) = trace_data.as_object() {
                            for (key, value) in trace_obj {
                                if let Some(value_str) = value.as_str() {
                                    trace_headers.insert(key.clone(), value_str.to_string());
                                }
                            }
                        }
                    }
                    clean_payload = serde_json::to_vec(&json_value)
                        .map_err(|e| GatewayError::ConversionError(format!("Failed to serialize clean payload: {}", e)))?;
                }
            }

            let mut client = DynamicGrpcClient::new(channel, service_name.to_string());
            client
                .call(method_name, clean_payload, trace_headers)
                .await
                .map_err(|e| {
                    error!(
                        grpc_method = %grpc_method,
                        error = %e,
                        "Dynamic gRPC call failed"
                    );
                    GatewayError::GrpcCallFailed(e.to_string())
                })
        }
    }
}


