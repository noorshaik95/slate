mod types;
mod constants;
pub mod client;
pub mod dynamic_client;
pub mod pool;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{GrpcError, GrpcRequest, GrpcResponse};

// Re-export public client
pub use client::GrpcClientPool;
pub use dynamic_client::DynamicGrpcClient;
pub use pool::{ConnectionPool, PoolStats};
