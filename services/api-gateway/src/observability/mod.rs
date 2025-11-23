// Service-specific observability utilities
pub mod json_formatter;

// Re-export from common-rust for convenience

// Export service-specific formatter
pub use json_formatter::FlattenedJsonFormat;
