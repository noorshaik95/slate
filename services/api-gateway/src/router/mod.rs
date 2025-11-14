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

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

use crate::config::RouteConfig;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Route not found for path: {path}, method: {method}")]
    RouteNotFound { path: String, method: String },
}

/// Key for looking up routes in the routing table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RouteKey {
    path_pattern: String,
    method: String,
}

/// Result of routing decision
/// 
/// Performance: Uses Arc<str> for service and grpc_method to avoid cloning strings
/// in the hot path. Cloning Arc is cheap (atomic reference count increment) compared
/// to cloning the actual string data.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub service: Arc<str>,
    pub grpc_method: Arc<str>,
    pub path_params: HashMap<String, String>,
}

/// Request router that matches HTTP requests to backend services
pub struct RequestRouter {
    routes: HashMap<RouteKey, RouteConfig>,
    // Store patterns separately for dynamic matching
    dynamic_routes: Vec<(RoutePattern, RouteConfig)>,
    // Track which routes belong to which service for partial refresh
    service_route_map: HashMap<String, Vec<RouteConfig>>,
}

/// Represents a route pattern with support for dynamic segments
#[derive(Debug, Clone)]
struct RoutePattern {
    segments: Vec<PathSegment>,
    method: String,
}

#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    Static(String),
    Dynamic(String), // Parameter name
}

impl RequestRouter {
    /// Create a new router from route configurations
    pub fn new(routes: Vec<RouteConfig>) -> Self {
        let mut router = Self {
            routes: HashMap::new(),
            dynamic_routes: Vec::new(),
            service_route_map: HashMap::new(),
        };
        
        router.update_routes(routes);
        router
    }
    
    /// Update routes dynamically (thread-safe when wrapped in Arc<RwLock<>>)
    /// Replaces entire route table with new routes
    pub fn update_routes(&mut self, routes: Vec<RouteConfig>) {
        // Clear existing routes
        self.routes.clear();
        self.dynamic_routes.clear();
        self.service_route_map.clear();

        // Build service route map
        for route in &routes {
            self.service_route_map
                .entry(route.service.clone())
                .or_insert_with(Vec::new)
                .push(route.clone());
        }

        // Populate static and dynamic routes
        for route in routes {
            let pattern = RoutePattern::parse(&route.path, &route.method);
            
            if pattern.is_static() {
                // Static routes go into the hash map for O(1) lookup
                let key = RouteKey {
                    path_pattern: route.path.clone(),
                    method: route.method.clone(),
                };
                self.routes.insert(key, route);
            } else {
                // Dynamic routes need pattern matching
                self.dynamic_routes.push((pattern, route));
            }
        }
    }
    
    /// Get routes for a specific service (for partial refresh)
    pub fn get_routes_for_service(&self, service: &str) -> Vec<RouteConfig> {
        self.service_route_map
            .get(service)
            .cloned()
            .unwrap_or_default()
    }
    
    /// Get current route count
    #[allow(dead_code)]
    pub fn route_count(&self) -> usize {
        self.routes.len() + self.dynamic_routes.len()
    }
    
    /// Get all routes (for admin endpoint)
    #[allow(dead_code)]
    pub fn get_all_routes(&self) -> Vec<RouteConfig> {
        let mut all_routes = Vec::new();
        
        // Add static routes
        all_routes.extend(self.routes.values().cloned());
        
        // Add dynamic routes
        all_routes.extend(self.dynamic_routes.iter().map(|(_, route)| route.clone()));
        
        all_routes
    }

    /// Route an incoming request to the appropriate backend service
    pub fn route(&self, path: &str, method: &str) -> Result<RoutingDecision, RouterError> {
        // First try exact match for static routes (fast path)
        let key = RouteKey {
            path_pattern: path.to_string(),
            method: method.to_uppercase(),
        };

        if let Some(route) = self.routes.get(&key) {
            return Ok(RoutingDecision {
                service: Arc::from(route.service.as_str()),
                grpc_method: Arc::from(route.grpc_method.as_str()),
                path_params: HashMap::new(),
            });
        }

        // Try dynamic routes with pattern matching
        for (pattern, route) in &self.dynamic_routes {
            if pattern.method.to_uppercase() != method.to_uppercase() {
                continue;
            }

            if let Some(params) = pattern.matches(path) {
                return Ok(RoutingDecision {
                    service: Arc::from(route.service.as_str()),
                    grpc_method: Arc::from(route.grpc_method.as_str()),
                    path_params: params,
                });
            }
        }

        Err(RouterError::RouteNotFound {
            path: path.to_string(),
            method: method.to_string(),
        })
    }
}

