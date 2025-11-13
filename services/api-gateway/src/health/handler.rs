use std::sync::Arc;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};

use super::checker::HealthChecker;

/// HTTP handler for the /health endpoint
pub async fn health_handler(
    State(health_checker): State<Arc<HealthChecker>>,
) -> impl IntoResponse {
    let health_status = health_checker.check_health().await;

    if health_status.healthy {
        (StatusCode::OK, Json(health_status))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(health_status))
    }
}
