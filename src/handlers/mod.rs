mod types;
mod constants;
pub mod gateway;
pub mod admin;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{GatewayError, ErrorResponse, ErrorDetail, map_grpc_error_to_status};

// Re-export public functions from gateway
pub use gateway::gateway_handler;

// Re-export public functions from admin
pub use admin::{refresh_routes_handler, RefreshResponse};
