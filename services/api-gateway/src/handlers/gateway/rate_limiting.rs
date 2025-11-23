//! Rate limiting enforcement.
//!
//! Applies rate limiting to incoming requests based on client IP.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tracing::warn;

use crate::handlers::types::GatewayError;
use crate::shared::state::AppState;
use common_rust::observability::extract_trace_id_from_span;

/// Apply rate limiting to a request.
///
/// Checks if the client IP has exceeded the rate limit and returns an error if so.
///
/// # Arguments
///
/// * `state` - Application state containing the rate limiter
/// * `client_ip` - The client's IP address
/// * `path` - The request path (for logging)
/// * `method` - The HTTP method (for logging)
/// * `remote_addr` - The remote socket address (for logging)
/// * `start_time` - Request start time (for duration logging)
///
/// # Returns
///
/// Ok(()) if rate limit not exceeded, Err otherwise
pub async fn apply_rate_limit(
    state: &Arc<AppState>,
    client_ip: IpAddr,
    path: &str,
    method: &str,
    remote_addr: IpAddr,
    start_time: Instant,
) -> Result<(), GatewayError> {
    if let Some(rate_limiter) = &state.rate_limiter {
        if !common_rust::rate_limit::should_exclude_path(path) {
            if let Err(e) = rate_limiter.check_rate_limit(client_ip).await {
                let duration_ms = start_time.elapsed().as_millis();
                let trace_id = extract_trace_id_from_span();

                warn!(
                    client_ip = %client_ip,
                    remote_addr = %remote_addr,
                    path = %path,
                    method = %method,
                    duration_ms = %duration_ms,
                    trace_id = %trace_id,
                    error_type = "rate_limit",
                    error = %e,
                    "Rate limit exceeded"
                );

                state.metrics.rate_limit_counter.inc();

                return Err(GatewayError::RateLimitExceeded);
            }
        }
    }

    Ok(())
}
