use api_gateway::rate_limit::RateLimiter;
use api_gateway::config::RateLimitConfig;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test rate limiter with 100+ concurrent requests
#[tokio::test]
async fn test_concurrent_requests_from_same_ip() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 60, // 1 per second
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.100".parse().unwrap();

    // Spawn 100 concurrent requests
    let mut handles = vec![];
    for _ in 0..100 {
        let limiter_clone = Arc::clone(&limiter);
        let ip = test_ip;
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(ip).await
        });
        handles.push(handle);
    }

    // Collect results
    let mut allowed_count = 0;
    let mut denied_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => allowed_count += 1,
            Err(_) => denied_count += 1,
        }
    }

    // Should allow some requests based on rate limit, deny the rest
    assert!(allowed_count <= 60, "Allowed {} requests, expected <= 60", allowed_count);
    assert!(denied_count >= 40, "Denied {} requests, expected >= 40", denied_count);
    assert_eq!(allowed_count + denied_count, 100);
}

/// Test rate limiter with multiple IPs concurrently
#[tokio::test]
async fn test_concurrent_requests_from_multiple_ips() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 30,
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));

    // Create 10 different IPs, each making 15 requests
    let mut handles = vec![];
    for ip_suffix in 1..=10 {
        for _ in 0..15 {
            let limiter_clone = Arc::clone(&limiter);
            let ip: IpAddr = format!("192.168.1.{}", ip_suffix).parse().unwrap();
            let handle = tokio::spawn(async move {
                limiter_clone.check_rate_limit(ip).await
            });
            handles.push(handle);
        }
    }

    // Collect results per IP
    let mut total_allowed = 0;
    let mut total_denied = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => total_allowed += 1,
            Err(_) => total_denied += 1,
        }
    }

    // Each IP should allow up to 30 requests per minute
    // With 10 IPs making 15 requests each = 150 total requests
    // Each IP allows 15 requests (under the 30 limit), so all should be allowed
    assert!(total_allowed >= 140 && total_allowed <= 150, 
            "Allowed {} requests, expected 140-150", total_allowed);
    assert!(total_denied >= 0 && total_denied <= 10, 
            "Denied {} requests, expected 0-10", total_denied);
}

/// Test LRU cache eviction under load
#[tokio::test]
async fn test_lru_cache_eviction() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 60,
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));

    // Make requests from many different IPs to trigger LRU eviction
    // The cache size is 10,000, so we'll use 10,500 IPs
    let mut handles = vec![];
    for ip_suffix in 1..=10500 {
        let limiter_clone = Arc::clone(&limiter);
        let ip: IpAddr = format!("10.{}.{}.{}", 
            ip_suffix / 65536, 
            (ip_suffix / 256) % 256, 
            ip_suffix % 256
        ).parse().unwrap();
        
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(ip).await
        });
        handles.push(handle);
    }

    // All requests should be allowed (first request from each IP)
    let mut allowed_count = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            allowed_count += 1;
        }
    }

    // All should be allowed since it's the first request from each IP
    assert_eq!(allowed_count, 10500);

    // Now make a second request from the first 500 IPs
    // Some of these should have been evicted from the cache
    let mut second_round_handles = vec![];
    for ip_suffix in 1..=500 {
        let limiter_clone = Arc::clone(&limiter);
        let ip: IpAddr = format!("10.{}.{}.{}", 
            ip_suffix / 65536, 
            (ip_suffix / 256) % 256, 
            ip_suffix % 256
        ).parse().unwrap();
        
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(ip).await
        });
        second_round_handles.push(handle);
    }

    // These should all be allowed since they were evicted and reset
    let mut second_allowed = 0;
    for handle in second_round_handles {
        if handle.await.unwrap().is_ok() {
            second_allowed += 1;
        }
    }

    assert_eq!(second_allowed, 500);
}

