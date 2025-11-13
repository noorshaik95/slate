use super::*;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;
use crate::config::RateLimitConfig;

fn create_test_config(requests_per_minute: u32, window_seconds: u64) -> RateLimitConfig {
    RateLimitConfig {
        enabled: true,
        requests_per_minute,
        window_seconds,
    }
}

#[tokio::test]
async fn test_rate_limiter_allows_requests_within_limit() {
    let config = create_test_config(5, 60);
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Should allow 5 requests
    for _ in 0..5 {
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());
    }
}

#[tokio::test]
async fn test_rate_limiter_blocks_requests_exceeding_limit() {
    let config = create_test_config(3, 60);
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Allow 3 requests
    for _ in 0..3 {
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());
    }

    // 4th request should be blocked
    let result = limiter.check_rate_limit(client_ip).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RateLimitError::Exceeded(_, _)));
}

#[tokio::test]
async fn test_rate_limiter_tracks_different_clients_separately() {
    let config = create_test_config(2, 60);
    let limiter = RateLimiter::new(config);
    let client1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let client2 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

    // Client 1 makes 2 requests
    assert!(limiter.check_rate_limit(client1).await.is_ok());
    assert!(limiter.check_rate_limit(client1).await.is_ok());

    // Client 1 is now at limit
    assert!(limiter.check_rate_limit(client1).await.is_err());

    // Client 2 should still be able to make requests
    assert!(limiter.check_rate_limit(client2).await.is_ok());
    assert!(limiter.check_rate_limit(client2).await.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_sliding_window() {
    let config = create_test_config(2, 1); // 2 requests per 1 second
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Make 2 requests
    assert!(limiter.check_rate_limit(client_ip).await.is_ok());
    assert!(limiter.check_rate_limit(client_ip).await.is_ok());

    // 3rd request should be blocked
    assert!(limiter.check_rate_limit(client_ip).await.is_err());

    // Wait for window to expire
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be able to make requests again
    assert!(limiter.check_rate_limit(client_ip).await.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_disabled() {
    let config = RateLimitConfig {
        enabled: false,
        requests_per_minute: 1,
        window_seconds: 60,
    };
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Should allow unlimited requests when disabled
    for _ in 0..100 {
        assert!(limiter.check_rate_limit(client_ip).await.is_ok());
    }
}

#[tokio::test]
async fn test_cleanup_expired() {
    let config = create_test_config(5, 1); // 1-second window
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Make some requests
    limiter.check_rate_limit(client_ip).await.ok();
    limiter.check_rate_limit(client_ip).await.ok();

    assert_eq!(limiter.tracked_clients_count().await, 1);

    // Wait for entries to expire
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Run cleanup
    limiter.cleanup_expired().await;

    // Client should be removed
    assert_eq!(limiter.tracked_clients_count().await, 0);
}

#[test]
fn test_should_exclude_path() {
    assert!(RateLimiter::should_exclude_path("/health"));
    assert!(RateLimiter::should_exclude_path("/metrics"));
    assert!(!RateLimiter::should_exclude_path("/api/users"));
    assert!(!RateLimiter::should_exclude_path("/api/health"));
}

#[tokio::test]
async fn test_get_client_request_count() {
    let config = create_test_config(5, 60);
    let limiter = RateLimiter::new(config);
    let client_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    // Initially no requests
    assert_eq!(limiter.get_client_request_count(client_ip).await, None);

    // Make 3 requests
    for _ in 0..3 {
        limiter.check_rate_limit(client_ip).await.ok();
    }

    // Should have 3 requests tracked
    assert_eq!(limiter.get_client_request_count(client_ip).await, Some(3));
}