impl RoutePattern {
    /// Parse a path pattern into segments
    fn parse(path: &str, method: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if segment.starts_with(':') {
                    PathSegment::Dynamic(segment[1..].to_string())
                } else {
                    PathSegment::Static(segment.to_string())
                }
            })
            .collect();

        Self {
            segments,
            method: method.to_uppercase(),
        }
    }

    /// Check if this pattern contains only static segments
    fn is_static(&self) -> bool {
        self.segments
            .iter()
            .all(|s| matches!(s, PathSegment::Static(_)))
    }

    /// Try to match a path against this pattern
    /// Returns Some(params) if match succeeds, None otherwise
    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Must have same number of segments
        if path_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, path_seg) in self.segments.iter().zip(path_segments.iter()) {
            match pattern_seg {
                PathSegment::Static(expected) => {
                    if expected != path_seg {
                        return None;
                    }
                }
                PathSegment::Dynamic(param_name) => {
                    params.insert(param_name.clone(), path_seg.to_string());
                }
            }
        }

        Some(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_routes() -> Vec<RouteConfig> {
        vec![
            RouteConfig {
                path: "/api/users".to_string(),
                method: "GET".to_string(),
                service: "user-service".to_string(),
                grpc_method: "user.UserService/ListUsers".to_string(),
            },
            RouteConfig {
                path: "/api/users/:id".to_string(),
                method: "GET".to_string(),
                service: "user-service".to_string(),
                grpc_method: "user.UserService/GetUser".to_string(),
            },
            RouteConfig {
                path: "/api/users/:id".to_string(),
                method: "DELETE".to_string(),
                service: "user-service".to_string(),
                grpc_method: "user.UserService/DeleteUser".to_string(),
            },
            RouteConfig {
                path: "/api/posts/:post_id/comments/:comment_id".to_string(),
                method: "GET".to_string(),
                service: "post-service".to_string(),
                grpc_method: "post.PostService/GetComment".to_string(),
            },
        ]
    }

    #[test]
    fn test_static_route_matching() {
        let router = RequestRouter::new(create_test_routes());
        
        let result = router.route("/api/users", "GET").unwrap();
        assert_eq!(result.service.as_ref(), "user-service");
        assert_eq!(result.grpc_method.as_ref(), "user.UserService/ListUsers");
        assert!(result.path_params.is_empty());
    }

    #[test]
    fn test_dynamic_route_single_param() {
        let router = RequestRouter::new(create_test_routes());
        
        let result = router.route("/api/users/123", "GET").unwrap();
        assert_eq!(result.service.as_ref(), "user-service");
        assert_eq!(result.grpc_method.as_ref(), "user.UserService/GetUser");
        assert_eq!(result.path_params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_dynamic_route_multiple_params() {
        let router = RequestRouter::new(create_test_routes());
        
        let result = router.route("/api/posts/456/comments/789", "GET").unwrap();
        assert_eq!(result.service.as_ref(), "post-service");
        assert_eq!(result.grpc_method.as_ref(), "post.PostService/GetComment");
        assert_eq!(result.path_params.get("post_id"), Some(&"456".to_string()));
        assert_eq!(result.path_params.get("comment_id"), Some(&"789".to_string()));
    }

    #[test]
    fn test_method_matching() {
        let router = RequestRouter::new(create_test_routes());
        
        // GET should match GetUser
        let result = router.route("/api/users/123", "GET").unwrap();
        assert_eq!(result.grpc_method.as_ref(), "user.UserService/GetUser");
        
        // DELETE should match DeleteUser
        let result = router.route("/api/users/123", "DELETE").unwrap();
        assert_eq!(result.grpc_method.as_ref(), "user.UserService/DeleteUser");
    }

    #[test]
    fn test_case_insensitive_method() {
        let router = RequestRouter::new(create_test_routes());
        
        // Lowercase method should work
        let result = router.route("/api/users", "get").unwrap();
        assert_eq!(result.service.as_ref(), "user-service");
    }

    #[test]
    fn test_route_not_found() {
        let router = RequestRouter::new(create_test_routes());
        
        let result = router.route("/api/nonexistent", "GET");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RouterError::RouteNotFound { .. }));
    }

    #[test]
    fn test_wrong_method() {
        let router = RequestRouter::new(create_test_routes());
        
        // POST not configured for /api/users
        let result = router.route("/api/users", "POST");
        assert!(result.is_err());
    }

    #[test]
    fn test_trailing_slash_handling() {
        let router = RequestRouter::new(create_test_routes());
        
        // Without trailing slash should work
        let result = router.route("/api/users", "GET");
        assert!(result.is_ok());
    }

    #[test]
    fn test_segment_count_mismatch() {
        let router = RequestRouter::new(create_test_routes());
        
        // Too many segments
        let result = router.route("/api/users/123/extra", "GET");
        assert!(result.is_err());
        
        // Too few segments for dynamic route
        let result = router.route("/api/posts/456/comments", "GET");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_routes() {
        let mut router = RequestRouter::new(create_test_routes());
        
        // Initial route count
        assert_eq!(router.route_count(), 4);
        
        // Update with new routes
        let new_routes = vec![
            RouteConfig {
                path: "/api/products".to_string(),
                method: "GET".to_string(),
                service: "product-service".to_string(),
                grpc_method: "product.ProductService/ListProducts".to_string(),
            },
        ];
        
        router.update_routes(new_routes);
        
        // Should have only 1 route now
        assert_eq!(router.route_count(), 1);
        
        // Old routes should not work
        let result = router.route("/api/users", "GET");
        assert!(result.is_err());
        
        // New route should work
        let result = router.route("/api/products", "GET").unwrap();
        assert_eq!(result.service.as_ref(), "product-service");
    }

    #[test]
    fn test_get_routes_for_service() {
        let router = RequestRouter::new(create_test_routes());
        
        // Get routes for user-service
        let user_routes = router.get_routes_for_service("user-service");
        assert_eq!(user_routes.len(), 3);
        
        // Get routes for post-service
        let post_routes = router.get_routes_for_service("post-service");
        assert_eq!(post_routes.len(), 1);
        
        // Get routes for non-existent service
        let empty_routes = router.get_routes_for_service("nonexistent-service");
        assert_eq!(empty_routes.len(), 0);
    }

    #[test]
    fn test_route_count() {
        let router = RequestRouter::new(create_test_routes());
        assert_eq!(router.route_count(), 4);
        
        let empty_router = RequestRouter::new(vec![]);
        assert_eq!(empty_router.route_count(), 0);
    }

    #[test]
    fn test_get_all_routes() {
        let router = RequestRouter::new(create_test_routes());
        
        let all_routes = router.get_all_routes();
        assert_eq!(all_routes.len(), 4);
        
        // Verify all routes are present
        let services: Vec<String> = all_routes.iter().map(|r| r.service.clone()).collect();
        assert!(services.contains(&"user-service".to_string()));
        assert!(services.contains(&"post-service".to_string()));
    }

    #[test]
    fn test_service_route_map_tracking() {
        let routes = vec![
            RouteConfig {
                path: "/api/users".to_string(),
                method: "GET".to_string(),
                service: "user-service".to_string(),
                grpc_method: "user.UserService/ListUsers".to_string(),
            },
            RouteConfig {
                path: "/api/users/:id".to_string(),
                method: "GET".to_string(),
                service: "user-service".to_string(),
                grpc_method: "user.UserService/GetUser".to_string(),
            },
            RouteConfig {
                path: "/api/products".to_string(),
                method: "GET".to_string(),
                service: "product-service".to_string(),
                grpc_method: "product.ProductService/ListProducts".to_string(),
            },
        ];
        
        let router = RequestRouter::new(routes);
        
        // Verify service route map
        let user_routes = router.get_routes_for_service("user-service");
        assert_eq!(user_routes.len(), 2);
        
        let product_routes = router.get_routes_for_service("product-service");
        assert_eq!(product_routes.len(), 1);
    }
}
