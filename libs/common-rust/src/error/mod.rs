//! Error response module for standardized error handling
//!
//! Provides error response types and utilities for HTTP and gRPC protocols.

mod trace;
mod types;

pub use trace::generate_trace_id;
pub use types::{ErrorDetail, ErrorResponse};

#[cfg(feature = "http")]
pub use trace::extract_trace_id_from_http;

#[cfg(feature = "grpc")]
pub use trace::extract_trace_id_from_grpc;
