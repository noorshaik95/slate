// Include generated proto code
#[allow(clippy::module_inception)]
pub mod auth {
    tonic::include_proto!("auth");
}

pub mod gateway {
    tonic::include_proto!("gateway");
}

// Declare submodules
mod constants;
mod service;
pub mod types;

// Export middleware module
pub mod middleware;

// Tests module
#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{AuthError, AuthResult};

// Re-export public service
pub use service::AuthService;
