use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::net::SocketAddr;
use tracing::{debug, warn};

/// Configuration for body size limits
#[derive(Clone, Debug)]
pub struct BodyLimitConfig {
    /// Default body size limit in bytes (1MB)
    pub default_limit: usize,
    /// Upload body size limit in bytes (10MB)
    pub upload_limit: usize,
    /// Paths that should use the upload limit
    pub upload_paths: Vec<String>,
}

impl Default for BodyLimitConfig {
    fn default() -> Self {
        Self {
            default_limit: 1024 * 1024,      // 1MB
            upload_limit: 10 * 1024 * 1024,  // 10MB
            upload_paths: vec![
                "/upload".to_string(),
                "/api/upload".to_string(),
            ],
        }
    }
}

impl BodyLimitConfig {
    /// Create a new body limit configuration
    pub fn new(default_limit: usize, upload_limit: usize, upload_paths: Vec<String>) -> Self {
        Self {
            default_limit,
            upload_limit,
            upload_paths,
        }
    }

    /// Get the appropriate limit for a given path
    pub fn get_limit_for_path(&self, path: &str) -> usize {
        if self.upload_paths.iter().any(|p| path.starts_with(p)) {
            self.upload_limit
        } else {
            self.default_limit
        }
    }
}

/// Layer for applying body size limits
#[derive(Clone)]
pub struct BodyLimitLayer {
    config: BodyLimitConfig,
}

impl BodyLimitLayer {
    /// Create a new body limit layer with the given configuration
    pub fn new(config: BodyLimitConfig) -> Self {
        Self { config }
    }
}

/// Middleware function to enforce body size limits
///
/// This middleware checks the Content-Length header and rejects requests
/// that exceed the configured limit for the path.
///
/// # Security
/// - Prevents DoS attacks via unlimited request body sizes
/// - Checks Content-Length before reading body to avoid memory exhaustion
/// - Different limits for upload vs regular endpoints
pub async fn body_limit_middleware(
    config: BodyLimitConfig,
    request: Request<Body>,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let path = request.uri().path();
    let limit = config.get_limit_for_path(path);

    // Extract client IP for logging
    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Check Content-Length header
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                if length > limit {
                    warn!(
                        path = %path,
                        client_ip = %client_ip,
                        content_length = length,
                        limit = limit,
                        "Request body size exceeds limit"
                    );

                    let error_body = json!({
                        "error": {
                            "message": format!(
                                "Request payload too large. Maximum allowed: {} bytes",
                                limit
                            ),
                            "status": 413,
                            "limit_bytes": limit,
                            "received_bytes": length,
                        }
                    });

                    return Err((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        axum::Json(error_body),
                    ));
                }

                debug!(
                    path = %path,
                    content_length = length,
                    limit = limit,
                    "Request body size within limit"
                );
            }
        }
    }

    Ok(next.run(request).await)
}