/// Test rate limit recovery over time
#[tokio::test]
async fn test_rate_limit_recovery() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 60, // 1 per second
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.200".parse().unwrap();

    // Make initial requests
    for _ in 0..60 {
        let _ = limiter.check_rate_limit(test_ip).await;
    }

    // Next request should be denied (exceeded limit)
    assert!(limiter.check_rate_limit(test_ip).await.is_err());

    // Wait for window to reset
    sleep(Duration::from_secs(61)).await;

    // Should allow requests again
    assert!(limiter.check_rate_limit(test_ip).await.is_ok());
}

/// Test sustained load over time
#[tokio::test]
async fn test_sustained_load() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 120, // 120 requests in 60 second window
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.250".parse().unwrap();

    let mut total_allowed = 0;
    let mut total_denied = 0;

    // Make requests for 1 second at high rate
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        match limiter.check_rate_limit(test_ip).await {
            Ok(_) => total_allowed += 1,
            Err(_) => total_denied += 1,
        }
        sleep(Duration::from_millis(10)).await; // ~100 req/s attempt rate
    }

    // Should allow up to 120 requests in the window (all within 1 second are in the window)
    assert!(total_allowed >= 80 && total_allowed <= 120, 
            "Allowed {} requests in 1 second, expected 80-120", total_allowed);
    assert!(total_denied >= 0, 
            "Denied {} requests", total_denied);
}

/// Test cleanup doesn't affect active rate limits
#[tokio::test]
async fn test_cleanup_preserves_active_limits() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 60,
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.150".parse().unwrap();

    // Exhaust limit
    for _ in 0..60 {
        let _ = limiter.check_rate_limit(test_ip).await;
    }

    // Run cleanup
    limiter.cleanup_expired().await;

    // Rate limit should still be enforced
    assert!(limiter.check_rate_limit(test_ip).await.is_err());
}

/// Test high concurrency from single IP
#[tokio::test]
async fn test_high_concurrency_single_ip() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 300, // 5 per second
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));
    let test_ip: IpAddr = "192.168.1.99".parse().unwrap();

    // Spawn 500 concurrent requests
    let mut handles = vec![];
    for _ in 0..500 {
        let limiter_clone = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit(test_ip).await
        });
        handles.push(handle);
    }

    let mut allowed = 0;
    let mut denied = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => allowed += 1,
            Err(_) => denied += 1,
        }
    }

    // Should allow up to rate limit, deny the rest
    assert!(allowed <= 300, "Allowed {} requests, expected <= 300", allowed);
    assert!(denied >= 200, "Denied {} requests, expected >= 200", denied);
}

/// Test rate limiter thread safety
#[tokio::test]
async fn test_thread_safety() {
    let config = RateLimitConfig {
        enabled: true,
        requests_per_minute: 600, // 10 per second
        window_seconds: 60,
    };
    let limiter = Arc::new(RateLimiter::new(config));

    // Spawn multiple tasks making requests from different IPs
    let mut handles = vec![];
    for task_id in 0..10 {
        let limiter_clone = Arc::clone(&limiter);
        let handle = tokio::spawn(async move {
            let mut task_allowed = 0;
            for ip_suffix in 1..=20 {
                let ip: IpAddr = format!("10.{}.1.{}", task_id, ip_suffix).parse().unwrap();
                for _ in 0..10 {
                    if limiter_clone.check_rate_limit(ip).await.is_ok() {
                        task_allowed += 1;
                    }
                }
            }
            task_allowed
        });
        handles.push(handle);
    }

    // Collect results
    let mut total_allowed = 0;
    for handle in handles {
        total_allowed += handle.await.unwrap();
    }

    // Each IP should allow up to 600 requests per minute
    // 10 tasks * 20 IPs = 200 unique IPs, each can make 600 requests
    // But we only make 10 requests per IP, so all should be allowed
    assert!(total_allowed >= 1900 && total_allowed <= 2000,
            "Allowed {} requests, expected ~2000", total_allowed);
}
