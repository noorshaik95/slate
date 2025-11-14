use lru::LruCache;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use crate::config::RateLimitConfig;

use super::types::{ClientRateState, RateLimitError};
use super::constants::{EXCLUDED_PATHS, CLEANUP_THRESHOLD_MULTIPLIER};

/// Maximum number of clients to track in the LRU cache
const MAX_TRACKED_CLIENTS: usize = 10_000;

/// Rate limiter that tracks requests per client IP using a sliding window algorithm
/// with LRU cache for bounded memory usage
#[derive(Clone)]
pub struct RateLimiter {
    store: Arc<RwLock<LruCache<IpAddr, ClientRateState>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    /// Uses an LRU cache with a maximum of 10,000 entries to prevent unbounded memory growth
    pub fn new(config: RateLimitConfig) -> Self {
        let capacity = NonZeroUsize::new(MAX_TRACKED_CLIENTS)
            .expect("MAX_TRACKED_CLIENTS must be non-zero");
        
        Self {
            store: Arc::new(RwLock::new(LruCache::new(capacity))),
            config,
        }
    }

    /// Check if a request from the given client IP should be allowed
    /// Returns Ok(()) if allowed, Err(RateLimitError) if rate limit exceeded
    /// 
    /// Uses LRU cache: when the cache is full, the least recently used client is automatically evicted
    pub async fn check_rate_limit(&self, client_ip: IpAddr) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let max_requests = self.config.requests_per_minute;

        let mut store = self.store.write().await;
        
        // Get or create client state
        // LRU cache automatically evicts least recently used entries when full
        if !store.contains(&client_ip) {
            store.put(client_ip, ClientRateState {
                requests: std::collections::VecDeque::new(),
                last_cleanup: now,
            });
        }
        
        let client_state = store.get_mut(&client_ip)
            .expect("Client state should exist after put");

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
    /// Returns the number of entries evicted
    pub async fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let cleanup_threshold = window_duration * CLEANUP_THRESHOLD_MULTIPLIER;

        let mut store = self.store.write().await;
        
        // Collect keys to remove (can't modify while iterating)
        let keys_to_remove: Vec<IpAddr> = store
            .iter()
            .filter_map(|(ip, state)| {
                // Check if this client should be removed
                let has_recent_requests = state.requests.iter().any(|&req_time| {
                    now.duration_since(req_time) <= window_duration
                });
                
                let recently_active = now.duration_since(state.last_cleanup) < cleanup_threshold;
                
                if !has_recent_requests && !recently_active {
                    Some(*ip)
                } else {
                    None
                }
            })
            .collect();
        
        // Remove expired entries
        for ip in &keys_to_remove {
            store.pop(ip);
        }
        
        let evicted_count = keys_to_remove.len();
        
        // Also clean up old requests from remaining clients
        for (_, state) in store.iter_mut() {
            while let Some(&oldest) = state.requests.front() {
                if now.duration_since(oldest) > window_duration {
                    state.requests.pop_front();
                } else {
                    break;
                }
            }
        }
        
        evicted_count
    }

    /// Get the current number of tracked clients (for monitoring/debugging)
    pub async fn tracked_clients_count(&self) -> usize {
        self.store.read().await.len()
    }

    /// Get the current request count for a specific client (for monitoring/debugging)
    #[allow(dead_code)]
    pub async fn get_client_request_count(&self, client_ip: IpAddr) -> Option<usize> {
        let mut store = self.store.write().await;
        store.get(&client_ip).map(|state| state.requests.len())
    }
}
