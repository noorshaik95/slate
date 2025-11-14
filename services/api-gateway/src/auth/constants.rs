use std::time::Duration;

/// Default cache TTL for authorization policies (5 minutes)
pub const DEFAULT_CACHE_TTL_SECS: u64 = 300;

/// Default cache TTL as Duration
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(DEFAULT_CACHE_TTL_SECS);

/// Fallback cache TTL when policy query fails (1 minute)
pub const FALLBACK_CACHE_TTL_SECS: u64 = 60;

/// Fallback cache TTL as Duration
pub const FALLBACK_CACHE_TTL: Duration = Duration::from_secs(FALLBACK_CACHE_TTL_SECS);

/// Error message for invalid token
pub const ERR_INVALID_TOKEN: &str = "Token validation failed";

/// Error message for missing claims
pub const ERR_NO_CLAIMS: &str = "No claims in token";

/// Error message prefix for insufficient permissions
pub const ERR_INSUFFICIENT_PERMISSIONS_PREFIX: &str = "User does not have any of the required roles";

/// Error message for invalid auth endpoint
pub const ERR_INVALID_ENDPOINT: &str = "Invalid auth endpoint";

/// Error message for connection failure
pub const ERR_CONNECTION_FAILED: &str = "Failed to connect to auth service";

/// Error message for cache read failure
pub const ERR_CACHE_READ: &str = "Failed to read policy cache";

/// Error message for cache write failure
pub const ERR_CACHE_WRITE: &str = "Failed to write policy cache";

/// Error message for cache clear failure
pub const ERR_CACHE_CLEAR: &str = "Failed to clear policy cache";
