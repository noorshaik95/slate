use api_gateway::handlers::error_mapping::{map_grpc_error_with_context, ErrorType};
use axum::http::StatusCode;
use tonic::Code;

/// Test all gRPC status codes map to correct HTTP status codes
#[test]
fn test_all_grpc_codes_mapping() {
    let test_cases = vec![
        (Code::Ok, StatusCode::OK),
        (Code::Cancelled, StatusCode::REQUEST_TIMEOUT),
        (Code::Unknown, StatusCode::INTERNAL_SERVER_ERROR),
        (Code::InvalidArgument, StatusCode::BAD_REQUEST),
        (Code::DeadlineExceeded, StatusCode::GATEWAY_TIMEOUT),
        (Code::NotFound, StatusCode::NOT_FOUND),
        (Code::AlreadyExists, StatusCode::CONFLICT),
        (Code::PermissionDenied, StatusCode::FORBIDDEN),
        (Code::ResourceExhausted, StatusCode::TOO_MANY_REQUESTS),
        (Code::FailedPrecondition, StatusCode::PRECONDITION_FAILED),
        (Code::Aborted, StatusCode::CONFLICT),
        (Code::OutOfRange, StatusCode::BAD_REQUEST),
        (Code::Unimplemented, StatusCode::NOT_IMPLEMENTED),
        (Code::Internal, StatusCode::INTERNAL_SERVER_ERROR),
        (Code::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
        (Code::DataLoss, StatusCode::INTERNAL_SERVER_ERROR),
        (Code::Unauthenticated, StatusCode::UNAUTHORIZED),
    ];

    for (grpc_code, expected_http_status) in test_cases {
        let (http_status, _) =
            map_grpc_error_with_context(grpc_code, "test-service", "TestMethod", "test error");
        assert_eq!(
            http_status, expected_http_status,
            "gRPC code {:?} should map to HTTP {}",
            grpc_code, expected_http_status
        );
    }
}

/// Test error context preservation in logs
#[test]
fn test_error_context_preservation() {
    let service = "user-auth-service";
    let method = "CreateUser";
    let error_msg = "email already exists";

    let (status, client_msg) =
        map_grpc_error_with_context(Code::AlreadyExists, service, method, error_msg);

    // HTTP status should be correct
    assert_eq!(status, StatusCode::CONFLICT);

    // Client message should be generic (not expose internal details)
    assert_eq!(client_msg, "Resource already exists");
    assert!(!client_msg.contains("email"));
}

/// Test generic error messages to clients
#[test]
fn test_generic_client_messages() {
    let test_cases = vec![
        (Code::InvalidArgument, "Invalid request parameters"),
        (Code::NotFound, "Resource not found"),
        (Code::AlreadyExists, "Resource already exists"),
        (Code::PermissionDenied, "Permission denied"),
        (Code::Unauthenticated, "Authentication required"),
        (Code::ResourceExhausted, "Rate limit exceeded"),
        (Code::DeadlineExceeded, "Request timeout"),
        (Code::Unavailable, "Service temporarily unavailable"),
        (Code::Unimplemented, "Operation not supported"),
        (Code::Internal, "An error occurred processing your request"),
        (Code::Unknown, "An error occurred processing your request"),
    ];

    for (grpc_code, expected_message) in test_cases {
        let (_, client_msg) = map_grpc_error_with_context(
            grpc_code,
            "test-service",
            "TestMethod",
            "internal error details that should not be exposed",
        );
        assert_eq!(
            client_msg, expected_message,
            "gRPC code {:?} should return generic message",
            grpc_code
        );
    }
}

/// Test that internal error details are not exposed to clients
#[test]
fn test_internal_details_not_exposed() {
    let sensitive_errors = vec![
        "database connection failed: postgres://user:password@localhost",
        "SQL error: duplicate key value violates unique constraint",
        "panic: runtime error: index out of range",
        "failed to connect to redis at 10.0.0.5:6379",
    ];

    for internal_error in sensitive_errors {
        let (_, client_msg) = map_grpc_error_with_context(
            Code::Internal,
            "test-service",
            "TestMethod",
            internal_error,
        );

        // Client message should be generic
        assert_eq!(client_msg, "An error occurred processing your request");

        // Should not contain any part of the internal error
        assert!(!client_msg.contains("database"));
        assert!(!client_msg.contains("SQL"));
        assert!(!client_msg.contains("panic"));
        assert!(!client_msg.contains("redis"));
        assert!(!client_msg.contains("password"));
        assert!(!client_msg.contains("10.0.0.5"));
    }
}

/// Test authentication and authorization error mapping
#[test]
fn test_auth_error_mapping() {
    // Unauthenticated - no credentials or invalid credentials
    let (status, msg) = map_grpc_error_with_context(
        Code::Unauthenticated,
        "user-service",
        "Login",
        "invalid password",
    );
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(msg, "Authentication required");

    // PermissionDenied - authenticated but not authorized
    let (status, msg) = map_grpc_error_with_context(
        Code::PermissionDenied,
        "user-service",
        "DeleteUser",
        "user does not have admin role",
    );
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(msg, "Permission denied");
}

/// Test rate limiting error mapping
#[test]
fn test_rate_limit_error_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::ResourceExhausted,
        "user-service",
        "CreateUser",
        "rate limit exceeded: 100 requests per minute",
    );
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(msg, "Rate limit exceeded");

    // Should not expose rate limit details
    assert!(!msg.contains("100"));
    assert!(!msg.contains("per minute"));
}

