pub mod admin;
mod constants;
pub mod gateway;
mod types;
// Deprecated: user_service module is no longer used
// All services now use the dynamic client in gateway.rs
// Typed handler for UserService with support for all CRUD and auth methods
pub mod error;
pub mod error_mapping;
pub mod user_service;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::GatewayError;

// Re-export public functions from gateway

// Re-export public functions from admin
pub use admin::refresh_routes_handler;
