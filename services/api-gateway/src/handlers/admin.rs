use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{error, info};

use crate::shared::state::AppState;

/// Response for the refresh routes endpoint
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub success: bool,
    pub routes_discovered: usize,
    pub services_queried: usize,
    pub errors: Vec<String>,
}

/// Handler for POST /admin/refresh-routes
///
/// Triggers immediate route discovery for all services and updates the router.
/// Requires authentication (enforced by auth middleware).
///
/// # Returns
/// * `200 OK` with RefreshResponse on success
/// * `500 Internal Server Error` on failure
pub async fn refresh_routes_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<RefreshResponse>, RefreshError> {
    info!("Admin refresh routes endpoint called");

    // Get discovery service from app state
    let discovery_service = state
        .discovery_service
        .as_ref()
        .ok_or_else(|| {
            error!("Discovery service not available in app state");
            RefreshError::ServiceUnavailable("Route discovery is not enabled".to_string())
        })?;

    // Discover routes from all services
    let services_queried = state
        .config
        .services
        .values()
        .filter(|s| s.auto_discover)
        .count();

    info!(
        services = services_queried,
        "Starting manual route discovery"
    );

    let mut errors = Vec::new();

    // Perform discovery
    let discovered_routes = match discovery_service.discover_routes(&state.config.services).await {
        Ok(routes) => {
            info!(
                routes = routes.len(),
                "Successfully discovered routes from all services"
            );
            routes
        }
        Err(e) => {
            error!(error = %e, "Failed to discover routes");
            errors.push(format!("Discovery failed: {}", e));
            
            // Return error response
            return Ok(Json(RefreshResponse {
                success: false,
                routes_discovered: 0,
                services_queried,
                errors,
            }));
        }
    };

    // Apply route overrides
    let final_routes = discovery_service.apply_overrides(
        discovered_routes,
        &state.config.route_overrides,
    );

    let routes_count = final_routes.len();

    // Update router with new routes
    let mut router_guard = state.router_lock.write().await;
    router_guard.update_routes(final_routes);
    drop(router_guard);

    info!(
        routes = routes_count,
        services = services_queried,
        "Successfully refreshed routes via admin endpoint"
    );

    Ok(Json(RefreshResponse {
        success: true,
        routes_discovered: routes_count,
        services_queried,
        errors,
    }))
}

/// Error type for admin refresh endpoint
#[derive(Debug)]
pub enum RefreshError {
    ServiceUnavailable(String),
}

impl IntoResponse for RefreshError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            RefreshError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
        };

        let body = Json(RefreshResponse {
            success: false,
            routes_discovered: 0,
            services_queried: 0,
            errors: vec![message],
        });

        (status, body).into_response()
    }
}
