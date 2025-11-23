/// Default paths to exclude from rate limiting
pub const DEFAULT_EXCLUDED_PATHS: &[&str] = &[
    "/health",
    "/health/liveness",
    "/health/readiness",
    "/metrics",
    "/api/health",
];

/// Multiplier for cleanup threshold (e.g., 2x window duration)
pub const CLEANUP_THRESHOLD_MULTIPLIER: u32 = 2;

/// Maximum number of clients to track in the LRU cache
pub const MAX_TRACKED_CLIENTS: usize = 10_000;
