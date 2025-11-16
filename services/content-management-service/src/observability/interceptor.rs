use tonic::Request;
use tracing::{info, warn};
use opentelemetry::trace::{TraceContextExt, TraceId, SpanId};
use opentelemetry::Context;

/// Extract trace context from gRPC metadata and inject into current span
///
/// This interceptor extracts W3C traceparent headers from incoming gRPC requests
/// and propagates them to the OpenTelemetry context, ensuring distributed tracing
/// works correctly across the API gateway -> content-management-service boundary.
pub fn extract_trace_context<T>(request: &Request<T>) -> Option<Context> {
    let metadata = request.metadata();
    
    // Try to extract traceparent header (W3C Trace Context format)
    if let Some(traceparent) = metadata.get("traceparent") {
        if let Ok(traceparent_str) = traceparent.to_str() {
            info!(
                traceparent = %traceparent_str,
                "Extracted traceparent from gRPC metadata"
            );
            
            // Parse W3C traceparent format: version-trace_id-span_id-flags
            // Example: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
            let parts: Vec<&str> = traceparent_str.split('-').collect();
            
            if parts.len() == 4 {
                let trace_id_hex = parts[1];
                let span_id_hex = parts[2];
                
                info!(
                    trace_id = %trace_id_hex,
                    span_id = %span_id_hex,
                    "Parsed trace context from traceparent"
                );
                
                // Parse trace ID and span ID
                if let (Ok(trace_id_bytes), Ok(span_id_bytes)) = (
                    hex::decode(trace_id_hex),
                    hex::decode(span_id_hex),
                ) {
                    if trace_id_bytes.len() == 16 && span_id_bytes.len() == 8 {
                        let trace_id = TraceId::from_bytes(
                            trace_id_bytes.try_into().unwrap()
                        );
                        let span_id = SpanId::from_bytes(
                            span_id_bytes.try_into().unwrap()
                        );
                        
                        // Create a span context with the extracted IDs
                        let span_context = opentelemetry::trace::SpanContext::new(
                            trace_id,
                            span_id,
                            opentelemetry::trace::TraceFlags::SAMPLED,
                            true, // is_remote
                            opentelemetry::trace::TraceState::default(),
                        );
                        
                        // Create a context with this span
                        let context = Context::current()
                            .with_remote_span_context(span_context);
                        
                        info!(
                            trace_id = %trace_id,
                            span_id = %span_id,
                            "Successfully created trace context from gRPC metadata"
                        );
                        
                        return Some(context);
                    }
                }
            }
            
            warn!(
                traceparent = %traceparent_str,
                "Failed to parse traceparent header"
            );
        }
    } else {
        warn!("No traceparent header found in gRPC metadata");
    }
    
    None
}

/// Log all metadata keys for debugging trace propagation issues
pub fn log_metadata_keys<T>(request: &Request<T>) {
    let metadata = request.metadata();
    info!(
        metadata_count = metadata.len(),
        "Incoming gRPC request metadata"
    );
    
    // Iterate over metadata entries
    use tonic::metadata::KeyAndValueRef;
    for key_value in metadata.iter() {
        match key_value {
            KeyAndValueRef::Ascii(key, value) => {
                if let Ok(value_str) = value.to_str() {
                    info!(
                        key = %key,
                        value = %value_str,
                        "Metadata entry (ASCII)"
                    );
                }
            }
            KeyAndValueRef::Binary(key, _value) => {
                info!(
                    key = %key,
                    "Metadata entry (Binary)"
                );
            }
        }
    }
}

/// Extract trace ID from current span context for logging
pub fn get_trace_id_from_context() -> String {
    let context = Context::current();
    let span = context.span();
    let span_context = span.span_context();
    
    if span_context.is_valid() {
        span_context.trace_id().to_string()
    } else {
        "no-trace-id".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::{MetadataMap, MetadataValue};
    
    #[test]
    fn test_extract_valid_traceparent() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "traceparent",
            MetadataValue::try_from("00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01").unwrap(),
        );
        
        let mut request = Request::new(());
        *request.metadata_mut() = metadata;
        
        let context = extract_trace_context(&request);
        assert!(context.is_some());
    }
    
    #[test]
    fn test_extract_missing_traceparent() {
        let request = Request::new(());
        let context = extract_trace_context(&request);
        assert!(context.is_none());
    }
    
    #[test]
    fn test_extract_invalid_traceparent() {
        let mut metadata = MetadataMap::new();
        metadata.insert(
            "traceparent",
            MetadataValue::try_from("invalid-format").unwrap(),
        );
        
        let mut request = Request::new(());
        *request.metadata_mut() = metadata;
        
        let context = extract_trace_context(&request);
        assert!(context.is_none());
    }
}
