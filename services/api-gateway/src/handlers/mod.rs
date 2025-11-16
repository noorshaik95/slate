mod types;
mod constants;
pub mod gateway;
pub mod admin;
// Deprecated: user_service module is no longer used
// All services now use the dynamic client in gateway.rs
// Typed handler for UserService with support for all CRUD and auth methods
pub mod user_service;
pub mod error_mapping;
pub mod error;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::GatewayError;

// Re-export public functions from gateway

// Re-export public functions from admin
pub use admin::refresh_routes_handler;
