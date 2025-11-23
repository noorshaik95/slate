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
        let request_stream = tokio_stream::iter(vec![ServerReflectionRequest {
            host: String::new(),
            message_request: Some(MessageRequest::FileContainingSymbol(
                self.service_name.clone(),
            )),
        }]);

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

        // Remove any non-standard google protobuf files that might conflict
        // Some reflection implementations return a google_protobuf.proto file that conflicts
        // with the standard google/protobuf/*.proto files
        file_descriptor_set.file.retain(|f| {
            let name = f.name.as_deref().unwrap_or("");
            // Keep all files except google_protobuf.proto (without the slash)
            name != "google_protobuf.proto"
        });

        // Add well-known type descriptors to ensure they're available
        // This is necessary because some gRPC reflection implementations don't include them
        add_well_known_types(&mut file_descriptor_set);

        // Fix missing dependencies in proto files
        // Some reflection implementations don't include dependency information
        fix_proto_dependencies(&mut file_descriptor_set);

        // Log FileDescriptorSet contents before building pool for diagnostics
        debug!(
            service = %self.service_name,
            file_count = file_descriptor_set.file.len(),
            "Building descriptor pool from FileDescriptorSet"
        );

        for (idx, file) in file_descriptor_set.file.iter().enumerate() {
            let message_names: Vec<String> = file
                .message_type
                .iter()
                .filter_map(|m| m.name.clone())
                .collect();

            let dependencies: Vec<String> = file.dependency.clone();

            warn!(
                index = idx,
                file_name = ?file.name,
                package = ?file.package,
                message_count = file.message_type.len(),
                service_count = file.service.len(),
                messages = ?message_names,
                dependencies = ?dependencies,
                "FileDescriptor in set"
            );
        }

        let pool =
            DescriptorPool::from_file_descriptor_set(file_descriptor_set.clone()).map_err(|e| {
                error!(
                    service = %self.service_name,
                    error = %e,
                    "Failed to build descriptor pool"
                );

                // Log all available files for debugging
                let available_files: Vec<String> = file_descriptor_set
                    .file
                    .iter()
                    .filter_map(|f| f.name.clone())
                    .collect();

                let available_packages: Vec<String> = file_descriptor_set
                    .file
                    .iter()
                    .filter_map(|f| f.package.clone())
                    .collect();

                error!(
                    available_files = ?available_files,
                    available_packages = ?available_packages,
                    "Available files and packages in FileDescriptorSet"
                );

                GrpcError::ConversionError(format!(
                    "Failed to build descriptor pool: {}. Available files: {:?}. Service: {}",
                    e, available_files, self.service_name
                ))
            })?;

        // Log success with pool statistics
        let service_count = pool.services().count();
        let message_count = pool.all_messages().count();

        debug!(
            service = %self.service_name,
            service_count = service_count,
            message_count = message_count,
            "Successfully built descriptor pool"
        );

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
        grpc.ready().await.map_err(|e| {
            error!(
                service = %self.service_name,
                error = %e,
                "Failed to ready gRPC client"
            );
            GrpcError::CallFailed(format!("Failed to ready gRPC client: {}", e))
        })?;

        let response = grpc
            .unary(request, method.parse().unwrap(), codec)
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

        let method_desc = service_desc
            .methods()
            .find(|m| m.name() == method)
            .ok_or_else(|| {
                error!(method = %method, "Method not found in service descriptor");
                GrpcError::ConversionError(format!("Method {} not found", method))
            })?;

        let input_desc = method_desc.input();

        // Parse JSON to remove metadata fields that aren't part of the protobuf message
        let mut json_value: serde_json::Value =
            serde_json::from_slice(json_bytes).map_err(|e| {
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
        let message =
            DynamicMessage::deserialize(input_desc.clone(), &mut deserializer).map_err(|e| {
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

        let method_desc = service_desc
            .methods()
            .find(|m| m.name() == method)
            .ok_or_else(|| {
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

    fn encode(
        &mut self,
        item: Self::Item,
        buf: &mut tonic::codec::EncodeBuf<'_>,
    ) -> Result<(), Self::Error> {
        buf.put_slice(&item);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct BytesDecoder;

impl tonic::codec::Decoder for BytesDecoder {
    type Item = Vec<u8>;
    type Error = Status;

    fn decode(
        &mut self,
        buf: &mut tonic::codec::DecodeBuf<'_>,
    ) -> Result<Option<Self::Item>, Self::Error> {
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

/// Add well-known type descriptors to the file descriptor set
/// This ensures that google.protobuf.Timestamp, Empty, Duration, and other well-known types
/// are available even if the gRPC reflection response doesn't include them
fn add_well_known_types(file_descriptor_set: &mut FileDescriptorSet) {
    use prost_types::{field_descriptor_proto, FieldDescriptorProto, FileDescriptorProto};

    // Track which well-known type files already exist
    let existing_files: std::collections::HashSet<String> = file_descriptor_set
        .file
        .iter()
        .filter_map(|f| f.name.clone())
        .collect();

    debug!(
        existing_files = ?existing_files,
        "Checking for well-known types in FileDescriptorSet"
    );

    let mut well_known_files = Vec::new();

    // Only add well-known types if the exact standard file name doesn't exist
    // We don't check for types in other files because they might not be properly structured

    // Add google/protobuf/timestamp.proto if not present
    if !existing_files.contains("google/protobuf/timestamp.proto") {
        let timestamp_descriptor = prost_types::DescriptorProto {
            name: Some("Timestamp".to_string()),
            field: vec![
                FieldDescriptorProto {
                    name: Some("seconds".to_string()),
                    number: Some(1),
                    r#type: Some(field_descriptor_proto::Type::Int64 as i32),
                    ..Default::default()
                },
                FieldDescriptorProto {
                    name: Some("nanos".to_string()),
                    number: Some(2),
                    r#type: Some(field_descriptor_proto::Type::Int32 as i32),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let timestamp_file = FileDescriptorProto {
            name: Some("google/protobuf/timestamp.proto".to_string()),
            package: Some("google.protobuf".to_string()),
            message_type: vec![timestamp_descriptor],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };

        well_known_files.push(timestamp_file);
        debug!("Adding google/protobuf/timestamp.proto to FileDescriptorSet");
    }

    // Add google/protobuf/empty.proto if not present
    if !existing_files.contains("google/protobuf/empty.proto") {
        let empty_descriptor = prost_types::DescriptorProto {
            name: Some("Empty".to_string()),
            field: vec![],
            ..Default::default()
        };

        let empty_file = FileDescriptorProto {
            name: Some("google/protobuf/empty.proto".to_string()),
            package: Some("google.protobuf".to_string()),
            message_type: vec![empty_descriptor],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };

        well_known_files.push(empty_file);
        debug!("Adding google/protobuf/empty.proto to FileDescriptorSet");
    }

    // Add google/protobuf/duration.proto if not present
    if !existing_files.contains("google/protobuf/duration.proto") {
        let duration_descriptor = prost_types::DescriptorProto {
            name: Some("Duration".to_string()),
            field: vec![
                FieldDescriptorProto {
                    name: Some("seconds".to_string()),
                    number: Some(1),
                    r#type: Some(field_descriptor_proto::Type::Int64 as i32),
                    ..Default::default()
                },
                FieldDescriptorProto {
                    name: Some("nanos".to_string()),
                    number: Some(2),
                    r#type: Some(field_descriptor_proto::Type::Int32 as i32),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let duration_file = FileDescriptorProto {
            name: Some("google/protobuf/duration.proto".to_string()),
            package: Some("google.protobuf".to_string()),
            message_type: vec![duration_descriptor],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };

        well_known_files.push(duration_file);
        debug!("Adding google/protobuf/duration.proto to FileDescriptorSet");
    }

    // Insert well-known types at the beginning of the file list
    // This ensures they're available when other files reference them
    if !well_known_files.is_empty() {
        debug!(
            count = well_known_files.len(),
            "Inserting well-known type files at beginning of FileDescriptorSet"
        );

        // Prepend well-known files to the beginning
        let mut new_files = well_known_files;
        new_files.extend(file_descriptor_set.file.drain(..));
        file_descriptor_set.file = new_files;
    }
}

/// Fix missing dependencies in proto files
/// Some gRPC reflection implementations don't include dependency information,
/// which causes the descriptor pool to fail when resolving imports
fn fix_proto_dependencies(file_descriptor_set: &mut FileDescriptorSet) {
    use prost_types::FileDescriptorProto;

    // Build a map of available files
    let available_files: std::collections::HashSet<String> = file_descriptor_set
        .file
        .iter()
        .filter_map(|f| f.name.clone())
        .collect();

    debug!(
        available_files = ?available_files,
        "Fixing proto dependencies"
    );

    // For each file, check if it needs well-known type dependencies
    for file in file_descriptor_set.file.iter_mut() {
        let file_name = file.name.as_deref().unwrap_or("");

        // Skip well-known type files themselves
        if file_name.starts_with("google/protobuf/") {
            continue;
        }

        // Check if this file uses Timestamp, Empty, or Duration types
        let uses_timestamp = file_uses_type(file, "Timestamp");
        let uses_empty = file_uses_type(file, "Empty");
        let uses_duration = file_uses_type(file, "Duration");

        // Add missing dependencies
        if uses_timestamp && available_files.contains("google/protobuf/timestamp.proto") {
            if !file
                .dependency
                .contains(&"google/protobuf/timestamp.proto".to_string())
            {
                file.dependency
                    .push("google/protobuf/timestamp.proto".to_string());
                debug!(
                    file = file_name,
                    "Added google/protobuf/timestamp.proto dependency"
                );
            }
        }

        if uses_empty && available_files.contains("google/protobuf/empty.proto") {
            if !file
                .dependency
                .contains(&"google/protobuf/empty.proto".to_string())
            {
                file.dependency
                    .push("google/protobuf/empty.proto".to_string());
                debug!(
                    file = file_name,
                    "Added google/protobuf/empty.proto dependency"
                );
            }
        }

        if uses_duration && available_files.contains("google/protobuf/duration.proto") {
            if !file
                .dependency
                .contains(&"google/protobuf/duration.proto".to_string())
            {
                file.dependency
                    .push("google/protobuf/duration.proto".to_string());
                debug!(
                    file = file_name,
                    "Added google/protobuf/duration.proto dependency"
                );
            }
        }
    }
}

/// Check if a file uses a specific type (by checking field types in messages and service methods)
fn file_uses_type(file: &prost_types::FileDescriptorProto, type_name: &str) -> bool {
    use prost_types::field_descriptor_proto;

    // Check message fields
    for message in &file.message_type {
        for field in &message.field {
            // Check if field type_name contains the type we're looking for
            if let Some(field_type_name) = &field.type_name {
                // Type names in protobuf are fully qualified, e.g., ".google.protobuf.Timestamp"
                if field_type_name.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }
        }

        // Also check nested messages recursively
        if message_uses_type_recursive(message, type_name) {
            return true;
        }
    }

    // Check service method input/output types
    for service in &file.service {
        for method in &service.method {
            // Check input type
            if let Some(input_type) = &method.input_type {
                if input_type.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }

            // Check output type
            if let Some(output_type) = &method.output_type {
                if output_type.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }
        }
    }

    false
}

/// Recursively check nested messages for type usage
fn message_uses_type_recursive(message: &prost_types::DescriptorProto, type_name: &str) -> bool {
    for nested in &message.nested_type {
        for field in &nested.field {
            if let Some(field_type_name) = &field.type_name {
                if field_type_name.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }
        }

        // Recurse further
        if message_uses_type_recursive(nested, type_name) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_status_mapping() {
        use tonic::Code;

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
