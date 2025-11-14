// Include generated proto code
pub mod auth {
    tonic::include_proto!("auth");
}

pub mod gateway {
    tonic::include_proto!("gateway");
}

// Declare submodules
mod types;
mod constants;
mod service;

// Export middleware module
pub mod middleware;

// Tests module
#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{
    AuthError,
    AuthResult,
};

// Re-export public service
pub use service::AuthService;
