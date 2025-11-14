mod types;
mod constants;
pub mod gateway;
pub mod admin;
pub mod user_service;
pub mod error_mapping;
pub mod error;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{GatewayError, ErrorResponse as LegacyErrorResponse, ErrorDetail, map_grpc_error_to_status};
pub use error_mapping::{map_grpc_error_with_context, ErrorType};
pub use error::{ErrorResponse, extract_trace_id};

// Re-export public functions from gateway
pub use gateway::gateway_handler;

// Re-export public functions from admin
pub use admin::{refresh_routes_handler, RefreshResponse};
