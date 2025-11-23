use super::types::{AuthPolicy, PolicyCacheKey};
use super::*;
use axum::http::HeaderMap;
use std::time::{Duration, Instant};

#[test]
fn test_extract_token_with_bearer_prefix() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer test-token-123".parse().unwrap());

    let token = AuthService::extract_token(&headers);
    assert_eq!(token, Some("test-token-123".to_string()));
}

#[test]
fn test_extract_token_with_lowercase_bearer() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "bearer test-token-456".parse().unwrap());

    let token = AuthService::extract_token(&headers);
    assert_eq!(token, Some("test-token-456".to_string()));
}

#[test]
fn test_extract_token_without_bearer() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "raw-token-789".parse().unwrap());

    let token = AuthService::extract_token(&headers);
    assert_eq!(token, Some("raw-token-789".to_string()));
}

#[test]
fn test_extract_token_missing() {
    let headers = HeaderMap::new();

    let token = AuthService::extract_token(&headers);
    assert_eq!(token, None);
}

#[test]
fn test_auth_policy_is_valid() {
    let policy = AuthPolicy {
        service: "test".to_string(),
        method: "test".to_string(),
        require_auth: true,
        required_roles: vec![],
        cached_at: Instant::now(),
        cache_ttl: Duration::from_secs(300),
    };

    assert!(policy.is_valid());
}

#[test]
fn test_auth_policy_expired() {
    let policy = AuthPolicy {
        service: "test".to_string(),
        method: "test".to_string(),
        require_auth: true,
        required_roles: vec![],
        cached_at: Instant::now() - Duration::from_secs(400),
        cache_ttl: Duration::from_secs(300),
    };

    assert!(!policy.is_valid());
}

#[test]
fn test_policy_cache_key_equality() {
    let key1 = PolicyCacheKey {
        service: "service1".to_string(),
        method: "method1".to_string(),
    };

    let key2 = PolicyCacheKey {
        service: "service1".to_string(),
        method: "method1".to_string(),
    };

    let key3 = PolicyCacheKey {
        service: "service2".to_string(),
        method: "method1".to_string(),
    };

    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
}
