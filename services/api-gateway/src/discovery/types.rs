//! Discovery type definitions.
//!
//! These types are part of the public API for route discovery.

#![allow(dead_code)]

use thiserror::Error;

/// Errors that can occur during route discovery
#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("Service does not support reflection: {0}")]
    ReflectionNotSupported(String),

    #[error("Failed to connect to service: {0}")]
    ConnectionFailed(String),

    #[error("Failed to query service methods: {0}")]
    QueryFailed(String),

    #[error("Invalid method descriptor: {0}")]
    InvalidDescriptor(String),

    #[error("Duplicate route detected: {method1} and {method2} both map to {http_route}")]
    DuplicateRoute {
        method1: String,
        method2: String,
        http_route: String,
    },

    #[error("No methods discovered for service: {0}")]
    EmptyService(String),
}

/// Errors that can occur during gRPC reflection
#[derive(Debug, Error)]
pub enum ReflectionError {
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Malformed method descriptor: {0}")]
    MalformedDescriptor(String),
}

/// Mapping from gRPC method to HTTP route
#[derive(Debug, Clone)]
pub struct RouteMapping {
    pub http_method: String,
    pub http_path: String,
    pub grpc_method: String,
}

/// Descriptor for a gRPC method
#[derive(Debug, Clone)]
pub struct MethodDescriptor {
    pub name: String,
    pub full_name: String,
    pub input_type: String,
    pub output_type: String,
}

/// Type of gRPC method based on naming convention
#[derive(Debug, Clone, PartialEq)]
pub enum MethodType {
    Get,    // GET /api/{resources}/:id
    List,   // GET /api/{resources}
    Create, // POST /api/{resources}
    Update, // PUT /api/{resources}/:id
    Delete, // DELETE /api/{resources}/:id
}
