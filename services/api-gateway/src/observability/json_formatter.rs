use opentelemetry::trace::TraceContextExt;
use serde_json::{json, Map, Value};
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

/// Custom JSON event formatter that places OpenTelemetry trace fields at root level
///
/// By default, tracing_subscriber's JSON formatter places span fields inside a nested
/// "fields" object. This formatter flattens trace_id, span_id, and trace_flags to the
/// root level for consistency with other services (Go/zerolog, NestJS/pino) and easier
/// querying in Loki.
///
/// Example output:
/// ```json
/// {
///   "timestamp": "2025-11-21T20:56:52.123Z",
///   "level": "info",
///   "message": "Request processed",
///   "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
///   "span_id": "00f067aa0ba902b7",
///   "trace_flags": "01",
///   "target": "api_gateway::handlers",
///   "fields": {
///     "http.method": "GET",
///     "http.target": "/api/users"
///   }
/// }
/// ```
pub struct FlattenedJsonFormat;

impl FlattenedJsonFormat {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FlattenedJsonFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, N> FormatEvent<S, N> for FlattenedJsonFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut json_map = Map::new();

        // Add timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        json_map.insert(
            "timestamp".to_string(),
            json!(format!("{}.{:03}", now.as_secs(), now.subsec_millis())),
        );

        // Add level
        let level = event.metadata().level();
        json_map.insert("level".to_string(), json!(level.to_string().to_lowercase()));

        // Add target
        json_map.insert("target".to_string(), json!(event.metadata().target()));

        // Extract trace context from OpenTelemetry
        let otel_context = opentelemetry::Context::current();
        let otel_span = otel_context.span();
        let span_context = otel_span.span_context();

        if span_context.is_valid() {
            json_map.insert(
                "trace_id".to_string(),
                json!(span_context.trace_id().to_string()),
            );
            json_map.insert(
                "span_id".to_string(),
                json!(span_context.span_id().to_string()),
            );
            json_map.insert(
                "trace_flags".to_string(),
                json!(format!("{:02x}", span_context.trace_flags().to_u8())),
            );
        }

        // Collect event fields
        let mut field_visitor = JsonVisitor::new();
        event.record(&mut field_visitor);

        // Add message if present
        if let Some(message) = field_visitor.message {
            json_map.insert("message".to_string(), json!(message));
        }

        // Add other fields
        if !field_visitor.fields.is_empty() {
            json_map.insert("fields".to_string(), json!(field_visitor.fields));
        }

        // Write JSON
        let json_str = serde_json::to_string(&json_map).map_err(|_| fmt::Error)?;
        writeln!(writer, "{}", json_str).map_err(|_| fmt::Error)?;

        Ok(())
    }
}

/// Visitor to collect event fields into a JSON map
struct JsonVisitor {
    message: Option<String>,
    fields: Map<String, Value>,
}

impl JsonVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: Map::new(),
        }
    }
}

impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        let field_name = field.name();

        if field_name == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            self.fields
                .insert(field_name.to_string(), json!(format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let field_name = field.name();

        if field_name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.insert(field_name.to_string(), json!(value));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flattened_json_format_creation() {
        let _formatter = FlattenedJsonFormat::new();
        // Just verify it can be created without panicking
    }

    #[test]
    fn test_flattened_json_format_default() {
        let _formatter = FlattenedJsonFormat;
        // Just verify default works without panicking
    }

    #[test]
    fn test_json_visitor_message() {
        let visitor = JsonVisitor::new();

        // Verify initial state
        assert!(visitor.message.is_none());
        assert_eq!(visitor.fields.len(), 0);
    }
}
