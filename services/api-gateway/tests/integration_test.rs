// Integration tests for API Gateway
// Tests end-to-end request flow with mock gRPC services

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use api_gateway::{
    auth::{middleware::auth_middleware, AuthService},
    config::{AuthConfig, GatewayConfig, RateLimitConfig, RouteConfig, ServiceConfig},
    grpc::client::GrpcClientPool,
    handlers::gateway::gateway_handler,
    health::{health_handler, HealthChecker},
    rate_limit::RateLimiter,
    router::RequestRouter,
    shared::state::AppState,
};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::sleep;
use tonic::{transport::Server, Request as TonicRequest, Response as TonicResponse, Status};

// Include generated proto code
pub mod auth {
    tonic::include_proto!("auth");
}

pub mod gateway {
    tonic::include_proto!("gateway");
}

// Mock Auth Service Implementation
#[derive(Debug, Default)]
struct MockAuthService {
    // Configurable responses for testing
    valid_tokens: Arc<tokio::sync::RwLock<HashMap<String, (String, Vec<String>)>>>,
}

impl MockAuthService {
    fn new() -> Self {
        let mut valid_tokens = HashMap::new();
        // Add some default test tokens
        valid_tokens.insert(
            "valid_token".to_string(),
            ("user123".to_string(), vec!["user".to_string()]),
        );
        valid_tokens.insert(
            "admin_token".to_string(),
            ("admin456".to_string(), vec!["admin".to_string()]),
        );
        
        Self {
            valid_tokens: Arc::new(tokio::sync::RwLock::new(valid_tokens)),
        }
    }
}

#[tonic::async_trait]
impl auth::auth_service_server::AuthService for MockAuthService {
    async fn validate_token(
        &self,
        request: TonicRequest<auth::ValidateTokenRequest>,
    ) -> Result<TonicResponse<auth::ValidateTokenResponse>, Status> {
        let token = request.into_inner().token;
        let valid_tokens = self.valid_tokens.read().await;
        
        if let Some((user_id, roles)) = valid_tokens.get(&token) {
            Ok(TonicResponse::new(auth::ValidateTokenResponse {
                valid: true,
                user_id: user_id.clone(),
                roles: roles.clone(),
                error: String::new(),
            }))
        } else {
            Ok(TonicResponse::new(auth::ValidateTokenResponse {
                valid: false,
                user_id: String::new(),
                roles: vec![],
                error: "Invalid token".to_string(),
            }))
        }
    }
}

// Mock Backend Service with ServiceAuthPolicy Implementation
#[derive(Debug, Default)]
struct MockBackendService {
    // Configurable auth policies for different methods
    auth_policies: Arc<tokio::sync::RwLock<HashMap<String, (bool, Vec<String>)>>>,
}

impl MockBackendService {
    fn new() -> Self {
        let mut policies = HashMap::new();
        // Public endpoint - no auth required
        policies.insert(
            "user.UserService/GetPublicStatus".to_string(),
            (false, vec![]),
        );
        // Protected endpoint - auth required, no specific roles
        policies.insert(
            "user.UserService/ListUsers".to_string(),
            (true, vec![]),
        );
        // Admin-only endpoint
        policies.insert(
            "user.UserService/DeleteUser".to_string(),
            (true, vec!["admin".to_string()]),
        );
        
        Self {
            auth_policies: Arc::new(tokio::sync::RwLock::new(policies)),
        }
    }
}

