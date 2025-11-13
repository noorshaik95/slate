use lazy_static::lazy_static;
use prometheus::{register_counter, register_gauge, register_gauge_vec, register_histogram, Counter, Gauge, GaugeVec, Histogram};

lazy_static! {
    /// Time taken to discover routes (in seconds)
    pub static ref DISCOVERY_DURATION: Histogram = register_histogram!(
        "route_discovery_duration_seconds",
        "Time taken to discover routes from backend services"
    )
    .expect("Failed to register route_discovery_duration_seconds metric");

    /// Total number of discovered routes currently active
    pub static ref DISCOVERED_ROUTES_TOTAL: Gauge = register_gauge!(
        "discovered_routes_total",
        "Total number of discovered routes currently active in the gateway"
    )
    .expect("Failed to register discovered_routes_total metric");

    /// Total number of route discovery errors
    pub static ref DISCOVERY_ERRORS_TOTAL: Counter = register_counter!(
        "route_discovery_errors_total",
        "Total number of route discovery errors encountered"
    )
    .expect("Failed to register route_discovery_errors_total metric");

    /// Methods skipped due to naming convention mismatch
    pub static ref SKIPPED_METHODS_TOTAL: Counter = register_counter!(
        "route_discovery_skipped_methods_total",
        "Total number of methods skipped due to naming convention mismatch"
    )
    .expect("Failed to register route_discovery_skipped_methods_total metric");

    /// Duplicate routes detected and skipped
    pub static ref DUPLICATE_ROUTES_TOTAL: Counter = register_counter!(
        "route_discovery_duplicate_routes_total",
        "Total number of duplicate routes detected and skipped"
    )
    .expect("Failed to register route_discovery_duplicate_routes_total metric");

    /// Status of route discovery per service (1=success, 0=failure)
    pub static ref SERVICE_DISCOVERY_STATUS: GaugeVec = register_gauge_vec!(
        "route_discovery_service_status",
        "Status of route discovery per service (1=success, 0=failure)",
        &["service"]
    )
    .expect("Failed to register route_discovery_service_status metric");
}
