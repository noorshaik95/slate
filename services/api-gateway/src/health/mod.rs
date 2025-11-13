// Declare submodules
mod types;
mod checker;
mod handler;

#[cfg(test)]
mod tests;

// Re-export public types
pub use types::{HealthState, HealthStatus, ServiceHealth};

// Re-export public components
pub use checker::HealthChecker;
pub use handler::health_handler;