#[tonic::async_trait]
impl gateway::service_auth_policy_server::ServiceAuthPolicy for MockBackendService {
    async fn get_auth_policy(
        &self,
        request: TonicRequest<gateway::AuthPolicyRequest>,
    ) -> Result<TonicResponse<gateway::AuthPolicyResponse>, Status> {
        let grpc_method = request.into_inner().grpc_method;
        let policies = self.auth_policies.read().await;
        
        if let Some((require_auth, required_roles)) = policies.get(&grpc_method) {
            Ok(TonicResponse::new(gateway::AuthPolicyResponse {
                require_auth: *require_auth,
                required_roles: required_roles.clone(),
                cache_ttl_seconds: 300,
            }))
        } else {
            // Default to requiring auth for unknown methods (fail-secure)
            Ok(TonicResponse::new(gateway::AuthPolicyResponse {
                require_auth: true,
                required_roles: vec![],
                cache_ttl_seconds: 60,
            }))
        }
    }
}

// Helper function to find an available port
async fn find_available_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    port
}

// Helper function to start mock gRPC servers with dynamic ports
async fn start_mock_servers() -> (String, String) {
    // Find available ports
    let auth_port = find_available_port().await;
    let backend_port = find_available_port().await;
    
    // Start mock auth service
    let auth_addr: SocketAddr = format!("127.0.0.1:{}", auth_port).parse().unwrap();
    let auth_service = MockAuthService::new();
    
    tokio::spawn(async move {
        let _ = Server::builder()
            .add_service(auth::auth_service_server::AuthServiceServer::new(
                auth_service,
            ))
            .serve(auth_addr)
            .await;
    });
    
    // Start mock backend service
    let backend_addr: SocketAddr = format!("127.0.0.1:{}", backend_port).parse().unwrap();
    let backend_service = MockBackendService::new();
    
    tokio::spawn(async move {
        let _ = Server::builder()
            .add_service(
                gateway::service_auth_policy_server::ServiceAuthPolicyServer::new(
                    backend_service,
                ),
            )
            .serve(backend_addr)
            .await;
    });
    
    // Give servers time to start
    sleep(Duration::from_millis(200)).await;
    
    (
        format!("http://127.0.0.1:{}", auth_port),
        format!("http://127.0.0.1:{}", backend_port),
    )
}

// Helper function to create test configuration
fn create_test_config(auth_endpoint: String, backend_endpoint: String) -> (GatewayConfig, Vec<RouteConfig>) {
    let mut services = HashMap::new();
    
    services.insert(
        "auth".to_string(),
        ServiceConfig {
            name: "auth".to_string(),
            endpoint: auth_endpoint.clone(),
            timeout_ms: 3000,
            connection_pool_size: 5,
            auto_discover: true,
        },
    );
    
    services.insert(
        "user-service".to_string(),
        ServiceConfig {
            name: "user-service".to_string(),
            endpoint: backend_endpoint,
            timeout_ms: 5000,
            connection_pool_size: 5,
            auto_discover: true
        },
    );
    
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
            path: "/api/users/:id".to_string(),
            method: "DELETE".to_string(),
            service: "user-service".to_string(),
            grpc_method: "user.UserService/DeleteUser".to_string(),
        },
        RouteConfig {
            path: "/api/public/status".to_string(),
            method: "GET".to_string(),
            service: "user-service".to_string(),
            grpc_method: "user.UserService/GetPublicStatus".to_string(),
        },
    ];
    
    use api_gateway::config::{DiscoveryConfig, ServerConfig, ObservabilityConfig};
    
    let gateway_config = GatewayConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
        },
        services,
        auth: AuthConfig {
            service_endpoint: auth_endpoint,
            timeout_ms: 2000,
        },
        rate_limit: Some(RateLimitConfig {
            enabled: true,
            requests_per_minute: 10,
            window_seconds: 60,
        }),
        observability: ObservabilityConfig {
            tempo_endpoint: "http://localhost:4317".to_string(),
            service_name: "test-gateway".to_string(),
        },
        discovery: DiscoveryConfig {
            enabled: false,  // Disabled for tests
            refresh_interval_seconds: 300,
        },
        route_overrides: vec![],
    };
    
    (gateway_config, routes)
}

