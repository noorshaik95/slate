use crate::handlers::GatewayError;
use crate::proto::user::user_service_client::UserServiceClient;
use crate::proto::user::*;
use tonic::transport::Channel;
use tracing::{debug, error};

/// Inject trace context into gRPC request metadata
fn inject_trace_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    use opentelemetry::propagation::Injector;
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    
    struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);
    
    impl<'a> Injector for MetadataInjector<'a> {
        fn set(&mut self, key: &str, value: String) {
            if let Ok(metadata_key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
                if let Ok(metadata_value) = tonic::metadata::MetadataValue::try_from(&value) {
                    self.0.insert(metadata_key, metadata_value);
                    debug!(key = %key, "Injected trace header into gRPC metadata");
                }
            }
        }
    }
    
    // CRITICAL: Extract the OpenTelemetry context from the current tracing span
    // This ensures we propagate the correct trace context across service boundaries
    let current_span = tracing::Span::current();
    let context = current_span.context();
    
    // Inject the context into gRPC metadata using the global propagator
    let mut injector = MetadataInjector(request.metadata_mut());
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut injector);
    });
    
    request
}

/// Handle user service gRPC calls with proper protobuf types
#[tracing::instrument(skip(channel, json_payload), fields(grpc.service = "user.UserService", grpc.method = %method))]
pub async fn call_user_service(
    channel: Channel,
    method: &str,
    json_payload: Vec<u8>,
) -> Result<Vec<u8>, GatewayError> {
    debug!(
        method = %method,
        payload_size = json_payload.len(),
        "Calling user service with typed client"
    );

    // Create client - we'll inject trace context manually for each request
    let mut client = UserServiceClient::new(channel);

    // Parse JSON manually for each request type
    let json_value: serde_json::Value = serde_json::from_slice(&json_payload)
        .map_err(|e| GatewayError::ConversionError(format!("Failed to parse JSON: {}", e)))?;

    match method {
        "Register" => {
            let request = RegisterRequest {
                email: json_value["email"].as_str().unwrap_or("").to_string(),
                password: json_value["password"].as_str().unwrap_or("").to_string(),
                first_name: json_value["first_name"].as_str().unwrap_or("").to_string(),
                last_name: json_value["last_name"].as_str().unwrap_or("").to_string(),
                phone: json_value["phone"].as_str().unwrap_or("").to_string(),
            };

            debug!(email = %request.email, "Registering user");
            
            let mut grpc_request = tonic::Request::new(request);
            grpc_request = inject_trace_context(grpc_request);
            
            let response = client
                .register(grpc_request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Register call failed");
                    GatewayError::GrpcCallFailed(format!("Register failed: {}", e))
                })?;

            let resp = response.into_inner();
            let json_response = serde_json::json!({
                "access_token": resp.access_token,
                "refresh_token": resp.refresh_token,
                "user": resp.user.map(|u| serde_json::json!({
                    "id": u.id,
                    "email": u.email,
                    "first_name": u.first_name,
                    "last_name": u.last_name,
                    "phone": u.phone,
                    "roles": u.roles,
                    "is_active": u.is_active,
                }))
            });

            Ok(serde_json::to_vec(&json_response)
                .map_err(|e| GatewayError::ConversionError(format!("Failed to serialize response: {}", e)))?)
        }
        "Login" => {
            let request = LoginRequest {
                email: json_value["email"].as_str().unwrap_or("").to_string(),
                password: json_value["password"].as_str().unwrap_or("").to_string(),
            };

            debug!(email = %request.email, "Logging in user");
            
            let mut grpc_request = tonic::Request::new(request);
            grpc_request = inject_trace_context(grpc_request);
            
            let response = client
                .login(grpc_request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Login call failed");
                    GatewayError::GrpcCallFailed(format!("Login failed: {}", e))
                })?;

            let resp = response.into_inner();
            let json_response = serde_json::json!({
                "access_token": resp.access_token,
                "refresh_token": resp.refresh_token,
                "expires_in": resp.expires_in,
                "user": resp.user.map(|u| serde_json::json!({
                    "id": u.id,
                    "email": u.email,
                    "first_name": u.first_name,
                    "last_name": u.last_name,
                    "phone": u.phone,
                    "roles": u.roles,
                    "is_active": u.is_active,
                }))
            });

            Ok(serde_json::to_vec(&json_response)
                .map_err(|e| GatewayError::ConversionError(format!("Failed to serialize response: {}", e)))?)
        }
        _ => {
            error!(method = %method, "Unsupported user service method");
            Err(GatewayError::ConversionError(format!("Unsupported method: {}", method)))
        }
    }
}
