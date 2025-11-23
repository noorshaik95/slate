#[cfg(test)]
mod tests {
    use super::super::{should_exclude_path, RateLimitConfig, RateLimitError, RateLimiter};
    use std::net::IpAddr;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 5,
            window_seconds: 60,
        };

        let limiter = RateLimiter::new(config, 100);
        let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Should allow first 5 requests
        for _ in 0..5 {
            let result = limiter.check_rate_limit(client_ip).await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 3,
            window_seconds: 60,
        };

        let limiter = RateLimiter::new(config, 100);
        let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Allow first 3 requests
        for _ in 0..3 {
            let _ = limiter.check_rate_limit(client_ip).await;
        }

        // 4th request should be blocked
        let result = limiter.check_rate_limit(client_ip).await;
        assert!(matches!(result, Err(RateLimitError::Exceeded(3, 60))));
    }

    #[tokio::test]
    async fn test_sliding_window() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 2,
            window_seconds: 1,
        };

        let limiter = RateLimiter::new(config, 100);
        let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Make 2 requests
        let _ = limiter.check_rate_limit(client_ip).await;
        let _ = limiter.check_rate_limit(client_ip).await;

        // 3rd request should be blocked
        let result = limiter.check_rate_limit(client_ip).await;
        assert!(result.is_err());

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should allow new request after window expires
        let result = limiter.check_rate_limit(client_ip).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_clients() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 2,
            window_seconds: 60,
        };

        let limiter = RateLimiter::new(config, 100);
        let client1: IpAddr = "127.0.0.1".parse().unwrap();
        let client2: IpAddr = "127.0.0.2".parse().unwrap();

        // Each client should have independent limits
        for _ in 0..2 {
            assert!(limiter.check_rate_limit(client1).await.is_ok());
            assert!(limiter.check_rate_limit(client2).await.is_ok());
        }

        // Both should be blocked on 3rd request
        assert!(limiter.check_rate_limit(client1).await.is_err());
        assert!(limiter.check_rate_limit(client2).await.is_err());
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 10,
            window_seconds: 60,
        };

        // Small cache size to test eviction
        let limiter = RateLimiter::new(config, 3);

        // Add 4 clients (should evict the first one)
        for i in 1..=4 {
            let ip: IpAddr = format!("127.0.0.{}", i).parse().unwrap();
            let _ = limiter.check_rate_limit(ip).await;
        }

        // Should have 3 clients tracked
        assert_eq!(limiter.tracked_clients_count().await, 3);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 10,
            window_seconds: 1,
        };

        let limiter = RateLimiter::new(config, 100);
        let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Make a request
        let _ = limiter.check_rate_limit(client_ip).await;
        assert_eq!(limiter.tracked_clients_count().await, 1);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(2500)).await;

        // Cleanup should remove expired entries
        let evicted = limiter.cleanup_expired().await;
        assert_eq!(evicted, 1);
        assert_eq!(limiter.tracked_clients_count().await, 0);
    }

    #[tokio::test]
    async fn test_disabled_rate_limiter() {
        let config = RateLimitConfig {
            enabled: false,
            requests_per_minute: 1,
            window_seconds: 60,
        };

        let limiter = RateLimiter::new(config, 100);
        let client_ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Should allow unlimited requests when disabled
        for _ in 0..100 {
            let result = limiter.check_rate_limit(client_ip).await;
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_excluded_paths() {
        assert!(should_exclude_path("/health"));
        assert!(should_exclude_path("/metrics"));
        assert!(should_exclude_path("/health/liveness"));
        assert!(!should_exclude_path("/api/users"));
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let config = RateLimitConfig {
            enabled: true,
            requests_per_minute: 100,
            window_seconds: 60,
        };

        let limiter = RateLimiter::new(config, 1000);

        // Spawn multiple concurrent tasks
        let mut handles = vec![];
        for i in 0..50 {
            let limiter_clone = limiter.clone();
            let handle = tokio::spawn(async move {
                let ip: IpAddr = format!("127.0.0.{}", i % 10).parse().unwrap();
                limiter_clone.check_rate_limit(ip).await
            });
            handles.push(handle);
        }

        // Wait for all tasks
        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(())) = handle.await {
                success_count += 1;
            }
        }

        // Most requests should succeed (some may fail due to rate limits)
        assert!(success_count > 0);
    }
}