// Helper function to create test app state
async fn create_test_app_state(config: GatewayConfig, routes: Vec<RouteConfig>) -> Arc<AppState> {
    let grpc_pool = GrpcClientPool::new(config.services.clone())
        .await
        .expect("Failed to create gRPC pool");
    
    let auth_service = AuthService::new(config.auth.clone())
        .await
        .expect("Failed to create auth service");
    
    // Use provided routes for testing (discovery is disabled in tests)
    let router = RequestRouter::new(routes);
    let rate_limiter = config.rate_limit.as_ref().map(|cfg| RateLimiter::new(cfg.clone()));
    
    Arc::new(AppState::new(
        config,
        grpc_pool,
        auth_service,
        router,
        rate_limiter,
    ))
}

// Helper function to create test router
fn create_test_router(state: Arc<AppState>) -> Router {
    let health_checker = Arc::new(HealthChecker::new(state.grpc_pool.clone()));
    
    let auth_middleware_state = api_gateway::auth::middleware::AuthMiddlewareState {
        auth_service: state.auth_service.clone(),
        grpc_pool: state.grpc_pool.clone(),
        router: state.router.clone(),
        router_lock: state.router_lock.clone(),
    };
    
    Router::new()
        .route("/health", axum::routing::get(health_handler))
        .with_state(health_checker)
        .route(
            "/*path",
            axum::routing::any(gateway_handler).layer(axum::middleware::from_fn_with_state(
                auth_middleware_state,
                auth_middleware,
            )),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;
    
    // Test 1: Request routing with static paths
    #[tokio::test]
    async fn test_request_routing_static_path() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Test that router correctly identifies the route
        let decision = state.router.route("/api/users", "GET");
        assert!(decision.is_ok());
        
        let decision = decision.unwrap();
        assert_eq!(decision.service, "user-service");
        assert_eq!(decision.grpc_method, "user.UserService/ListUsers");
    }
    
    // Test 2: Request routing with dynamic path parameters
    #[tokio::test]
    async fn test_request_routing_dynamic_path() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Test dynamic route with parameter
        let decision = state.router.route("/api/users/123", "GET");
        assert!(decision.is_ok());
        
        let decision = decision.unwrap();
        assert_eq!(decision.service, "user-service");
        assert_eq!(decision.grpc_method, "user.UserService/GetUser");
        assert_eq!(decision.path_params.get("id"), Some(&"123".to_string()));
    }
    
    // Test 3: Route not found returns 404
    #[tokio::test]
    async fn test_route_not_found() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let decision = state.router.route("/api/nonexistent", "GET");
        assert!(decision.is_err());
    }
    
    // Test 4: Dynamic authorization - public endpoint (no auth required)
    #[tokio::test]
    async fn test_dynamic_auth_public_endpoint() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Get service channel for backend service
        let service_channel = state.grpc_pool.get_channel("user-service").unwrap();
        
        // Query auth policy for public endpoint
        let policy = state
            .auth_service
            .get_auth_policy("user-service", "user.UserService/GetPublicStatus", service_channel)
            .await;
        
        assert!(policy.is_ok());
        let policy = policy.unwrap();
        assert!(!policy.require_auth, "Public endpoint should not require auth");
    }
    
    // Test 5: Dynamic authorization - protected endpoint requires auth
    #[tokio::test]
    async fn test_dynamic_auth_protected_endpoint() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Get service channel for backend service
        let service_channel = state.grpc_pool.get_channel("user-service").unwrap();
        
        // Query auth policy for protected endpoint
        let policy = state
            .auth_service
            .get_auth_policy("user-service", "user.UserService/ListUsers", service_channel)
            .await;
        
        assert!(policy.is_ok());
        let policy = policy.unwrap();
        assert!(policy.require_auth, "Protected endpoint should require auth");
        assert!(policy.required_roles.is_empty(), "Should not require specific roles");
    }
    
    // Test 6: Dynamic authorization - admin-only endpoint requires specific role
    #[tokio::test]
    async fn test_dynamic_auth_admin_endpoint() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Get service channel for backend service
        let service_channel = state.grpc_pool.get_channel("user-service").unwrap();
        
        // Query auth policy for admin endpoint
        let policy = state
            .auth_service
            .get_auth_policy("user-service", "user.UserService/DeleteUser", service_channel)
            .await;
        
        assert!(policy.is_ok());
        let policy = policy.unwrap();
        assert!(policy.require_auth, "Admin endpoint should require auth");
        assert_eq!(policy.required_roles, vec!["admin"], "Should require admin role");
    }
    
    // Test 7: Token validation - valid token
    #[tokio::test]
    async fn test_token_validation_valid() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let result = state.auth_service.validate_token("valid_token").await;
        assert!(result.is_ok());
        
        let auth_result = result.unwrap();
        assert!(auth_result.valid);
        assert_eq!(auth_result.claims.as_ref().unwrap().user_id, "user123");
        assert_eq!(auth_result.claims.as_ref().unwrap().roles, vec!["user"]);
    }
    
    // Test 8: Token validation - invalid token
    #[tokio::test]
    async fn test_token_validation_invalid() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let result = state.auth_service.validate_token("invalid_token").await;
        assert!(result.is_ok());
        
        let auth_result = result.unwrap();
        assert!(!auth_result.valid);
        assert!(auth_result.claims.is_none());
    }
    
    // Test 9: Rate limiting - requests within limit
    #[tokio::test]
    async fn test_rate_limiting_within_limit() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let rate_limiter = state.rate_limiter.as_ref().unwrap();
        let client_ip = "127.0.0.1".parse().unwrap();
        
        // Make requests within limit (10 per minute)
        for _ in 0..5 {
            let result = rate_limiter.check_rate_limit(client_ip).await;
            assert!(result.is_ok(), "Requests within limit should succeed");
        }
    }
    
    // Test 10: Rate limiting - requests exceed limit
    #[tokio::test]
    async fn test_rate_limiting_exceed_limit() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let rate_limiter = state.rate_limiter.as_ref().unwrap();
        let client_ip = "127.0.0.2".parse().unwrap();
        
        // Make requests up to limit
        for _ in 0..10 {
            let result = rate_limiter.check_rate_limit(client_ip).await;
            assert!(result.is_ok());
        }
        
        // Next request should be rate limited
        let result = rate_limiter.check_rate_limit(client_ip).await;
        assert!(result.is_err(), "Request exceeding limit should fail");
    }
    
    // Test 11: Health check - all services healthy
    #[tokio::test]
    async fn test_health_check_all_healthy() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        let health_checker = HealthChecker::new(state.grpc_pool.clone());
        let status = health_checker.check_health().await;
        
        assert!(status.healthy, "All services should be healthy");
        assert_eq!(status.services.len(), 2, "Should check 2 services");
    }
    
    // Test 12: Error handling - missing authorization header
    #[tokio::test]
    async fn test_error_handling_missing_auth() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        let app = create_test_router(state);
        
        // Request to protected endpoint without auth header
        let request = Request::builder()
            .uri("/api/users")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        // Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    
    // Test 13: Error handling - invalid token returns 403
    #[tokio::test]
    async fn test_error_handling_invalid_token() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        let app = create_test_router(state);
        
        // Request with invalid token
        let request = Request::builder()
            .uri("/api/users")
            .method("GET")
            .header("Authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        
        // Should return 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    
    // Test 14: Successful request with valid token
    #[tokio::test]
    async fn test_successful_request_with_valid_token() {
        let (auth_endpoint, backend_endpoint) = start_mock_servers().await;
        let (config, routes) = create_test_config(auth_endpoint, backend_endpoint);
        let state = create_test_app_state(config, routes).await;
        
        // Verify token validation works
        let result = state.auth_service.validate_token("valid_token").await;
        assert!(result.is_ok());
        assert!(result.unwrap().valid);
    }
}
