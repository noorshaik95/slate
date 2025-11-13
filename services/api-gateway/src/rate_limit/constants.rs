/// Paths that should be excluded from rate limiting
pub const EXCLUDED_PATHS: &[&str] = &["/health", "/metrics"];

/// Cleanup threshold multiplier - clean up clients inactive for this many times the window duration
pub const CLEANUP_THRESHOLD_MULTIPLIER: u32 = 2;
