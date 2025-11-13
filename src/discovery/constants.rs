/// Default refresh interval in seconds (5 minutes)
pub const DEFAULT_REFRESH_INTERVAL_SECONDS: u64 = 300;

/// Minimum allowed refresh interval in seconds (1 minute)
pub const MIN_REFRESH_INTERVAL_SECONDS: u64 = 60;

/// Maximum allowed refresh interval in seconds (1 hour)
pub const MAX_REFRESH_INTERVAL_SECONDS: u64 = 3600;

/// Naming convention patterns for gRPC method names
pub const CONVENTION_PATTERNS: &[&str] = &["Get", "List", "Create", "Update", "Delete"];

/// Default API path prefix
pub const DEFAULT_API_PREFIX: &str = "/api";
