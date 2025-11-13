use std::collections::HashMap;
use tonic::Status;

/// Error types for gRPC client operations
#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Call failed: {0}")]
    CallFailed(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<tonic::transport::Error> for GrpcError {
    fn from(err: tonic::transport::Error) -> Self {
        GrpcError::ConnectionError(err.to_string())
    }
}

impl From<Status> for GrpcError {
    fn from(status: Status) -> Self {
        GrpcError::CallFailed(status.to_string())
    }
}

/// Request structure for generic gRPC calls
#[derive(Debug, Clone)]
pub struct GrpcRequest {
    pub service: String,
    pub method: String,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// Response structure for generic gRPC calls
#[derive(Debug, Clone)]
pub struct GrpcResponse {
    pub status: tonic::Code,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
}