/// Test timeout error mapping
#[test]
fn test_timeout_error_mapping() {
    // DeadlineExceeded
    let (status, msg) = map_grpc_error_with_context(
        Code::DeadlineExceeded,
        "slow-service",
        "SlowOperation",
        "operation timed out after 30 seconds",
    );
    assert_eq!(status, StatusCode::GATEWAY_TIMEOUT);
    assert_eq!(msg, "Request timeout");

    // Cancelled
    let (status, _msg) = map_grpc_error_with_context(
        Code::Cancelled,
        "user-service",
        "CreateUser",
        "request cancelled by client",
    );
    assert_eq!(status, StatusCode::REQUEST_TIMEOUT);
}

/// Test validation error mapping
#[test]
fn test_validation_error_mapping() {
    let validation_errors = vec![
        "email format is invalid",
        "password must be at least 8 characters",
        "phone number must be in E.164 format",
    ];

    for error in validation_errors {
        let (status, msg) =
            map_grpc_error_with_context(Code::InvalidArgument, "user-service", "CreateUser", error);
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(msg, "Invalid request parameters");
    }
}

/// Test service unavailability error mapping
#[test]
fn test_service_unavailable_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::Unavailable,
        "user-service",
        "GetUser",
        "service is temporarily unavailable",
    );
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(msg, "Service temporarily unavailable");
}

/// Test conflict error mapping
#[test]
fn test_conflict_error_mapping() {
    // AlreadyExists
    let (status, msg) = map_grpc_error_with_context(
        Code::AlreadyExists,
        "user-service",
        "CreateUser",
        "user with email test@example.com already exists",
    );
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(msg, "Resource already exists");

    // Aborted
    let (status, _) = map_grpc_error_with_context(
        Code::Aborted,
        "user-service",
        "UpdateUser",
        "transaction aborted due to conflict",
    );
    assert_eq!(status, StatusCode::CONFLICT);
}

/// Test not found error mapping
#[test]
fn test_not_found_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::NotFound,
        "user-service",
        "GetUser",
        "user with id 12345 not found",
    );
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(msg, "Resource not found");

    // Should not expose resource ID
    assert!(!msg.contains("12345"));
}

/// Test unimplemented operation mapping
#[test]
fn test_unimplemented_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::Unimplemented,
        "user-service",
        "FutureFeature",
        "this feature is not yet implemented",
    );
    assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(msg, "Operation not supported");
}

