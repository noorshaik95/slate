use super::*;
use super::types::{GrpcError, GrpcRequest};
use std::collections::HashMap;

#[test]
fn test_service_config_creation() {
    use common_rust::circuit_breaker::CircuitBreakerConfig;

    let config = crate::config::ServiceConfig {
        name: "test-service".to_string(),
        endpoint: "http://localhost:50051".to_string(),
        timeout_ms: 5000,
        connection_pool_size: 5,
        auto_discover: true,
        tls_enabled: false,
        tls_domain: None,
        tls_ca_cert_path: None,
        circuit_breaker: CircuitBreakerConfig::default(),
    };

    assert_eq!(config.name, "test-service");
    assert_eq!(config.endpoint, "http://localhost:50051");
    assert_eq!(config.timeout_ms, 5000);
    assert_eq!(config.connection_pool_size, 5);
    assert!(config.auto_discover);
}

#[test]
fn test_is_retryable_error() {
    assert!(GrpcClientPool::is_retryable_error(&GrpcError::Timeout(
        "test".to_string()
    )));
    assert!(GrpcClientPool::is_retryable_error(
        &GrpcError::ConnectionError("test".to_string())
    ));
    assert!(GrpcClientPool::is_retryable_error(&GrpcError::CallFailed(
        "Unavailable".to_string()
    )));
    assert!(!GrpcClientPool::is_retryable_error(
        &GrpcError::ServiceNotFound("test".to_string())
    ));
    assert!(!GrpcClientPool::is_retryable_error(
        &GrpcError::InvalidConfig("test".to_string())
    ));
}

#[test]
fn test_grpc_request_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("trace-id".to_string(), "abc123".to_string());

    let request = GrpcRequest {
        service: "user-service".to_string(),
        method: "GetUser".to_string(),
        payload: vec![1, 2, 3],
        metadata,
    };

    assert_eq!(request.service, "user-service");
    assert_eq!(request.method, "GetUser");
    assert_eq!(request.payload, vec![1, 2, 3]);
    assert_eq!(request.metadata.get("trace-id").unwrap(), "abc123");
}
