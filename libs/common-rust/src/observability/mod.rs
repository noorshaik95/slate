//! Observability module for tracing and logging
//!
//! Provides utilities for distributed tracing, logging, and observability.

mod interceptor;
mod json_formatter;
mod logging;
mod tracing_utils;

pub use tracing_utils::{parse_traceparent, TraceContext};

#[cfg(feature = "observability")]
pub use tracing_utils::extract_trace_id_from_span;

#[cfg(feature = "http")]
pub use tracing_utils::extract_trace_context_from_headers;

#[cfg(all(feature = "grpc", feature = "observability"))]
pub use interceptor::{extract_trace_context_from_grpc, get_trace_id_from_context};

#[cfg(feature = "grpc")]
pub use interceptor::log_grpc_metadata;

#[cfg(feature = "observability")]
pub use logging::{init_tracing, shutdown_tracing, TracingConfig};

#[cfg(feature = "observability")]
pub use json_formatter::{mask_password, mask_sensitive_url, JsonFormatter};
