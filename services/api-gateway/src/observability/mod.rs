// Service-specific observability utilities
pub mod json_formatter;

// Re-export from common-rust for convenience
pub use common_rust::observability::extract_trace_id_from_span;

// Export service-specific formatter
pub use json_formatter::FlattenedJsonFormat;
