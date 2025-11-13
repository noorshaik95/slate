use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use crate::auth::middleware::AuthContext;
use crate::rate_limit::RateLimiter;
use crate::shared::state::AppState;

use super::constants::*;
use super::types::{GatewayError, map_grpc_error_to_status};

/// Main gateway handler that processes all incoming requests
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
pub async fn gateway_handler(
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
                warn!(
                    client_ip = %addr.ip(),
                    path = %path,
                    error = %e,
                    "Rate limit exceeded"
                );
                
                state.metrics.rate_limit_counter.inc();
                
                return Err(GatewayError::RateLimitExceeded);
            }
        }
    }

    // Route the request to determine target service
    // Use RwLock-wrapped router if available (for dynamic updates), otherwise use static router
    let routing_decision = if let Some(router_lock) = &state.router_lock {
        // Dynamic routing with RwLock (allows concurrent reads)
        let router_guard = router_lock.read().await;
        let result = router_guard.route(&path, &method);
        drop(router_guard);
        
        match result {
            Ok(decision) => decision,
            Err(e) => {
                warn!(
                    path = %path,
                    method = %method,
                    error = %e,
                    "Route not found"
                );
                
                state.metrics.request_counter
                    .with_label_values(&[path.as_str(), method.as_str(), "404"])
                    .inc();
                
                return Err(GatewayError::RouteNotFound(e));
            }
        }
    } else {
        // Static routing (no dynamic updates)
        match state.router.route(&path, &method) {
            Ok(decision) => decision,
            Err(e) => {
                warn!(
                    path = %path,
                    method = %method,
                    error = %e,
                    "Route not found"
                );
                
                state.metrics.request_counter
                    .with_label_values(&[path.as_str(), method.as_str(), "404"])
                    .inc();
                
                return Err(GatewayError::RouteNotFound(e));
            }
        }
    };

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
            
            return Err(GatewayError::ServiceUnavailable(routing_decision.service.clone()));
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

    // Call backend service via gRPC
    // Note: For now, we'll use a placeholder since we need service-specific clients
    // In a real implementation, this would use the generated proto clients
    let grpc_response = match call_backend_service(
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
                .with_label_values(&[routing_decision.service.as_str(), routing_decision.grpc_method.as_str(), "error"])
                .inc();
            
            return Err(e);
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
        .with_label_values(&[routing_decision.service.as_str(), routing_decision.grpc_method.as_str(), "success"])
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
    
    debug!(
        grpc_method = %grpc_method,
        path_params = ?path_params,
        "Converting HTTP request to gRPC"
    );

    // Extract request body
    let body_bytes = to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_READ_BODY, e)))?;

    // Parse JSON body if present
    let mut payload: serde_json::Value = if body_bytes.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body_bytes)
            .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_INVALID_JSON, e)))?
    };

    // Merge path parameters into payload
    if !path_params.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            for (key, value) in path_params {
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

    // Extract trace headers for propagation
    let mut trace_metadata = std::collections::HashMap::new();
    for header_name in TRACE_HEADERS {
        if let Some(value) = headers.get(*header_name) {
            if let Ok(value_str) = value.to_str() {
                trace_metadata.insert(header_name.to_string(), value_str.to_string());
            }
        }
    }

    // Add trace metadata to payload for now
    // In a real implementation, this would be sent as gRPC metadata
    if !trace_metadata.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(METADATA_TRACE.to_string(), serde_json::json!(trace_metadata));
        }
    }

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

/// Call backend service via gRPC
///
/// Note: This is a placeholder implementation. In a real system, this would:
/// 1. Use the service-specific generated proto clients
/// 2. Make actual gRPC calls with proper request/response types
/// 3. Handle streaming if needed
///
/// For now, we'll return a mock response to demonstrate the flow
pub(crate) async fn call_backend_service(
    _channel: tonic::transport::Channel,
    grpc_method: &str,
    _request_payload: Vec<u8>,
) -> Result<Vec<u8>, GatewayError> {
    debug!(
        grpc_method = %grpc_method,
        "Calling backend service (placeholder implementation)"
    );

    // TODO: Implement actual gRPC calls using service-specific clients
    // For now, return a placeholder response
    warn!(
        grpc_method = %grpc_method,
        ERR_MSG_PLACEHOLDER_GRPC
    );

    // Return a mock success response
    let mock_response = serde_json::json!({
        "success": true,
        "message": "Placeholder response - gRPC call not yet implemented",
        "method": grpc_method
    });

    let response_bytes = serde_json::to_vec(&mock_response)
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_SERIALIZE_MOCK, e)))?;

    Ok(response_bytes)
}


