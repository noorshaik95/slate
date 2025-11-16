// Unit tests for error mapping to gRPC status codes
// Extracted from src/error.rs

use content_management_service::error::{ServiceError, ErrorResponse};
use tonic::Code;

#[test]
fn test_validation_error_to_status() {
    let error = ServiceError::Validation("Invalid input".to_string());
    let status = error.to_status("trace-123");
    
    assert_eq!(status.code(), Code::InvalidArgument);
    assert!(status.message().contains("VALIDATION_ERROR"));
}

#[test]
fn test_not_found_error_to_status() {
    let error = ServiceError::ResourceNotFound("resource-123".to_string());
    let status = error.to_status("trace-456");
    
    assert_eq!(status.code(), Code::NotFound);
    assert!(status.message().contains("RESOURCE_NOT_FOUND"));
}

#[test]
fn test_authorization_error_to_status() {
    let error = ServiceError::AccessDenied("Insufficient permissions".to_string());
    let status = error.to_status("trace-789");
    
    assert_eq!(status.code(), Code::PermissionDenied);
    assert!(status.message().contains("ACCESS_DENIED"));
}

#[test]
fn test_service_unavailable_error_to_status() {
    let error = ServiceError::DatabaseUnavailable;
    let status = error.to_status("trace-abc");
    
    assert_eq!(status.code(), Code::Unavailable);
    assert!(status.message().contains("DATABASE_UNAVAILABLE"));
}

#[test]
fn test_error_response_serialization() {
    let response = ErrorResponse::new(
        "TEST_ERROR".to_string(),
        "Test error message".to_string(),
        "trace-123".to_string(),
    );
    
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("TEST_ERROR"));
    assert!(json.contains("Test error message"));
    assert!(json.contains("trace-123"));
}

#[test]
fn test_error_response_with_details() {
    let response = ErrorResponse::new(
        "TEST_ERROR".to_string(),
        "Test error message".to_string(),
        "trace-123".to_string(),
    ).with_details(serde_json::json!({
        "field": "name",
        "constraint": "max_length"
    }));
    
    assert!(response.details.is_some());
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("field"));
    assert!(json.contains("name"));
}
