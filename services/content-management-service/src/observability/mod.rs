// Service-specific observability modules
pub mod metrics;
pub mod metrics_server;
pub mod tracing;

// Re-export common-rust observability utilities for convenience
pub use common_rust::observability::*;

// Export service-specific types and functions
pub use metrics::{init_metrics, Metrics};
pub use metrics_server::start_metrics_server;
pub use tracing::{init_tracing, shutdown_tracing};
