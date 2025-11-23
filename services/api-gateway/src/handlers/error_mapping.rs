//! Error mapping utilities.
//!
//! This module provides utilities for mapping errors between different formats.

#![allow(dead_code)]

use axum::http::StatusCode;
use tonic::Code;
use tracing::error;

/// Maps gRPC status codes to HTTP status codes with context preservation
pub fn map_grpc_error_with_context(
    grpc_code: Code,
    service: &str,
    method: &str,
    _error_message: &str,
) -> (StatusCode, String) {
    let http_status = match grpc_code {
        Code::Ok => StatusCode::OK,
        Code::Cancelled => StatusCode::REQUEST_TIMEOUT,
        Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        Code::InvalidArgument => StatusCode::BAD_REQUEST,
        Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        Code::NotFound => StatusCode::NOT_FOUND,
        Code::AlreadyExists => StatusCode::CONFLICT,
        Code::PermissionDenied => StatusCode::FORBIDDEN,
        Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        Code::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
        Code::Aborted => StatusCode::CONFLICT,
        Code::OutOfRange => StatusCode::BAD_REQUEST,
        Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
        Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
        Code::Unauthenticated => StatusCode::UNAUTHORIZED,
    };

    // Log the error with full context for debugging
    error!(
        service = %service,
        method = %method,
        grpc_code = ?grpc_code,
        http_status = %http_status.as_u16(),
        error_type = "grpc_error",
        "gRPC error mapped to HTTP status"
    );

    // Return generic error message to client (security best practice)
    let client_message = match grpc_code {
        Code::InvalidArgument => "Invalid request parameters".to_string(),
        Code::NotFound => "Resource not found".to_string(),
        Code::AlreadyExists => "Resource already exists".to_string(),
        Code::PermissionDenied => "Permission denied".to_string(),
        Code::Unauthenticated => "Authentication required".to_string(),
        Code::ResourceExhausted => "Rate limit exceeded".to_string(),
        Code::DeadlineExceeded => "Request timeout".to_string(),
        Code::Unavailable => "Service temporarily unavailable".to_string(),
        Code::Unimplemented => "Operation not supported".to_string(),
        _ => "An error occurred processing your request".to_string(),
    };

    (http_status, client_message)
}

/// Error type classification for metrics
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Validation,
    Timeout,
    CircuitOpen,
    GrpcError,
    RateLimit,
    NotFound,
    Internal,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Validation => "validation",
            ErrorType::Timeout => "timeout",
            ErrorType::CircuitOpen => "circuit_open",
            ErrorType::GrpcError => "grpc_error",
            ErrorType::RateLimit => "rate_limit",
            ErrorType::NotFound => "not_found",
            ErrorType::Internal => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_grpc_invalid_argument() {
        let (status, message) = map_grpc_error_with_context(
            Code::InvalidArgument,
            "user-service",
            "CreateUser",
            "email is invalid",
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "Invalid request parameters");
    }

    #[test]
    fn test_map_grpc_not_found() {
        let (status, message) = map_grpc_error_with_context(
            Code::NotFound,
            "user-service",
            "GetUser",
            "user not found",
        );
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(message, "Resource not found");
    }

    #[test]
    fn test_map_grpc_unauthenticated() {
        let (status, message) = map_grpc_error_with_context(
            Code::Unauthenticated,
            "user-service",
            "Login",
            "invalid credentials",
        );
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, "Authentication required");
    }

    #[test]
    fn test_map_grpc_permission_denied() {
        let (status, message) = map_grpc_error_with_context(
            Code::PermissionDenied,
            "user-service",
            "DeleteUser",
            "insufficient permissions",
        );
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(message, "Permission denied");
    }

    #[test]
    fn test_map_grpc_resource_exhausted() {
        let (status, message) = map_grpc_error_with_context(
            Code::ResourceExhausted,
            "user-service",
            "CreateUser",
            "rate limit exceeded",
        );
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(message, "Rate limit exceeded");
    }

    #[test]
    fn test_map_grpc_unavailable() {
        let (status, message) = map_grpc_error_with_context(
            Code::Unavailable,
            "user-service",
            "GetUser",
            "service unavailable",
        );
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(message, "Service temporarily unavailable");
    }

    #[test]
    fn test_map_grpc_deadline_exceeded() {
        let (status, message) = map_grpc_error_with_context(
            Code::DeadlineExceeded,
            "user-service",
            "SlowOperation",
            "deadline exceeded",
        );
        assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(message, "Request timeout");
    }

    #[test]
    fn test_error_type_as_str() {
        assert_eq!(ErrorType::Validation.as_str(), "validation");
        assert_eq!(ErrorType::Timeout.as_str(), "timeout");
        assert_eq!(ErrorType::CircuitOpen.as_str(), "circuit_open");
        assert_eq!(ErrorType::GrpcError.as_str(), "grpc_error");
        assert_eq!(ErrorType::RateLimit.as_str(), "rate_limit");
        assert_eq!(ErrorType::NotFound.as_str(), "not_found");
        assert_eq!(ErrorType::Internal.as_str(), "internal");
    }
}
