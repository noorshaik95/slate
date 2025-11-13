use api_gateway::auth::middleware::{AuthContext, map_auth_error_to_response, AuthMiddlewareResponse};
use api_gateway::auth::{AuthError, AuthResult, TokenClaims};

#[test]
fn test_auth_context_unauthenticated() {
    let ctx = AuthContext::unauthenticated();
    assert!(!ctx.authenticated);
    assert!(ctx.user_id.is_none());
    assert!(ctx.roles.is_empty());
}

#[test]
fn test_auth_context_from_auth_result_with_claims() {
    let result = AuthResult {
        valid: true,
        claims: Some(TokenClaims {
            user_id: "user123".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
            exp: 0,
        }),
        error: None,
    };

    let ctx = AuthContext::from_auth_result(&result);
    assert!(ctx.authenticated);
    assert_eq!(ctx.user_id, Some("user123".to_string()));
    assert_eq!(ctx.roles, vec!["admin", "user"]);
}

#[test]
fn test_auth_context_from_auth_result_without_claims() {
    let result = AuthResult {
        valid: false,
        claims: None,
        error: Some("Invalid token".to_string()),
    };

    let ctx = AuthContext::from_auth_result(&result);
    assert!(!ctx.authenticated);
    assert!(ctx.user_id.is_none());
    assert!(ctx.roles.is_empty());
}

#[test]
fn test_map_auth_error_missing_token() {
    let response = map_auth_error_to_response(AuthError::MissingToken);
    match response {
        AuthMiddlewareResponse::Unauthorized(msg) => {
            assert_eq!(msg, "Missing authentication token");
        }
        _ => panic!("Expected Unauthorized response"),
    }
}

#[test]
fn test_map_auth_error_invalid_token() {
    let response = map_auth_error_to_response(AuthError::InvalidToken("bad token".to_string()));
    match response {
        AuthMiddlewareResponse::Forbidden(msg) => {
            assert!(msg.contains("bad token"));
        }
        _ => panic!("Expected Forbidden response"),
    }
}

#[test]
fn test_map_auth_error_insufficient_permissions() {
    let response = map_auth_error_to_response(AuthError::InsufficientPermissions(
        "Need admin role".to_string(),
    ));
    match response {
        AuthMiddlewareResponse::Forbidden(msg) => {
            assert_eq!(msg, "Need admin role");
        }
        _ => panic!("Expected Forbidden response"),
    }
}

#[test]
fn test_map_auth_error_service_error() {
    let response = map_auth_error_to_response(AuthError::ServiceError("Service down".to_string()));
    match response {
        AuthMiddlewareResponse::ServiceUnavailable => {}
        _ => panic!("Expected ServiceUnavailable response"),
    }
}
