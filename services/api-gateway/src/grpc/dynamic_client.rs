use prost::Message;
use prost_types::Any;
use std::collections::HashMap;
use tonic::transport::Channel;
use tonic::{codec::ProstCodec, metadata::MetadataMap, Request, Response, Status};
use tracing::{debug, error, warn};

use super::types::GrpcError;

/// Dynamic gRPC client that can call any gRPC method without code generation
pub struct DynamicGrpcClient {
    channel: Channel,
    service_name: String,
}

impl DynamicGrpcClient {
    /// Create a new dynamic gRPC client
    pub fn new(channel: Channel, service_name: String) -> Self {
        debug!(service = %service_name, "Creating dynamic gRPC client");
        Self {
            channel,
            service_name,
        }
    }

    /// Call a gRPC method dynamically with JSON payload
    ///
    /// This method:
    /// 1. Converts JSON payload to protobuf Any type
    /// 2. Injects trace headers into gRPC metadata
    /// 3. Makes the gRPC call
    /// 4. Converts the response back to JSON
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
            "Making dynamic gRPC call"
        );

        // Convert JSON to protobuf Any
        let any_request = self.json_to_any(&json_payload, method)?;

        // Create gRPC request with metadata
        let mut request = Request::new(any_request);

        // Inject trace headers into gRPC metadata
        let metadata = request.metadata_mut();
        for (key, value) in trace_headers {
            // Convert string key to MetadataKey
            if let Ok(metadata_key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
                // Convert string value to MetadataValue
                if let Ok(metadata_value) = tonic::metadata::MetadataValue::try_from(&value) {
                    metadata.insert(metadata_key, metadata_value);
                    debug!(header = %key, "Injected trace header into gRPC metadata");
                } else {
                    warn!(header = %key, "Failed to parse metadata value");
                }
            } else {
                warn!(header = %key, "Failed to parse metadata key");
            }
        }

        // Make the gRPC call
        let full_method = format!("/{}/{}", self.service_name, method);
        debug!(full_method = %full_method, "Invoking gRPC method");

        let response = self.invoke_unary(full_method, request).await?;

        // Convert response back to JSON
        let json_response = self.any_to_json(response.into_inner())?;

        debug!(
            service = %self.service_name,
            method = %method,
            response_size = json_response.len(),
            "Dynamic gRPC call completed successfully"
        );

        Ok(json_response)
    }

    /// Invoke a unary gRPC call
    async fn invoke_unary(
        &mut self,
        method: String,
        request: Request<Any>,
    ) -> Result<Response<Any>, GrpcError> {
        use tonic::client::Grpc;

        let mut grpc = Grpc::new(self.channel.clone());
        let codec = ProstCodec::<Any, Any>::default();

        // CRITICAL: Must call ready() before unary() to ensure tower buffer is ready
        // This prevents "buffer full; poll_ready must be called first" panic
        grpc.ready()
            .await
            .map_err(|e| {
                error!(
                    service = %self.service_name,
                    error = %e,
                    "Failed to ready gRPC client"
                );
                GrpcError::CallFailed(format!("Failed to ready gRPC client: {}", e))
            })?;

        grpc.unary(request, method.parse().unwrap(), codec)
            .await
            .map_err(|status| {
                error!(
                    service = %self.service_name,
                    status_code = ?status.code(),
                    message = %status.message(),
                    "gRPC call failed"
                );
                GrpcError::CallFailed(format!(
                    "gRPC call failed: {} - {}",
                    status.code(),
                    status.message()
                ))
            })
    }

    /// Convert JSON payload to protobuf Any type
    fn json_to_any(&self, json_bytes: &[u8], method: &str) -> Result<Any, GrpcError> {
        // Parse JSON
        let json_value: serde_json::Value = serde_json::from_slice(json_bytes)
            .map_err(|e| GrpcError::ConversionError(format!("Failed to parse JSON: {}", e)))?;

        // For now, we'll serialize the JSON as a string in the Any type
        // In a production system, you'd want to use proper protobuf encoding
        let json_string = serde_json::to_string(&json_value)
            .map_err(|e| GrpcError::ConversionError(format!("Failed to serialize JSON: {}", e)))?;

        // Create Any type with JSON payload
        // Note: This is a simplified approach. For production, you'd want to:
        // 1. Use the actual protobuf message types
        // 2. Convert JSON to protobuf using prost-reflect or similar
        let type_url = format!("type.googleapis.com/{}/{}", self.service_name, method);

        Ok(Any {
            type_url,
            value: json_string.into_bytes(),
        })
    }

    /// Convert protobuf Any type back to JSON
    fn any_to_json(&self, any: Any) -> Result<Vec<u8>, GrpcError> {
        // For now, we'll assume the value is JSON string
        // In production, you'd want to properly decode the protobuf message
        let json_string = String::from_utf8(any.value)
            .map_err(|e| GrpcError::ConversionError(format!("Failed to decode response: {}", e)))?;

        // Validate it's valid JSON
        let json_value: serde_json::Value = serde_json::from_str(&json_string)
            .map_err(|e| GrpcError::ConversionError(format!("Response is not valid JSON: {}", e)))?;

        // Serialize back to bytes
        serde_json::to_vec(&json_value)
            .map_err(|e| GrpcError::ConversionError(format!("Failed to serialize response: {}", e)))
    }
}

/// Map gRPC status codes to HTTP status codes
pub fn grpc_status_to_http(status: &Status) -> u16 {
    use tonic::Code;

    match status.code() {
        Code::Ok => 200,
        Code::Cancelled => 499, // Client closed request
        Code::Unknown => 500,
        Code::InvalidArgument => 400,
        Code::DeadlineExceeded => 504,
        Code::NotFound => 404,
        Code::AlreadyExists => 409,
        Code::PermissionDenied => 403,
        Code::ResourceExhausted => 429,
        Code::FailedPrecondition => 400,
        Code::Aborted => 409,
        Code::OutOfRange => 400,
        Code::Unimplemented => 501,
        Code::Internal => 500,
        Code::Unavailable => 503,
        Code::DataLoss => 500,
        Code::Unauthenticated => 401,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_status_mapping() {
        use tonic::Code;

        assert_eq!(grpc_status_to_http(&Status::new(Code::Ok, "")), 200);
        assert_eq!(grpc_status_to_http(&Status::new(Code::NotFound, "")), 404);
        assert_eq!(grpc_status_to_http(&Status::new(Code::InvalidArgument, "")), 400);
        assert_eq!(grpc_status_to_http(&Status::new(Code::Unauthenticated, "")), 401);
        assert_eq!(grpc_status_to_http(&Status::new(Code::PermissionDenied, "")), 403);
        assert_eq!(grpc_status_to_http(&Status::new(Code::Internal, "")), 500);
        assert_eq!(grpc_status_to_http(&Status::new(Code::Unavailable, "")), 503);
    }
}
