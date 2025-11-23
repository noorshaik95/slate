//! Request router for matching HTTP requests to backend gRPC services.
//!
//! # Thread-Safe Dynamic Updates
//!
//! The router supports dynamic route updates for auto-discovery scenarios.
//! When using with periodic refresh, wrap the router in `Arc<RwLock<RequestRouter>>`:
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! // Create router wrapped in Arc<RwLock<>> for thread-safe updates
//! let router = Arc::new(RwLock::new(RequestRouter::new(initial_routes)));
//!
//! // Reading (concurrent reads are allowed)
//! let router_guard = router.read().await;
//! let decision = router_guard.route(path, method)?;
//! drop(router_guard);
//!
//! // Updating (exclusive write lock)
//! let mut router_guard = router.write().await;
//! router_guard.update_routes(new_routes);
//! drop(router_guard);
//! ```

mod decision;
mod matcher;
mod pattern;

#[cfg(test)]
mod tests;

pub use decision::{RouterError, RoutingDecision};

use std::collections::HashMap;

use crate::config::RouteConfig;
use matcher::RouteMatcher;

/// Request router that matches HTTP requests to backend services.
pub struct RequestRouter {
    matcher: RouteMatcher,
    // Track which routes belong to which service for partial refresh
    service_route_map: HashMap<String, Vec<RouteConfig>>,
}

impl RequestRouter {
    /// Create a new router from route configurations.
    pub fn new(routes: Vec<RouteConfig>) -> Self {
        let mut router = Self {
            matcher: RouteMatcher::new(),
            service_route_map: HashMap::new(),
        };

        router.update_routes(routes);
        router
    }

    /// Update routes dynamically (thread-safe when wrapped in Arc<RwLock<>>).
    ///
    /// Replaces entire route table with new routes.
    pub fn update_routes(&mut self, routes: Vec<RouteConfig>) {
        // Clear existing routes
        self.matcher.clear();
        self.service_route_map.clear();

        // Build service route map
        for route in &routes {
            self.service_route_map
                .entry(route.service.clone())
                .or_default()
                .push(route.clone());
        }

        // Add routes to matcher
        self.matcher.add_routes(routes);
    }

    /// Get routes for a specific service (for partial refresh).
    pub fn get_routes_for_service(&self, service: &str) -> Vec<RouteConfig> {
        self.service_route_map
            .get(service)
            .cloned()
            .unwrap_or_default()
    }

    /// Get current route count.
    #[allow(dead_code)]
    pub fn route_count(&self) -> usize {
        self.matcher.route_count()
    }

    /// Get all routes (for admin endpoint).
    #[allow(dead_code)]
    pub fn get_all_routes(&self) -> Vec<RouteConfig> {
        self.matcher.get_all_routes()
    }

    /// Route an incoming request to the appropriate backend service.
    pub fn route(&self, path: &str, method: &str) -> Result<RoutingDecision, RouterError> {
        self.matcher.match_route(path, method)
    }
}
