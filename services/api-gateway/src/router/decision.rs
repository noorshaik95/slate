//! Routing decision types and errors.
//!
//! Defines the result types for routing operations.

use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during routing.
#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Route not found for path: {path}, method: {method}")]
    RouteNotFound { path: String, method: String },
}

/// Result of a routing decision.
///
/// Performance: Uses Arc<str> for service and grpc_method to avoid cloning strings
/// in the hot path. Cloning Arc is cheap (atomic reference count increment) compared
/// to cloning the actual string data.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub service: Arc<str>,
    pub grpc_method: Arc<str>,
    pub path_params: HashMap<String, String>,
}

impl RoutingDecision {
    /// Create a new routing decision.
    pub fn new(service: impl AsRef<str>, grpc_method: impl AsRef<str>) -> Self {
        Self {
            service: Arc::from(service.as_ref()),
            grpc_method: Arc::from(grpc_method.as_ref()),
            path_params: HashMap::new(),
        }
    }

    /// Create a new routing decision with path parameters.
    pub fn with_params(
        service: impl AsRef<str>,
        grpc_method: impl AsRef<str>,
        path_params: HashMap<String, String>,
    ) -> Self {
        Self {
            service: Arc::from(service.as_ref()),
            grpc_method: Arc::from(grpc_method.as_ref()),
            path_params,
        }
    }
}
