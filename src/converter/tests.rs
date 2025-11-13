use super::*;
use axum::body::Body;
use axum::http::Request;
use serde_json::json;
use std::collections::HashMap;

use crate::grpc::{GrpcResponse};
use crate::router::RoutingDecision;

#[tokio::test]
async fn test_convert_empty_body() {
    let request = Request::builder()
        .method("GET")
        .uri("/api/users")
        .body(Body::empty())
        .unwrap();

    let routing = RoutingDecision {
        service: "user-service".to_string(),
        grpc_method: "user.UserService/ListUsers".to_string(),
        path_params: HashMap::new(),
    };

    let result = HttpToGrpcConverter::convert_request(request, &routing).await;
    assert!(result.is_ok());

    let grpc_req = result.unwrap();
    assert_eq!(grpc_req.service, "user-service");
    assert_eq!(grpc_req.method, "user.UserService/ListUsers");

    // Should have empty JSON object
    let payload: serde_json::Value = serde_json::from_slice(&grpc_req.payload).unwrap();
    assert!(payload.is_object());
}

#[tokio::test]
async fn test_convert_with_json_body() {
    let body = json!({
        "name": "John Doe",
        "email": "john@example.com"
    });

    let request = Request::builder()
        .method("POST")
        .uri("/api/users")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let routing = RoutingDecision {
        service: "user-service".to_string(),
        grpc_method: "user.UserService/CreateUser".to_string(),
        path_params: HashMap::new(),
    };

    let result = HttpToGrpcConverter::convert_request(request, &routing).await;
    assert!(result.is_ok());

    let grpc_req = result.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&grpc_req.payload).unwrap();
    assert_eq!(payload["name"], "John Doe");
    assert_eq!(payload["email"], "john@example.com");
}

#[tokio::test]
async fn test_convert_with_path_params() {
    let request = Request::builder()
        .method("GET")
        .uri("/api/users/123")
        .body(Body::empty())
        .unwrap();

    let mut path_params = HashMap::new();
    path_params.insert("id".to_string(), "123".to_string());

    let routing = RoutingDecision {
        service: "user-service".to_string(),
        grpc_method: "user.UserService/GetUser".to_string(),
        path_params,
    };

    let result = HttpToGrpcConverter::convert_request(request, &routing).await;
    assert!(result.is_ok());

    let grpc_req = result.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&grpc_req.payload).unwrap();
    assert_eq!(payload["id"], "123");
}

#[tokio::test]
async fn test_merge_path_params_with_body() {
    let body = json!({
        "name": "John Doe"
    });

    let request = Request::builder()
        .method("PUT")
        .uri("/api/users/456")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let mut path_params = HashMap::new();
    path_params.insert("id".to_string(), "456".to_string());

    let routing = RoutingDecision {
        service: "user-service".to_string(),
        grpc_method: "user.UserService/UpdateUser".to_string(),
        path_params,
    };

    let result = HttpToGrpcConverter::convert_request(request, &routing).await;
    assert!(result.is_ok());

    let grpc_req = result.unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&grpc_req.payload).unwrap();
    assert_eq!(payload["id"], "456");
    assert_eq!(payload["name"], "John Doe");
}

#[tokio::test]
async fn test_trace_header_propagation() {
    let request = Request::builder()
        .method("GET")
        .uri("/api/users")
        .header("x-trace-id", "trace-123")
        .header("x-span-id", "span-456")
        .body(Body::empty())
        .unwrap();

    let routing = RoutingDecision {
        service: "user-service".to_string(),
        grpc_method: "user.UserService/ListUsers".to_string(),
        path_params: HashMap::new(),
    };

    let result = HttpToGrpcConverter::convert_request(request, &routing).await;
    assert!(result.is_ok());

    let grpc_req = result.unwrap();
    assert_eq!(grpc_req.metadata.get("x-trace-id").unwrap(), "trace-123");
    assert_eq!(grpc_req.metadata.get("x-span-id").unwrap(), "span-456");
}

#[test]
fn test_grpc_to_http_success() {
    let payload = json!({
        "id": "123",
        "name": "John Doe"
    });

    let grpc_resp = GrpcResponse {
        status: tonic::Code::Ok,
        payload: serde_json::to_vec(&payload).unwrap(),
        metadata: HashMap::new(),
    };

    let result = GrpcToHttpConverter::convert_response(grpc_resp);
    assert!(result.is_ok());

    let http_resp = result.unwrap();
    assert_eq!(http_resp.status(), axum::http::StatusCode::OK);
}

#[test]
fn test_grpc_status_mapping() {
    use axum::http::StatusCode;
    
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::Ok),
        StatusCode::OK
    );
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::NotFound),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::InvalidArgument),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::Unauthenticated),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::PermissionDenied),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        GrpcToHttpConverter::map_grpc_status_to_http(tonic::Code::Unavailable),
        StatusCode::SERVICE_UNAVAILABLE
    );
}

#[test]
fn test_grpc_to_http_with_metadata() {
    let payload = json!({"result": "success"});
    let mut metadata = HashMap::new();
    metadata.insert("x-trace-id".to_string(), "trace-789".to_string());
    metadata.insert("x-internal-header".to_string(), "should-not-propagate".to_string());

    let grpc_resp = GrpcResponse {
        status: tonic::Code::Ok,
        payload: serde_json::to_vec(&payload).unwrap(),
        metadata,
    };

    let result = GrpcToHttpConverter::convert_response(grpc_resp);
    assert!(result.is_ok());

    let http_resp = result.unwrap();
    assert!(http_resp.headers().contains_key("x-trace-id"));
    assert!(!http_resp.headers().contains_key("x-internal-header"));
}

#[test]
fn test_grpc_to_http_empty_payload() {
    let grpc_resp = GrpcResponse {
        status: tonic::Code::Ok,
        payload: vec![],
        metadata: HashMap::new(),
    };

    let result = GrpcToHttpConverter::convert_response(grpc_resp);
    assert!(result.is_ok());

    let http_resp = result.unwrap();
    assert_eq!(http_resp.status(), axum::http::StatusCode::OK);
}
