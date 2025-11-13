use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::config::RateLimitConfig;

use super::types::{ClientRateState, RateLimitError};
use super::constants::{EXCLUDED_PATHS, CLEANUP_THRESHOLD_MULTIPLIER};

/// Rate limiter that tracks requests per client IP using a sliding window algorithm
pub struct RateLimiter {
    store: Arc<RwLock<HashMap<IpAddr, ClientRateState>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request from the given client IP should be allowed
    /// Returns Ok(()) if allowed, Err(RateLimitError) if rate limit exceeded
    pub async fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let max_requests = self.config.requests_per_minute;

        let mut store = self.store.write().await;
        
        // Get or create client state
        let client_state = store.entry(client_ip).or_insert_with(|| ClientRateState {
            requests: std::collections::VecDeque::new(),
            last_cleanup: now,
        });

        // Remove requests outside the sliding window
        while let Some(&oldest) = client_state.requests.front() {
            if now.duration_since(oldest) > window_duration {
                client_state.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if adding this request would exceed the limit
        if client_state.requests.len() >= max_requests as usize {
            return Err(RateLimitError::Exceeded(
                max_requests,
                self.config.window_seconds,
            ));
        }

        // Add the current request timestamp
        client_state.requests.push_back(now);
        client_state.last_cleanup = now;

        Ok(())
    }

    /// Check if a path should be excluded from rate limiting
    /// Health and metrics endpoints are excluded
    pub fn should_exclude_path(path: &str) -> bool {
        EXCLUDED_PATHS.contains(&path)
    }

    /// Clean up expired entries from the store
    /// This should be called periodically as a background task
    pub async fn cleanup_expired(&self) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let cleanup_threshold = window_duration * CLEANUP_THRESHOLD_MULTIPLIER;

        let mut store = self.store.write().await;
        
        // Remove clients that haven't made requests recently
        store.retain(|_, state| {
            // First, clean up old requests from this client's queue
            while let Some(&oldest) = state.requests.front() {
                if now.duration_since(oldest) > window_duration {
                    state.requests.pop_front();
                } else {
                    break;
                }
            }

            // Keep the client if they have recent requests or were recently active
            !state.requests.is_empty() || now.duration_since(state.last_cleanup) < cleanup_threshold
        });
    }

    /// Get the current number of tracked clients (for monitoring/debugging)
    pub async fn tracked_clients_count(&self) -> usize {
        self.store.read().await.len()
    }

    /// Get the current request count for a specific client (for monitoring/debugging)
    pub async fn get_client_request_count(&self, client_ip: IpAddr) -> Option<usize> {
        let store = self.store.read().await;
        store.get(&client_ip).map(|state| state.requests.len())
    }
}
