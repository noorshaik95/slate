use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use tracing::{debug, error, warn};

use crate::auth::{AuthError, AuthResult, AuthService};
use crate::grpc::client::GrpcClientPool;
use crate::router::{RequestRouter, RoutingDecision};
use tokio::sync::RwLock;

/// Extension type to pass auth context to downstream handlers
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub authenticated: bool,
}

impl AuthContext {
    /// Create an unauthenticated context
    pub fn unauthenticated() -> Self {
        Self {
            user_id: None,
            roles: vec![],
            authenticated: false,
        }
    }

    /// Create an authenticated context from auth result
    pub fn from_auth_result(result: &AuthResult) -> Self {
        if let Some(claims) = &result.claims {
            Self {
                user_id: Some(claims.user_id.clone()),
                roles: claims.roles.clone(),
                authenticated: true,
            }
        } else {
            Self::unauthenticated()
        }
    }
}

/// State required for authorization middleware
#[derive(Clone)]
pub struct AuthMiddlewareState {
    pub auth_service: Arc<AuthService>,
    pub grpc_pool: Arc<GrpcClientPool>,
    pub router_lock: Arc<RwLock<RequestRouter>>,
    pub public_routes: Vec<(String, String)>, // (path, method) tuples
}

/// Authorization middleware that enforces dynamic auth policies
///
/// This middleware:
/// 1. Routes the request to determine target service and method
/// 2. Queries the backend service for its auth policy
/// 3. If auth required, extracts and validates the token
/// 4. Checks user roles against required roles
/// 5. Passes auth context to downstream handlers
pub async fn auth_middleware(
    State(state): State<AuthMiddlewareState>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthMiddlewareResponse> {
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    debug!(
        path = %path,
        method = %method,
        "Processing authorization middleware"
    );

    // Skip auth for health and metrics endpoints
    if path == "/health" || path == "/metrics" {
        debug!(path = %path, "Skipping auth for system endpoint");
        let auth_ctx = AuthContext::unauthenticated();
        request.extensions_mut().insert(auth_ctx);
        return Ok(next.run(request).await);
    }

    // Route the request to determine target service and gRPC method
    let router_guard = state.router_lock.read().await;
    let routing_decision = match router_guard.route(&path, &method) {
        Ok(decision) => {
            let result = decision.clone();
            drop(router_guard);
            result
        }
        Err(e) => {
            drop(router_guard);
            warn!(
                path = %path,
                method = %method,
                error = %e,
                "Route not found"
            );
            return Err(AuthMiddlewareResponse::NotFound);
        }
    };

    debug!(
        service = %routing_decision.service,
        grpc_method = %routing_decision.grpc_method,
        "Request routed to backend service"
    );

    // Store routing decision in request extensions for reuse in gateway handler
    // This avoids duplicate route lookup
    request.extensions_mut().insert(routing_decision.clone());

    // Check if this is a public route (after routing so we have the decision)
    let is_public = state.public_routes.iter().any(|(p, m)| p == &path && m == &method);
    if is_public {
        debug!(path = %path, method = %method, "Skipping auth for public route");
        let auth_ctx = AuthContext::unauthenticated();
        request.extensions_mut().insert(auth_ctx);
        return Ok(next.run(request).await);
    }

    // Get channel for the backend service
    let service_channel = match state.grpc_pool.get_channel(&routing_decision.service) {
        Ok(channel) => channel,
        Err(e) => {
            error!(
                service = %routing_decision.service,
                error = %e,
                "Failed to get service channel"
            );
            return Err(AuthMiddlewareResponse::ServiceUnavailable);
        }
    };

    // Query backend service for auth policy
    let auth_policy = match state
        .auth_service
        .get_auth_policy(
            &routing_decision.service,
            &routing_decision.grpc_method,
            service_channel,
        )
        .await
    {
        Ok(policy) => policy,
        Err(e) => {
            error!(
                service = %routing_decision.service,
                method = %routing_decision.grpc_method,
                error = %e,
                "Failed to get auth policy"
            );
            // Fail-secure: if we can't get policy, require auth
            return Err(AuthMiddlewareResponse::ServiceUnavailable);
        }
    };

    debug!(
        service = %routing_decision.service,
        method = %routing_decision.grpc_method,
        require_auth = auth_policy.require_auth,
        required_roles = ?auth_policy.required_roles,
        "Auth policy retrieved"
    );

    // Extract token from headers
    let token = AuthService::extract_token(&headers);

    // Check authorization based on policy
    let auth_result = match state
        .auth_service
        .check_authorization(token.as_deref(), &auth_policy)
        .await
    {
        Ok(result) => result,
        Err(e) => {
            warn!(
                service = %routing_decision.service,
                method = %routing_decision.grpc_method,
                error = %e,
                "Authorization check failed"
            );
            return Err(map_auth_error_to_response(e));
        }
    };

    // Create auth context for downstream handlers
    let auth_ctx = AuthContext::from_auth_result(&auth_result);

    debug!(
        authenticated = auth_ctx.authenticated,
        user_id = ?auth_ctx.user_id,
        roles = ?auth_ctx.roles,
        "Authorization successful, passing context to handler"
    );

    // Insert auth context into request extensions
    request.extensions_mut().insert(auth_ctx);

    // Continue to next handler
    Ok(next.run(request).await)
}

/// Map AuthError to appropriate HTTP response
pub fn map_auth_error_to_response(error: AuthError) -> AuthMiddlewareResponse {
    match error {
        AuthError::MissingToken => {
            debug!("Authorization failed: missing token");
            AuthMiddlewareResponse::Unauthorized("Missing authentication token".to_string())
        }
        AuthError::InvalidToken(msg) => {
            debug!(error = %msg, "Authorization failed: invalid token");
            AuthMiddlewareResponse::Forbidden(format!("Invalid token: {}", msg))
        }
        AuthError::ExpiredToken => {
            debug!("Authorization failed: expired token");
            AuthMiddlewareResponse::Unauthorized("Token has expired".to_string())
        }
        AuthError::InsufficientPermissions(msg) => {
            debug!(error = %msg, "Authorization failed: insufficient permissions");
            AuthMiddlewareResponse::Forbidden(msg)
        }
        AuthError::ServiceError(msg) => {
            error!(error = %msg, "Auth service error");
            AuthMiddlewareResponse::ServiceUnavailable
        }
        AuthError::ConnectionError(msg) => {
            error!(error = %msg, "Auth service connection error");
            AuthMiddlewareResponse::ServiceUnavailable
        }
        AuthError::CacheError(msg) => {
            error!(error = %msg, "Policy cache error");
            AuthMiddlewareResponse::InternalError
        }
    }
}

/// Response types for authorization middleware errors
#[derive(Debug)]
pub enum AuthMiddlewareResponse {
    Unauthorized(String),
    Forbidden(String),
    NotFound,
    ServiceUnavailable,
    InternalError,
}

/// Error response body structure
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    message: String,
    status: u16,
}

impl IntoResponse for AuthMiddlewareResponse {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthMiddlewareResponse::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AuthMiddlewareResponse::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AuthMiddlewareResponse::NotFound => {
                (StatusCode::NOT_FOUND, "Route not found".to_string())
            }
            AuthMiddlewareResponse::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Service temporarily unavailable".to_string(),
            ),
            AuthMiddlewareResponse::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        let body = ErrorResponse {
            error: ErrorDetail {
                message,
                status: status.as_u16(),
            },
        };

        (status, Json(body)).into_response()
    }
}
