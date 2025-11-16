use bytes::{Buf, BufMut};
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage};
use prost_types::FileDescriptorSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tonic::transport::Channel;
use tonic::{codec::Codec, Request, Response, Status};
use tracing::{debug, error, warn};

use super::types::GrpcError;

/// Dynamic gRPC client that can call any gRPC method without code generation
/// Uses prost-reflect for proper JSON-to-protobuf conversion
pub struct DynamicGrpcClient {
    channel: Channel,
    service_name: String,
    descriptor_pool: Option<Arc<DescriptorPool>>,
}

impl DynamicGrpcClient {
    /// Create a new dynamic gRPC client
    pub fn new(channel: Channel, service_name: String) -> Self {
        debug!(service = %service_name, "Creating dynamic gRPC client");
        Self {
            channel,
            service_name,
            descriptor_pool: None,
        }
    }

    /// Set the descriptor pool for this client (obtained from gRPC reflection)
    pub fn with_descriptor_pool(mut self, pool: Arc<DescriptorPool>) -> Self {
        self.descriptor_pool = Some(pool);
        self
    }

    /// Call a gRPC method dynamically with JSON payload
    ///
    /// This method:
    /// 1. Converts JSON payload to protobuf using prost-reflect
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
            "Making dynamic gRPC call with prost-reflect"
        );

        // Get descriptor pool - for now we'll fetch it via reflection if not set
        let pool = if let Some(pool) = &self.descriptor_pool {
            pool.clone()
        } else {
            // Fetch descriptors via gRPC reflection
            let pool = self.fetch_descriptors().await?;
            let pool_arc = Arc::new(pool);
            self.descriptor_pool = Some(pool_arc.clone());
            pool_arc
        };

        // Convert JSON to protobuf bytes using prost-reflect
        let request_bytes = self.json_to_protobuf(&json_payload, method, &pool)?;

        // Create gRPC request with raw bytes
        let mut request = Request::new(request_bytes);

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

        let response_bytes = self.invoke_unary_bytes(full_method, request).await?;

        // Convert response back to JSON using prost-reflect
        let json_response = self.protobuf_to_json(&response_bytes, method, &pool)?;

        debug!(
            service = %self.service_name,
            method = %method,
            response_size = json_response.len(),
            "Dynamic gRPC call completed successfully"
        );

        Ok(json_response)
    }

    /// Fetch service descriptors via gRPC Server Reflection
    async fn fetch_descriptors(&self) -> Result<DescriptorPool, GrpcError> {
        use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
        use tonic_reflection::pb::v1::server_reflection_request::MessageRequest;
        use tonic_reflection::pb::v1::ServerReflectionRequest;

        debug!(service = %self.service_name, "Fetching service descriptors via gRPC reflection");

        let mut client = ServerReflectionClient::new(self.channel.clone());

        // Create a stream request
        let request_stream = tokio_stream::iter(vec![
            ServerReflectionRequest {
                host: String::new(),
                message_request: Some(MessageRequest::FileContainingSymbol(
                    self.service_name.clone(),
                )),
            },
        ]);

        let mut response_stream = client
            .server_reflection_info(request_stream)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to call server reflection");
                GrpcError::CallFailed(format!("Reflection call failed: {}", e))
            })?
            .into_inner();

        // Collect file descriptor protos
        let mut file_descriptor_protos = Vec::new();

        while let Some(response) = response_stream.message().await.map_err(|e| {
            error!(error = %e, "Failed to read reflection response");
            GrpcError::CallFailed(format!("Failed to read reflection response: {}", e))
        })? {
            if let Some(response) = response.message_response {
                use tonic_reflection::pb::v1::server_reflection_response::MessageResponse;
                match response {
                    MessageResponse::FileDescriptorResponse(fd_response) => {
                        for fd_bytes in fd_response.file_descriptor_proto {
                            file_descriptor_protos.push(fd_bytes);
                        }
                    }
                    MessageResponse::ErrorResponse(err) => {
                        error!(error_code = err.error_code, message = %err.error_message, "Reflection error");
                        return Err(GrpcError::CallFailed(format!(
                            "Reflection error: {}",
                            err.error_message
                        )));
                    }
                    _ => {}
                }
            }
        }

        // Build descriptor pool from file descriptors
        let mut file_descriptor_set = FileDescriptorSet { file: Vec::new() };
        for fd_bytes in file_descriptor_protos {
            let fd = prost_types::FileDescriptorProto::decode(&fd_bytes[..]).map_err(|e| {
                error!(error = %e, "Failed to decode file descriptor");
                GrpcError::ConversionError(format!("Failed to decode file descriptor: {}", e))
            })?;
            file_descriptor_set.file.push(fd);
        }

        let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set).map_err(|e| {
            error!(error = %e, "Failed to build descriptor pool");
            GrpcError::ConversionError(format!("Failed to build descriptor pool: {}", e))
        })?;

        debug!(service = %self.service_name, "Successfully built descriptor pool");
        Ok(pool)
    }

    /// Invoke a unary gRPC call with raw bytes
    async fn invoke_unary_bytes(
        &mut self,
        method: String,
        request: Request<Vec<u8>>,
    ) -> Result<Vec<u8>, GrpcError> {
        use tonic::client::Grpc;

        let mut grpc = Grpc::new(self.channel.clone());
        let codec = BytesCodec::default();

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

        let response = grpc.unary(request, method.parse().unwrap(), codec)
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
            })?;

        Ok(response.into_inner())
    }

    /// Convert JSON payload to protobuf bytes using prost-reflect
    fn json_to_protobuf(
        &self,
        json_bytes: &[u8],
        method: &str,
        pool: &DescriptorPool,
    ) -> Result<Vec<u8>, GrpcError> {
        // Find the method descriptor
        let service_desc = pool
            .get_service_by_name(&self.service_name)
            .ok_or_else(|| {
                error!(service = %self.service_name, "Service not found in descriptor pool");
                GrpcError::ConversionError(format!("Service {} not found", self.service_name))
            })?;

        let method_desc = service_desc.methods().find(|m| m.name() == method).ok_or_else(|| {
            error!(method = %method, "Method not found in service descriptor");
            GrpcError::ConversionError(format!("Method {} not found", method))
        })?;

        let input_desc = method_desc.input();

        // Parse JSON to remove metadata fields that aren't part of the protobuf message
        let mut json_value: serde_json::Value = serde_json::from_slice(json_bytes).map_err(|e| {
            error!(error = %e, "JSON payload is not valid JSON");
            GrpcError::ConversionError(format!("Invalid JSON: {}", e))
        })?;

        // Remove gateway metadata fields that aren't part of the protobuf schema
        if let Some(obj) = json_value.as_object_mut() {
            obj.remove("_auth_user_id");
            obj.remove("_auth_roles");
            obj.remove("_trace");
        }

        // Convert back to JSON string for deserialization
        let json_str = serde_json::to_string(&json_value).map_err(|e| {
            error!(error = %e, "Failed to serialize cleaned JSON");
            GrpcError::ConversionError(format!("JSON serialization failed: {}", e))
        })?;

        // Create deserializer from JSON string
        let mut deserializer = serde_json::Deserializer::from_str(&json_str);
        
        // Deserialize JSON into dynamic message using prost-reflect 0.14 API
        let message = DynamicMessage::deserialize(input_desc.clone(), &mut deserializer).map_err(|e| {
            error!(error = %e, "Failed to deserialize JSON to protobuf");
            GrpcError::ConversionError(format!("JSON to protobuf conversion failed: {}", e))
        })?;

        // Ensure all JSON was consumed
        deserializer.end().map_err(|e| {
            error!(error = %e, "JSON has trailing data");
            GrpcError::ConversionError(format!("Invalid JSON: {}", e))
        })?;

        // Encode to protobuf bytes
        let mut buf = Vec::new();
        message.encode(&mut buf).map_err(|e| {
            error!(error = %e, "Failed to encode protobuf message");
            GrpcError::ConversionError(format!("Protobuf encoding failed: {}", e))
        })?;

        debug!(
            service = %self.service_name,
            method = %method,
            protobuf_size = buf.len(),
            "Successfully converted JSON to protobuf"
        );

        Ok(buf)
    }

    /// Convert protobuf bytes back to JSON using prost-reflect
    fn protobuf_to_json(
        &self,
        protobuf_bytes: &[u8],
        method: &str,
        pool: &DescriptorPool,
    ) -> Result<Vec<u8>, GrpcError> {
        // Find the method descriptor
        let service_desc = pool
            .get_service_by_name(&self.service_name)
            .ok_or_else(|| {
                error!(service = %self.service_name, "Service not found in descriptor pool");
                GrpcError::ConversionError(format!("Service {} not found", self.service_name))
            })?;

        let method_desc = service_desc.methods().find(|m| m.name() == method).ok_or_else(|| {
            error!(method = %method, "Method not found in service descriptor");
            GrpcError::ConversionError(format!("Method {} not found", method))
        })?;

        let output_desc = method_desc.output();

        // Decode protobuf message
        let message = DynamicMessage::decode(output_desc.clone(), protobuf_bytes).map_err(|e| {
            error!(error = %e, "Failed to decode protobuf message");
            GrpcError::ConversionError(format!("Protobuf decoding failed: {}", e))
        })?;

        // Serialize to JSON using prost-reflect 0.14 with serde feature
        let json_string = serde_json::to_string(&message).map_err(|e| {
            error!(error = %e, "Failed to serialize protobuf to JSON");
            GrpcError::ConversionError(format!("Protobuf to JSON conversion failed: {}", e))
        })?;

        let json_bytes = json_string.into_bytes();

        debug!(
            service = %self.service_name,
            method = %method,
            json_size = json_bytes.len(),
            "Successfully converted protobuf to JSON"
        );

        Ok(json_bytes)
    }
}

/// Simple codec for raw bytes
#[derive(Debug, Clone, Default)]
struct BytesCodec;

impl Codec for BytesCodec {
    type Encode = Vec<u8>;
    type Decode = Vec<u8>;

    type Encoder = BytesEncoder;
    type Decoder = BytesDecoder;

    fn encoder(&mut self) -> Self::Encoder {
        BytesEncoder
    }

    fn decoder(&mut self) -> Self::Decoder {
        BytesDecoder
    }
}

#[derive(Debug, Clone, Default)]
struct BytesEncoder;

impl tonic::codec::Encoder for BytesEncoder {
    type Item = Vec<u8>;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, buf: &mut tonic::codec::EncodeBuf<'_>) -> Result<(), Self::Error> {
        buf.put_slice(&item);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct BytesDecoder;

impl tonic::codec::Decoder for BytesDecoder {
    type Item = Vec<u8>;
    type Error = Status;

    fn decode(&mut self, buf: &mut tonic::codec::DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let chunk = buf.chunk();
        if chunk.is_empty() {
            return Ok(None);
        }
        let bytes = chunk.to_vec();
        buf.advance(chunk.len());
        Ok(Some(bytes))
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
