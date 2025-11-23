//! Descriptor pool management for dynamic gRPC clients.
//!
//! Handles fetching service descriptors via gRPC Server Reflection and building
//! descriptor pools with proper well-known type support.

use prost::Message;
use prost_reflect::DescriptorPool;
use prost_types::FileDescriptorSet;
use tonic::transport::Channel;
use tracing::{debug, error, warn};

use crate::grpc::types::GrpcError;

/// Fetch service descriptors via gRPC Server Reflection.
///
/// This function queries the gRPC server for its service descriptors using the
/// Server Reflection protocol, then builds a descriptor pool that can be used
/// for dynamic message conversion.
///
/// # Arguments
///
/// * `channel` - The gRPC channel to use for reflection
/// * `service_name` - The fully qualified service name to fetch descriptors for
///
/// # Returns
///
/// A descriptor pool containing all necessary type information
pub async fn fetch_descriptors(
    channel: &Channel,
    service_name: &str,
) -> Result<DescriptorPool, GrpcError> {
    use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
    use tonic_reflection::pb::v1::server_reflection_request::MessageRequest;
    use tonic_reflection::pb::v1::ServerReflectionRequest;

    debug!(service = %service_name, "Fetching service descriptors via gRPC reflection");

    let mut client = ServerReflectionClient::new(channel.clone());

    // Create reflection request
    let request_stream = tokio_stream::iter(vec![ServerReflectionRequest {
        host: String::new(),
        message_request: Some(MessageRequest::FileContainingSymbol(
            service_name.to_string(),
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
    let file_descriptor_protos = collect_file_descriptors(&mut response_stream).await?;

    // Build and return descriptor pool
    build_descriptor_pool(file_descriptor_protos, service_name).await
}

/// Collect file descriptor protos from the reflection response stream.
async fn collect_file_descriptors(
    response_stream: &mut tonic::Streaming<tonic_reflection::pb::v1::ServerReflectionResponse>,
) -> Result<Vec<Vec<u8>>, GrpcError> {
    use tonic_reflection::pb::v1::server_reflection_response::MessageResponse;

    let mut file_descriptor_protos = Vec::new();

    while let Some(response) = response_stream.message().await.map_err(|e| {
        error!(error = %e, "Failed to read reflection response");
        GrpcError::CallFailed(format!("Failed to read reflection response: {}", e))
    })? {
        if let Some(response) = response.message_response {
            match response {
                MessageResponse::FileDescriptorResponse(fd_response) => {
                    file_descriptor_protos.extend(fd_response.file_descriptor_proto);
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

    Ok(file_descriptor_protos)
}

/// Build a descriptor pool from file descriptor protos.
async fn build_descriptor_pool(
    file_descriptor_protos: Vec<Vec<u8>>,
    service_name: &str,
) -> Result<DescriptorPool, GrpcError> {
    // Decode and prepare file descriptor set
    let mut file_descriptor_set = decode_file_descriptors(file_descriptor_protos)?;

    // Clean up conflicting files
    remove_conflicting_files(&mut file_descriptor_set);

    // Add well-known types and fix dependencies
    add_well_known_types(&mut file_descriptor_set);
    fix_proto_dependencies(&mut file_descriptor_set);

    // Log diagnostics
    log_descriptor_set_info(&file_descriptor_set, service_name);

    // Build and validate descriptor pool
    let pool = create_descriptor_pool(file_descriptor_set, service_name)?;

    // Log success metrics
    log_pool_success(&pool, service_name);

    Ok(pool)
}

/// Decode file descriptor protos into a FileDescriptorSet.
fn decode_file_descriptors(
    file_descriptor_protos: Vec<Vec<u8>>,
) -> Result<FileDescriptorSet, GrpcError> {
    let mut file_descriptor_set = FileDescriptorSet { file: Vec::new() };

    for fd_bytes in file_descriptor_protos {
        let fd = prost_types::FileDescriptorProto::decode(&fd_bytes[..]).map_err(|e| {
            error!(error = %e, "Failed to decode file descriptor");
            GrpcError::ConversionError(format!("Failed to decode file descriptor: {}", e))
        })?;
        file_descriptor_set.file.push(fd);
    }

    Ok(file_descriptor_set)
}

/// Remove conflicting proto files from the descriptor set.
fn remove_conflicting_files(file_descriptor_set: &mut FileDescriptorSet) {
    file_descriptor_set.file.retain(|f| {
        let name = f.name.as_deref().unwrap_or("");
        name != "google_protobuf.proto"
    });
}

/// Create a descriptor pool from a FileDescriptorSet with error handling.
fn create_descriptor_pool(
    file_descriptor_set: FileDescriptorSet,
    service_name: &str,
) -> Result<DescriptorPool, GrpcError> {
    DescriptorPool::from_file_descriptor_set(file_descriptor_set.clone()).map_err(|e| {
        error!(
            service = %service_name,
            error = %e,
            "Failed to build descriptor pool"
        );

        let available_files: Vec<String> = file_descriptor_set
            .file
            .iter()
            .filter_map(|f| f.name.clone())
            .collect();

        error!(
            available_files = ?available_files,
            "Available files in FileDescriptorSet"
        );

        GrpcError::ConversionError(format!(
            "Failed to build descriptor pool: {}. Available files: {:?}. Service: {}",
            e, available_files, service_name
        ))
    })
}

/// Log successful descriptor pool creation metrics.
fn log_pool_success(pool: &DescriptorPool, service_name: &str) {
    let service_count = pool.services().count();
    let message_count = pool.all_messages().count();

    debug!(
        service = %service_name,
        service_count = service_count,
        message_count = message_count,
        "Successfully built descriptor pool"
    );
}

/// Log information about the descriptor set for diagnostics.
fn log_descriptor_set_info(file_descriptor_set: &FileDescriptorSet, service_name: &str) {
    debug!(
        service = %service_name,
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
}

/// Add well-known type descriptors to the file descriptor set.
///
/// Ensures that google.protobuf.Timestamp, Empty, and Duration are available
/// even if the gRPC reflection response doesn't include them.
fn add_well_known_types(file_descriptor_set: &mut FileDescriptorSet) {
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

    // Add Timestamp if missing
    if !existing_files.contains("google/protobuf/timestamp.proto") {
        well_known_files.push(create_timestamp_descriptor());
        debug!("Adding google/protobuf/timestamp.proto to FileDescriptorSet");
    }

    // Add Empty if missing
    if !existing_files.contains("google/protobuf/empty.proto") {
        well_known_files.push(create_empty_descriptor());
        debug!("Adding google/protobuf/empty.proto to FileDescriptorSet");
    }

    // Add Duration if missing
    if !existing_files.contains("google/protobuf/duration.proto") {
        well_known_files.push(create_duration_descriptor());
        debug!("Adding google/protobuf/duration.proto to FileDescriptorSet");
    }

    // Prepend well-known files
    if !well_known_files.is_empty() {
        debug!(
            count = well_known_files.len(),
            "Inserting well-known type files at beginning of FileDescriptorSet"
        );

        let mut new_files = well_known_files;
        new_files.append(&mut file_descriptor_set.file);
        file_descriptor_set.file = new_files;
    }
}

/// Create the Timestamp well-known type descriptor.
fn create_timestamp_descriptor() -> prost_types::FileDescriptorProto {
    use prost_types::{field_descriptor_proto, FieldDescriptorProto, FileDescriptorProto};

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

    FileDescriptorProto {
        name: Some("google/protobuf/timestamp.proto".to_string()),
        package: Some("google.protobuf".to_string()),
        message_type: vec![timestamp_descriptor],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    }
}

/// Create the Empty well-known type descriptor.
fn create_empty_descriptor() -> prost_types::FileDescriptorProto {
    use prost_types::FileDescriptorProto;

    let empty_descriptor = prost_types::DescriptorProto {
        name: Some("Empty".to_string()),
        field: vec![],
        ..Default::default()
    };

    FileDescriptorProto {
        name: Some("google/protobuf/empty.proto".to_string()),
        package: Some("google.protobuf".to_string()),
        message_type: vec![empty_descriptor],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    }
}

/// Create the Duration well-known type descriptor.
fn create_duration_descriptor() -> prost_types::FileDescriptorProto {
    use prost_types::{field_descriptor_proto, FieldDescriptorProto, FileDescriptorProto};

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

    FileDescriptorProto {
        name: Some("google/protobuf/duration.proto".to_string()),
        package: Some("google.protobuf".to_string()),
        message_type: vec![duration_descriptor],
        syntax: Some("proto3".to_string()),
        ..Default::default()
    }
}

/// Fix missing dependencies in proto files.
///
/// Some gRPC reflection implementations don't include dependency information,
/// which causes the descriptor pool to fail when resolving imports.
fn fix_proto_dependencies(file_descriptor_set: &mut FileDescriptorSet) {
    let available_files: std::collections::HashSet<String> = file_descriptor_set
        .file
        .iter()
        .filter_map(|f| f.name.clone())
        .collect();

    debug!(
        available_files = ?available_files,
        "Fixing proto dependencies"
    );

    for file in file_descriptor_set.file.iter_mut() {
        let file_name = file.name.as_deref().unwrap_or("");

        // Skip well-known type files themselves
        if file_name.starts_with("google/protobuf/") {
            continue;
        }

        // Check and add missing dependencies
        add_missing_dependency(
            file,
            &available_files,
            "Timestamp",
            "google/protobuf/timestamp.proto",
        );
        add_missing_dependency(
            file,
            &available_files,
            "Empty",
            "google/protobuf/empty.proto",
        );
        add_missing_dependency(
            file,
            &available_files,
            "Duration",
            "google/protobuf/duration.proto",
        );
    }
}

/// Add a missing dependency to a file if it uses the specified type.
fn add_missing_dependency(
    file: &mut prost_types::FileDescriptorProto,
    available_files: &std::collections::HashSet<String>,
    type_name: &str,
    dependency_path: &str,
) {
    if file_uses_type(file, type_name) && available_files.contains(dependency_path)
        && !file.dependency.contains(&dependency_path.to_string()) {
            file.dependency.push(dependency_path.to_string());
            debug!(
                file = ?file.name,
                dependency = dependency_path,
                "Added missing dependency"
            );
        }
}

/// Check if a file uses a specific type.
fn file_uses_type(file: &prost_types::FileDescriptorProto, type_name: &str) -> bool {
    // Check message fields
    for message in &file.message_type {
        if message_uses_type(message, type_name) {
            return true;
        }
    }

    // Check service method input/output types
    for service in &file.service {
        for method in &service.method {
            if let Some(input_type) = &method.input_type {
                if input_type.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }

            if let Some(output_type) = &method.output_type {
                if output_type.ends_with(&format!(".{}", type_name)) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a message uses a specific type (recursively).
fn message_uses_type(message: &prost_types::DescriptorProto, type_name: &str) -> bool {
    // Check fields
    for field in &message.field {
        if let Some(field_type_name) = &field.type_name {
            if field_type_name.ends_with(&format!(".{}", type_name)) {
                return true;
            }
        }
    }

    // Check nested messages recursively
    for nested in &message.nested_type {
        if message_uses_type(nested, type_name) {
            return true;
        }
    }

    false
}
