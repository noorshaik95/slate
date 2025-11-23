#[cfg(feature = "observability")]
use serde_json::json;
#[cfg(feature = "observability")]
use time::OffsetDateTime;
#[cfg(feature = "observability")]
use tracing::Subscriber;
#[cfg(feature = "observability")]
use tracing_subscriber::fmt::format::Writer;
#[cfg(feature = "observability")]
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
#[cfg(feature = "observability")]
use tracing_subscriber::registry::LookupSpan;

/// JSON formatter for structured logging
#[cfg(feature = "observability")]
pub struct JsonFormatter;

#[cfg(feature = "observability")]
impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();

        // Get current span context
        let current_span = ctx.lookup_current();
        let mut span_name = None;
        let mut span_fields = serde_json::Map::new();

        if let Some(span) = current_span {
            span_name = Some(span.name());
            let extensions = span.extensions();
            if let Some(fields) = extensions.get::<tracing_subscriber::fmt::FormattedFields<N>>() {
                // Parse fields if possible
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(fields.as_str()) {
                    if let Some(obj) = parsed.as_object() {
                        span_fields = obj.clone();
                    }
                }
            }
        }

        // Build JSON object
        let mut json_obj = json!({
            "timestamp": OffsetDateTime::now_utc().to_string(),
            "level": metadata.level().to_string(),
            "target": metadata.target(),
        });

        if let Some(name) = span_name {
            json_obj["span"] = json!(name);
        }

        if !span_fields.is_empty() {
            json_obj["fields"] = json!(span_fields);
        }

        // Add event message
        let mut visitor = JsonVisitor::new();
        event.record(&mut visitor);
        json_obj["message"] = json!(visitor.message);

        if !visitor.fields.is_empty() {
            json_obj["event_fields"] = json!(visitor.fields);
        }

        writeln!(writer, "{}", json_obj)
    }
}

#[cfg(feature = "observability")]
struct JsonVisitor {
    message: String,
    fields: serde_json::Map<String, serde_json::Value>,
}

#[cfg(feature = "observability")]
impl JsonVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: serde_json::Map::new(),
        }
    }
}

#[cfg(feature = "observability")]
impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .insert(field.name().to_string(), json!(format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields.insert(field.name().to_string(), json!(value));
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
}

/// Mask sensitive information in database URLs
pub fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

/// Mask sensitive information in URLs
pub fn mask_sensitive_url(url: &str) -> String {
    mask_password(url)
}
