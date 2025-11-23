use lru::LruCache;
use std::collections::VecDeque;
use std::hash::Hash;
use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::constants::{CLEANUP_THRESHOLD_MULTIPLIER, DEFAULT_EXCLUDED_PATHS};
use super::types::{ClientRateState, RateLimitConfig, RateLimitError};

/// Rate limiter that tracks requests per client using a sliding window algorithm
/// with LRU cache for bounded memory usage
///
/// Generic over key type K to support different client identifiers (IP, user ID, etc.)
#[derive(Clone)]
pub struct RateLimiter<K: Hash + Eq + Clone> {
    store: Arc<RwLock<LruCache<K, ClientRateState>>>,
    config: RateLimitConfig,
}

impl<K: Hash + Eq + Clone> RateLimiter<K> {
    /// Create a new rate limiter with the given configuration
    /// Uses an LRU cache with a maximum number of entries to prevent unbounded memory growth
    pub fn new(config: RateLimitConfig, max_clients: usize) -> Self {
        let capacity = NonZeroUsize::new(max_clients).expect("max_clients must be non-zero");

        Self {
            store: Arc::new(RwLock::new(LruCache::new(capacity))),
            config,
        }
    }

    /// Check if a request from the given client should be allowed
    /// Returns Ok(()) if allowed, Err(RateLimitError) if rate limit exceeded
    ///
    /// Uses LRU cache: when the cache is full, the least recently used client is automatically evicted
    pub async fn check_rate_limit(&self, client_key: K) -> Result<(), RateLimitError> {
        if !self.config.enabled {
            return Ok(());
        }

        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let max_requests = self.config.requests_per_minute;

        let mut store = self.store.write().await;

        // Get or create client state
        // LRU cache automatically evicts least recently used entries when full
        if !store.contains(&client_key) {
            store.put(
                client_key.clone(),
                ClientRateState {
                    requests: VecDeque::new(),
                    last_cleanup: now,
                },
            );
        }

        let client_state = store
            .get_mut(&client_key)
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

    /// Clean up expired entries from the store
    /// This should be called periodically as a background task
    /// Returns the number of entries evicted
    pub async fn cleanup_expired(&self) -> usize {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds);
        let cleanup_threshold = window_duration * CLEANUP_THRESHOLD_MULTIPLIER;

        let mut store = self.store.write().await;

        // Collect keys to remove (can't modify while iterating)
        let keys_to_remove: Vec<K> = store
            .iter()
            .filter_map(|(key, state)| {
                // Check if this client should be removed
                let has_recent_requests = state
                    .requests
                    .iter()
                    .any(|&req_time| now.duration_since(req_time) <= window_duration);

                let recently_active = now.duration_since(state.last_cleanup) < cleanup_threshold;

                if !has_recent_requests && !recently_active {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove expired entries
        for key in &keys_to_remove {
            store.pop(key);
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
}

/// Check if a path should be excluded from rate limiting
/// Health and metrics endpoints are excluded
pub fn should_exclude_path(path: &str) -> bool {
    DEFAULT_EXCLUDED_PATHS.contains(&path)
}

/// Convenience type alias for IP-based rate limiting
pub type IpRateLimiter = RateLimiter<IpAddr>;
