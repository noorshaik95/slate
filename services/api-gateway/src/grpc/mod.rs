mod types;
mod constants;
pub mod client;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{GrpcError, GrpcRequest, GrpcResponse};

// Re-export public client
pub use client::GrpcClientPool;
