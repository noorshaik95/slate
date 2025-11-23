// Integration tests for the request router module

use std::collections::HashMap;

// Since we're testing a binary crate, we need to duplicate the necessary types
// In a real scenario, these would be in a library crate

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub method: String,
    pub service: String,
    pub grpc_method: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RouteKey {
    path_pattern: String,
    method: String,
}

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub service: String,
    pub grpc_method: String,
    pub path_params: HashMap<String, String>,
}

pub struct RequestRouter {
    routes: HashMap<RouteKey, RouteConfig>,
    dynamic_routes: Vec<(RoutePattern, RouteConfig)>,
}

#[derive(Debug, Clone)]
struct RoutePattern {
    segments: Vec<PathSegment>,
    method: String,
}

#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    Static(String),
    Dynamic(String),
}

impl RequestRouter {
    pub fn new(routes: Vec<RouteConfig>) -> Self {
        let mut static_routes = HashMap::new();
        let mut dynamic_routes = Vec::new();

        for route in routes {
            let pattern = RoutePattern::parse(&route.path, &route.method);

            if pattern.is_static() {
                let key = RouteKey {
                    path_pattern: route.path.clone(),
                    method: route.method.clone(),
                };
                static_routes.insert(key, route);
            } else {
                dynamic_routes.push((pattern, route));
            }
        }

        Self {
            routes: static_routes,
            dynamic_routes,
        }
    }

    pub fn route(&self, path: &str, method: &str) -> Option<RoutingDecision> {
        let key = RouteKey {
            path_pattern: path.to_string(),
            method: method.to_uppercase(),
        };

        if let Some(route) = self.routes.get(&key) {
            return Some(RoutingDecision {
                service: route.service.clone(),
                grpc_method: route.grpc_method.clone(),
                path_params: HashMap::new(),
            });
        }

        for (pattern, route) in &self.dynamic_routes {
            if pattern.method.to_uppercase() != method.to_uppercase() {
                continue;
            }

            if let Some(params) = pattern.matches(path) {
                return Some(RoutingDecision {
                    service: route.service.clone(),
                    grpc_method: route.grpc_method.clone(),
                    path_params: params,
                });
            }
        }

        None
    }
}

impl RoutePattern {
    fn parse(path: &str, method: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if let Some(stripped) = segment.strip_prefix(':') {
                    PathSegment::Dynamic(stripped.to_string())
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

    fn is_static(&self) -> bool {
        self.segments
            .iter()
            .all(|s| matches!(s, PathSegment::Static(_)))
    }

    fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

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
        assert_eq!(result.service, "user-service");
        assert_eq!(result.grpc_method, "user.UserService/ListUsers");
        assert!(result.path_params.is_empty());
    }

    #[test]
    fn test_dynamic_route_single_param() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/users/123", "GET").unwrap();
        assert_eq!(result.service, "user-service");
        assert_eq!(result.grpc_method, "user.UserService/GetUser");
        assert_eq!(result.path_params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_dynamic_route_multiple_params() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/posts/456/comments/789", "GET").unwrap();
        assert_eq!(result.service, "post-service");
        assert_eq!(result.grpc_method, "post.PostService/GetComment");
        assert_eq!(result.path_params.get("post_id"), Some(&"456".to_string()));
        assert_eq!(
            result.path_params.get("comment_id"),
            Some(&"789".to_string())
        );
    }

    #[test]
    fn test_method_matching() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/users/123", "GET").unwrap();
        assert_eq!(result.grpc_method, "user.UserService/GetUser");

        let result = router.route("/api/users/123", "DELETE").unwrap();
        assert_eq!(result.grpc_method, "user.UserService/DeleteUser");
    }

    #[test]
    fn test_case_insensitive_method() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/users", "get").unwrap();
        assert_eq!(result.service, "user-service");
    }

    #[test]
    fn test_route_not_found() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/nonexistent", "GET");
        assert!(result.is_none());
    }

    #[test]
    fn test_wrong_method() {
        let router = RequestRouter::new(create_test_routes());

        let result = router.route("/api/users", "POST");
        assert!(result.is_none());
    }
}
