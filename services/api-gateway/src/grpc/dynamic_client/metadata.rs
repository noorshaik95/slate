//! Trace header injection for gRPC requests.
//!
//! Handles propagation of trace context headers into gRPC metadata for
//! distributed tracing support.

use std::collections::HashMap;
use tonic::Request;
use tracing::{debug, warn};

/// Inject trace headers into gRPC metadata.
///
/// Converts trace context headers (e.g., traceparent, tracestate) into gRPC
/// metadata format for proper trace propagation across service boundaries.
///
/// # Arguments
///
/// * `request_bytes` - The request payload bytes
/// * `trace_headers` - Map of trace context headers to inject
///
/// # Returns
///
/// A gRPC request with trace headers injected into metadata
pub fn inject_trace_headers(
    request_bytes: Vec<u8>,
    trace_headers: HashMap<String, String>,
) -> Request<Vec<u8>> {
    let mut request = Request::new(request_bytes);
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

    request
}
