use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::{debug, info};

use super::checker::HealthChecker;

/// HTTP handler for the /health endpoint (legacy, redirects to readiness)
pub async fn health_handler(
    State(health_checker): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    readiness_handler(State(health_checker)).await
}

/// HTTP handler for the /health/live endpoint
///
/// Liveness probe - returns 200 OK if the application is running
/// This endpoint always returns success if the server can respond
pub async fn liveness_handler() -> impl IntoResponse {
    debug!("Liveness probe check");
    
    let response = json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    (StatusCode::OK, Json(response))
}

/// HTTP handler for the /health/ready endpoint
///
/// Readiness probe - returns 200 OK if the application is ready to serve traffic
/// Checks connectivity to all backend services
pub async fn readiness_handler(
    State(health_checker): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    debug!("Readiness probe check");

    // Check health of all backend services with timeout
    let health_check_result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        health_checker.check_health()
    ).await;

    match health_check_result {
        Ok(health_status) => {
            if health_status.healthy {
                info!("Readiness check passed - all services healthy");
                (StatusCode::OK, Json(json!({
                    "status": "ready",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "services": health_status.services,
                })))
            } else {
                info!("Readiness check failed - some services unhealthy");
                (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
                    "status": "not_ready",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "services": health_status.services,
                    "message": "One or more backend services are unavailable",
                })))
            }
        }
        Err(_) => {
            info!("Readiness check timed out");
            (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
                "status": "timeout",
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": "Health check timed out after 2 seconds",
            })))
        }
    }
}
