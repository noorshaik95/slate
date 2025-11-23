//! Dynamic gRPC client for calling any gRPC method without code generation.
//!
//! This module provides a dynamic gRPC client that uses prost-reflect for
//! JSON-to-protobuf conversion, enabling runtime service discovery and invocation.

use prost_reflect::DescriptorPool;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::transport::Channel;
use tracing::debug;

use super::types::GrpcError;

mod conversion;
mod descriptor;
mod invocation;
mod metadata;

pub use conversion::{json_to_protobuf, protobuf_to_json};
pub use descriptor::fetch_descriptors;
pub use invocation::invoke_unary_bytes;
pub use metadata::inject_trace_headers;

/// Dynamic gRPC client that can call any gRPC method without code generation.
///
/// Uses prost-reflect for proper JSON-to-protobuf conversion and supports
/// automatic descriptor fetching via gRPC Server Reflection.
pub struct DynamicGrpcClient {
    channel: Channel,
    service_name: String,
    descriptor_pool: Option<Arc<DescriptorPool>>,
}

impl DynamicGrpcClient {
    /// Create a new dynamic gRPC client.
    ///
    /// # Arguments
    ///
    /// * `channel` - The gRPC channel to use for communication
    /// * `service_name` - The fully qualified service name (e.g., "user.UserService")
    pub fn new(channel: Channel, service_name: String) -> Self {
        debug!(service = %service_name, "Creating dynamic gRPC client");
        Self {
            channel,
            service_name,
            descriptor_pool: None,
        }
    }

    /// Set the descriptor pool for this client.
    ///
    /// If not set, the descriptor pool will be fetched automatically via gRPC reflection
    /// on the first call.
    #[allow(dead_code)]
    pub fn with_descriptor_pool(mut self, pool: Arc<DescriptorPool>) -> Self {
        self.descriptor_pool = Some(pool);
        self
    }

    /// Call a gRPC method dynamically with JSON payload.
    ///
    /// This method:
    /// 1. Converts JSON payload to protobuf using prost-reflect
    /// 2. Injects trace headers into gRPC metadata
    /// 3. Makes the gRPC call
    /// 4. Converts the response back to JSON
    ///
    /// # Arguments
    ///
    /// * `method` - The method name (e.g., "GetUser")
    /// * `json_payload` - The request payload as JSON bytes
    /// * `trace_headers` - Trace context headers to propagate
    ///
    /// # Returns
    ///
    /// The response as JSON bytes
    pub async fn call(
        &mut self,
        method: &str,
        json_payload: Vec<u8>,
        trace_headers: HashMap<String, String>,
    ) -> Result<Vec<u8>, GrpcError> {
        debug!(
            service = %self.service_name,
            method = %method,
            payload_size = json_payload.len(),
            "Making dynamic gRPC call with prost-reflect"
        );

        // Get or fetch descriptor pool
        let pool = self.get_or_fetch_descriptor_pool().await?;

        // Convert JSON to protobuf bytes
        let request_bytes = json_to_protobuf(&json_payload, method, &pool, &self.service_name)?;

        // Create gRPC request and inject trace headers
        let request = inject_trace_headers(request_bytes, trace_headers);

        // Make the gRPC call
        let full_method = format!("/{}/{}", self.service_name, method);
        debug!(full_method = %full_method, "Invoking gRPC method");

        let response_bytes =
            invoke_unary_bytes(&mut self.channel, full_method, request, &self.service_name).await?;

        // Convert response back to JSON
        let json_response = protobuf_to_json(&response_bytes, method, &pool, &self.service_name)?;

        debug!(
            service = %self.service_name,
            method = %method,
            response_size = json_response.len(),
            "Dynamic gRPC call completed successfully"
        );

        Ok(json_response)
    }

    /// Get the descriptor pool, fetching it if necessary.
    async fn get_or_fetch_descriptor_pool(&mut self) -> Result<Arc<DescriptorPool>, GrpcError> {
        if let Some(pool) = &self.descriptor_pool {
            Ok(pool.clone())
        } else {
            let pool = fetch_descriptors(&self.channel, &self.service_name).await?;
            let pool_arc = Arc::new(pool);
            self.descriptor_pool = Some(pool_arc.clone());
            Ok(pool_arc)
        }
    }
}

#[cfg(test)]
mod tests {
    
    use tonic::{Code, Status};

    #[test]
    fn test_grpc_status_mapping() {
        use crate::grpc::dynamic_client::invocation::grpc_status_to_http;

        assert_eq!(grpc_status_to_http(&Status::new(Code::Ok, "")), 200);
        assert_eq!(grpc_status_to_http(&Status::new(Code::NotFound, "")), 404);
        assert_eq!(
            grpc_status_to_http(&Status::new(Code::InvalidArgument, "")),
            400
        );
        assert_eq!(
            grpc_status_to_http(&Status::new(Code::Unauthenticated, "")),
            401
        );
        assert_eq!(
            grpc_status_to_http(&Status::new(Code::PermissionDenied, "")),
            403
        );
        assert_eq!(grpc_status_to_http(&Status::new(Code::Internal, "")), 500);
        assert_eq!(
            grpc_status_to_http(&Status::new(Code::Unavailable, "")),
            503
        );
    }
}