/// Test precondition failed mapping
#[test]
fn test_precondition_failed_mapping() {
    let (status, _) = map_grpc_error_with_context(
        Code::FailedPrecondition,
        "user-service",
        "UpdateUser",
        "user must verify email before updating profile",
    );
    assert_eq!(status, StatusCode::PRECONDITION_FAILED);
}

/// Test out of range mapping
#[test]
fn test_out_of_range_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::OutOfRange,
        "user-service",
        "ListUsers",
        "page number 1000 is out of range",
    );
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // OutOfRange falls into the default case
    assert_eq!(msg, "An error occurred processing your request");
}

/// Test error type classification
#[test]
fn test_error_type_classification() {
    assert_eq!(ErrorType::Validation.as_str(), "validation");
    assert_eq!(ErrorType::Timeout.as_str(), "timeout");
    assert_eq!(ErrorType::CircuitOpen.as_str(), "circuit_open");
    assert_eq!(ErrorType::GrpcError.as_str(), "grpc_error");
    assert_eq!(ErrorType::RateLimit.as_str(), "rate_limit");
    assert_eq!(ErrorType::NotFound.as_str(), "not_found");
    assert_eq!(ErrorType::Internal.as_str(), "internal");
}

/// Test multiple services with same error code
#[test]
fn test_multiple_services_same_error() {
    let services = vec![
        "user-auth-service",
        "payment-service",
        "notification-service",
    ];

    for service in services {
        let (status, msg) = map_grpc_error_with_context(
            Code::NotFound,
            service,
            "GetResource",
            "resource not found",
        );
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(msg, "Resource not found");
    }
}

/// Test error mapping consistency
#[test]
fn test_error_mapping_consistency() {
    // Same error code should always map to same HTTP status
    for _ in 0..100 {
        let (status, _) = map_grpc_error_with_context(
            Code::InvalidArgument,
            "test-service",
            "TestMethod",
            "test error",
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

/// Test edge case: empty error message
#[test]
fn test_empty_error_message() {
    let (status, msg) =
        map_grpc_error_with_context(Code::Internal, "test-service", "TestMethod", "");
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(msg, "An error occurred processing your request");
}

/// Test edge case: very long error message
#[test]
fn test_long_error_message() {
    let long_error = "a".repeat(10000);
    let (status, msg) =
        map_grpc_error_with_context(Code::Internal, "test-service", "TestMethod", &long_error);
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    // Client message should still be generic and short
    assert_eq!(msg, "An error occurred processing your request");
    assert!(msg.len() < 100);
}

/// Test security: SQL injection in error message
#[test]
fn test_sql_injection_in_error_not_exposed() {
    let sql_injection = "'; DROP TABLE users; --";
    let (_, msg) = map_grpc_error_with_context(
        Code::InvalidArgument,
        "user-service",
        "CreateUser",
        sql_injection,
    );

    // Should return generic message, not the SQL injection attempt
    assert_eq!(msg, "Invalid request parameters");
    assert!(!msg.contains("DROP TABLE"));
    assert!(!msg.contains("--"));
}

/// Test security: XSS in error message
#[test]
fn test_xss_in_error_not_exposed() {
    let xss_attempt = "<script>alert('xss')</script>";
    let (_, msg) = map_grpc_error_with_context(
        Code::InvalidArgument,
        "user-service",
        "CreateUser",
        xss_attempt,
    );

    // Should return generic message, not the XSS attempt
    assert_eq!(msg, "Invalid request parameters");
    assert!(!msg.contains("<script>"));
    assert!(!msg.contains("alert"));
}

/// Test data loss error mapping
#[test]
fn test_data_loss_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::DataLoss,
        "storage-service",
        "SaveData",
        "data corruption detected",
    );
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(msg, "An error occurred processing your request");
}

/// Test unknown error mapping
#[test]
fn test_unknown_error_mapping() {
    let (status, msg) = map_grpc_error_with_context(
        Code::Unknown,
        "test-service",
        "TestMethod",
        "unknown error occurred",
    );
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(msg, "An error occurred processing your request");
}
