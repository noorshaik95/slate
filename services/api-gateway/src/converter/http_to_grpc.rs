use axum::body::Body;
use axum::http::{HeaderMap, Request};
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::debug;

use crate::grpc::GrpcRequest;
use crate::router::RoutingDecision;

use super::constants::PROPAGATE_HEADERS;
use super::types::ConversionError;

/// Converter for HTTP to gRPC transformations
pub struct HttpToGrpcConverter;

impl HttpToGrpcConverter {
    /// Convert an HTTP request to gRPC format
    /// 
    /// This function:
    /// - Extracts the request body and parses it as JSON
    /// - Merges path parameters into the payload
    /// - Extracts relevant headers and converts them to gRPC metadata
    /// - Prepares the request for gRPC transmission
    pub async fn convert_request(
        http_req: Request<Body>,
        routing: &RoutingDecision,
    ) -> Result<GrpcRequest, ConversionError> {
        debug!(
            service = %routing.service,
            method = %routing.grpc_method,
            "Converting HTTP request to gRPC"
        );

        // Extract headers before consuming the request
        let headers = http_req.headers().clone();

        // Read the request body
        let body_bytes = axum::body::to_bytes(http_req.into_body(), usize::MAX)
            .await
            .map_err(|e| ConversionError::BodyReadError(e.to_string()))?;

        // Parse body as JSON (or create empty object if body is empty)
        let mut payload: Value = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body_bytes)
                .map_err(|e| ConversionError::JsonParseError(e.to_string()))?
        };

        // Merge path parameters into the payload
        if !routing.path_params.is_empty() {
            debug!(params = ?routing.path_params, "Merging path parameters into payload");
            
            if let Some(obj) = payload.as_object_mut() {
                for (key, value) in &routing.path_params {
                    obj.insert(key.clone(), Value::String(value.clone()));
                }
            } else {
                // If payload is not an object, create a new object with path params
                let mut new_payload = serde_json::Map::new();
                for (key, value) in &routing.path_params {
                    new_payload.insert(key.clone(), Value::String(value.clone()));
                }
                payload = Value::Object(new_payload);
            }
        }

        // Convert payload to bytes
        let payload_bytes = serde_json::to_vec(&payload)
            .map_err(|e| ConversionError::JsonSerializeError(e.to_string()))?;

        // Extract metadata from headers for trace propagation
        let metadata = Self::extract_metadata(&headers);

        debug!(
            payload_size = payload_bytes.len(),
            metadata_count = metadata.len(),
            "HTTP to gRPC conversion complete"
        );

        Ok(GrpcRequest {
            service: routing.service.to_string(),
            method: routing.grpc_method.to_string(),
            payload: payload_bytes,
            metadata,
        })
    }

    /// Extract relevant headers and convert them to gRPC metadata
    /// 
    /// This includes:
    /// - Trace propagation headers (trace-id, span-id, etc.)
    /// - Authorization context (if needed for backend)
    /// - Custom headers that should be propagated
    fn extract_metadata(headers: &HeaderMap) -> HashMap<String, String> {
        let mut metadata = HashMap::new();

        for header_name in PROPAGATE_HEADERS {
            if let Some(value) = headers.get(*header_name) {
                if let Ok(value_str) = value.to_str() {
                    metadata.insert(header_name.to_string(), value_str.to_string());
                    debug!(header = %header_name, value = %value_str, "Propagating header to gRPC metadata");
                }
            }
        }

        // Generate trace ID if not present
        if !metadata.contains_key("x-trace-id") && !metadata.contains_key("traceparent") {
            let trace_id = uuid::Uuid::new_v4().to_string();
            metadata.insert("x-trace-id".to_string(), trace_id.clone());
            debug!(trace_id = %trace_id, "Generated new trace ID");
        }

        metadata
    }
}
