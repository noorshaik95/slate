use tonic::transport::Channel;
use tonic_reflection::pb::v1::server_reflection_client::ServerReflectionClient;
use tonic_reflection::pb::v1::{
    server_reflection_request::MessageRequest, server_reflection_response::MessageResponse,
    ServerReflectionRequest, ServerReflectionResponse,
};
use tracing::{debug, warn};

use super::{MethodDescriptor, ReflectionError};

/// Client for querying gRPC services using Server Reflection protocol
pub struct ReflectionClient {
    client: ServerReflectionClient<Channel>,
}

impl ReflectionClient {
    /// Create a new reflection client from a gRPC channel
    pub fn new(channel: Channel) -> Self {
        Self {
            client: ServerReflectionClient::new(channel),
        }
    }

    /// List all services exposed by the server
    ///
    /// Returns a list of fully-qualified service names (e.g., "user.UserService")
    pub async fn list_services(&mut self) -> Result<Vec<String>, ReflectionError> {
        debug!("Querying server for available services");

        // Create a request to list all services
        let request = ServerReflectionRequest {
            host: String::new(),
            message_request: Some(MessageRequest::ListServices(String::new())),
        };

        // Send the request
        let response = self.send_request(request).await?;

        // Parse the response
        match response.message_response {
            Some(MessageResponse::ListServicesResponse(list_response)) => {
                let services: Vec<String> = list_response
                    .service
                    .into_iter()
                    .map(|s| s.name)
                    .filter(|name| {
                        // Filter out the reflection service itself
                        !name.starts_with("grpc.reflection")
                    })
                    .collect();

                debug!(count = services.len(), "Found services");
                Ok(services)
            }
            Some(MessageResponse::ErrorResponse(error)) => {
                warn!(
                    error_code = error.error_code,
                    error_message = %error.error_message,
                    "Server returned error response"
                );
                Err(ReflectionError::ProtocolError(format!(
                    "Server error: {}",
                    error.error_message
                )))
            }
            _ => Err(ReflectionError::ProtocolError(
                "Unexpected response type for list_services".to_string(),
            )),
        }
    }

    /// Get methods for a specific service
    ///
    /// Returns a list of method descriptors for all methods in the service
    pub async fn list_methods(
        &mut self,
        service: &str,
    ) -> Result<Vec<MethodDescriptor>, ReflectionError> {
        debug!(service = %service, "Querying service for methods");

        // Request the file descriptor for the service
        let request = ServerReflectionRequest {
            host: String::new(),
            message_request: Some(MessageRequest::FileContainingSymbol(service.to_string())),
        };

        let response = self.send_request(request).await?;

        // Parse the file descriptor
        match response.message_response {
            Some(MessageResponse::FileDescriptorResponse(fd_response)) => {
                self.parse_file_descriptors(fd_response.file_descriptor_proto, service)
            }
            Some(MessageResponse::ErrorResponse(error)) => {
                warn!(
                    service = %service,
                    error_code = error.error_code,
                    error_message = %error.error_message,
                    "Server returned error for service query"
                );
                Err(ReflectionError::ProtocolError(format!(
                    "Server error for service {}: {}",
                    service, error.error_message
                )))
            }
            _ => Err(ReflectionError::ProtocolError(
                "Unexpected response type for file_containing_symbol".to_string(),
            )),
        }
    }

    /// Send a reflection request and get the response
    async fn send_request(
        &mut self,
        request: ServerReflectionRequest,
    ) -> Result<ServerReflectionResponse, ReflectionError> {
        use futures_util::StreamExt;

        // Create a stream with a single request
        let request_stream = futures_util::stream::once(async { request });

        // Send the request and get the response stream
        let mut response_stream = self
            .client
            .server_reflection_info(request_stream)
            .await?
            .into_inner();

        // Get the first (and only) response
        match response_stream.next().await {
            Some(Ok(response)) => Ok(response),
            Some(Err(status)) => {
                // All gRPC errors are handled the same way
                Err(ReflectionError::GrpcError(status))
            }
            None => Err(ReflectionError::ProtocolError(
                "No response received from server".to_string(),
            )),
        }
    }

    /// Parse file descriptor protos to extract method information
    #[allow(clippy::result_large_err)]
    fn parse_file_descriptors(
        &self,
        file_descriptor_protos: Vec<Vec<u8>>,
        service_name: &str,
    ) -> Result<Vec<MethodDescriptor>, ReflectionError> {
        use prost::Message;

        // Early return optimization: once we find the service, we can stop
        for fd_bytes in file_descriptor_protos {
            // Decode the FileDescriptorProto
            let file_descriptor = match prost_types::FileDescriptorProto::decode(&fd_bytes[..]) {
                Ok(fd) => fd,
                Err(e) => {
                    warn!(
                        error = %e,
                        service = %service_name,
                        "Failed to decode file descriptor (malformed descriptor)"
                    );
                    return Err(ReflectionError::MalformedDescriptor(format!(
                        "Failed to decode file descriptor for {}: {}",
                        service_name, e
                    )));
                }
            };

            // Extract package name once per file
            let package = file_descriptor.package.as_deref().unwrap_or("");

            // Find the target service in this file
            if let Some(methods) =
                self.extract_methods_from_service(&file_descriptor.service, package, service_name)?
            {
                // Found the service, return immediately
                debug!(
                    service = %service_name,
                    count = methods.len(),
                    "Extracted methods from service"
                );
                return Ok(methods);
            }
        }

        // Service not found in any file descriptor
        debug!(service = %service_name, "No methods found in service");
        Ok(Vec::new())
    }

    /// Extract methods from a specific service in the file descriptor
    /// Returns Some(methods) if the service is found, None otherwise
    #[allow(clippy::result_large_err)]
    fn extract_methods_from_service(
        &self,
        services: &[prost_types::ServiceDescriptorProto],
        package: &str,
        target_service_name: &str,
    ) -> Result<Option<Vec<MethodDescriptor>>, ReflectionError> {
        for service in services {
            let full_service_name = if package.is_empty() {
                service.name.clone().unwrap_or_default()
            } else {
                format!(
                    "{}.{}",
                    package,
                    service.name.as_ref().unwrap_or(&String::new())
                )
            };

            // Only process the requested service
            if full_service_name != target_service_name {
                continue;
            }

            // Found the target service, extract all methods
            let mut methods = Vec::with_capacity(service.method.len());

            for method in &service.method {
                let method_name = match &method.name {
                    Some(name) => name.clone(),
                    None => {
                        warn!(
                            service = %target_service_name,
                            "Method without name found (malformed descriptor), skipping"
                        );
                        continue;
                    }
                };

                let full_method_name = format!("{}/{}", full_service_name, method_name);

                let input_type = method.input_type.clone().unwrap_or_default();
                let output_type = method.output_type.clone().unwrap_or_default();

                // Validate the method descriptor
                if input_type.is_empty() || output_type.is_empty() {
                    warn!(
                        service = %target_service_name,
                        method = %method_name,
                        "Method has empty input or output type (malformed descriptor), skipping"
                    );
                    continue;
                }

                methods.push(MethodDescriptor {
                    name: method_name,
                    full_name: full_method_name,
                    input_type,
                    output_type,
                });
            }

            // Found the service, return the methods
            return Ok(Some(methods));
        }

        // Service not found in this file
        Ok(None)
    }
}
