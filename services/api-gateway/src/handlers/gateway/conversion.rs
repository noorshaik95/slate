//! HTTP to gRPC conversion utilities.
//!
//! Handles bidirectional conversion between HTTP and gRPC formats.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use tracing::debug;

use crate::auth::middleware::AuthContext;
use crate::handlers::constants::*;
use crate::handlers::types::GatewayError;
use crate::security::PathValidator;

/// Convert HTTP request to gRPC format.
///
/// This function:
/// - Validates and sanitizes path parameters
/// - Extracts the request body
/// - Includes path parameters in the payload
/// - Includes auth context in metadata
///
/// # Arguments
///
/// * `request` - The HTTP request
/// * `grpc_method` - The target gRPC method
/// * `path_params` - Path parameters extracted from the route
/// * `_headers` - HTTP headers (currently unused)
/// * `auth_context` - Authentication context if available
///
/// # Returns
///
/// JSON-encoded bytes ready for gRPC transmission
pub async fn convert_http_to_grpc(
    request: Request<Body>,
    grpc_method: &str,
    path_params: &std::collections::HashMap<String, String>,
    _headers: &HeaderMap,
    auth_context: Option<&AuthContext>,
) -> Result<Vec<u8>, GatewayError> {
    

    debug!(
        grpc_method = %grpc_method,
        path_params = ?path_params,
        "Converting HTTP request to gRPC"
    );

    // Validate and sanitize path parameters
    let sanitized_params = sanitize_path_params(path_params)?;

    // Extract request body
    let body_bytes = extract_body(request).await?;

    // Parse JSON body
    let mut payload = parse_json_body(&body_bytes)?;

    // Merge path parameters
    merge_path_params(&mut payload, &sanitized_params);

    // Add auth context
    add_auth_context(&mut payload, auth_context);

    // Serialize to bytes
    let payload_bytes = serde_json::to_vec(&payload).map_err(|e| {
        GatewayError::ConversionError(format!("{}: {}", ERR_MSG_SERIALIZE_PAYLOAD, e))
    })?;

    debug!(
        grpc_method = %grpc_method,
        payload_size = payload_bytes.len(),
        "HTTP request converted to gRPC format"
    );

    Ok(payload_bytes)
}

/// Convert gRPC response to HTTP format.
///
/// Parses the gRPC response as JSON and creates an HTTP response.
pub async fn convert_grpc_to_http(grpc_response: Vec<u8>) -> Result<Response, GatewayError> {
    debug!(
        response_size = grpc_response.len(),
        "Converting gRPC response to HTTP"
    );

    // Parse the response as JSON
    let json_value: serde_json::Value = serde_json::from_slice(&grpc_response).map_err(|e| {
        GatewayError::ConversionError(format!("{}: {}", ERR_MSG_PARSE_GRPC_RESPONSE, e))
    })?;

    // Create HTTP response with JSON body
    let response = (StatusCode::OK, Json(json_value)).into_response();

    debug!("gRPC response converted to HTTP successfully");

    Ok(response)
}

/// Sanitize path parameters to prevent security issues.
fn sanitize_path_params(
    path_params: &std::collections::HashMap<String, String>,
) -> Result<std::collections::HashMap<String, String>, GatewayError> {
    if path_params.is_empty() {
        return Ok(path_params.clone());
    }

    PathValidator::sanitize_path_params(path_params).map_err(|e| {
        tracing::error!(error = %e, "Path parameter validation failed");
        GatewayError::ConversionError(format!("Invalid path parameter: {}", e))
    })
}

/// Extract request body with size limit.
async fn extract_body(request: Request<Body>) -> Result<bytes::Bytes, GatewayError> {
    axum::body::to_bytes(request.into_body(), MAX_REQUEST_BODY_SIZE)
        .await
        .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_READ_BODY, e)))
}

/// Parse JSON body from bytes.
fn parse_json_body(body_bytes: &[u8]) -> Result<serde_json::Value, GatewayError> {
    if body_bytes.is_empty() {
        Ok(serde_json::json!({}))
    } else {
        serde_json::from_slice(body_bytes)
            .map_err(|e| GatewayError::ConversionError(format!("{}: {}", ERR_MSG_INVALID_JSON, e)))
    }
}

/// Merge path parameters into payload.
fn merge_path_params(
    payload: &mut serde_json::Value,
    sanitized_params: &std::collections::HashMap<String, String>,
) {
    if !sanitized_params.is_empty() {
        if let Some(obj) = payload.as_object_mut() {
            for (key, value) in sanitized_params {
                obj.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }
    }
}

/// Add authentication context to payload.
fn add_auth_context(payload: &mut serde_json::Value, auth_context: Option<&AuthContext>) {
    if let Some(ctx) = auth_context {
        if ctx.authenticated {
            if let Some(obj) = payload.as_object_mut() {
                if let Some(user_id) = &ctx.user_id {
                    obj.insert(
                        METADATA_AUTH_USER_ID.to_string(),
                        serde_json::Value::String(user_id.clone()),
                    );
                }
                if !ctx.roles.is_empty() {
                    obj.insert(
                        METADATA_AUTH_ROLES.to_string(),
                        serde_json::json!(ctx.roles),
                    );
                }
            }
        }
    }
}
