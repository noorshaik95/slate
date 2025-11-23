//! Authentication type definitions.
//!
//! These types are part of the public API for authentication.

#![allow(dead_code)]

use std::time::{Duration, Instant};

/// Error types for authorization operations
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication token")]
    MissingToken,

    #[error("Invalid authentication token: {0}")]
    InvalidToken(String),

    #[error("Expired authentication token")]
    ExpiredToken,

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Auth service error: {0}")]
    ServiceError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Policy cache error: {0}")]
    CacheError(String),
}

impl From<tonic::transport::Error> for AuthError {
    fn from(err: tonic::transport::Error) -> Self {
        AuthError::ConnectionError(err.to_string())
    }
}

impl From<tonic::Status> for AuthError {
    fn from(status: tonic::Status) -> Self {
        AuthError::ServiceError(status.to_string())
    }
}

/// Represents an authentication token with optional parsed claims
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub raw: String,
    pub claims: Option<TokenClaims>,
}

/// Claims extracted from a validated token
#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub roles: Vec<String>,
    pub exp: i64,
}

/// Result of token validation
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub valid: bool,
    pub claims: Option<TokenClaims>,
    pub error: Option<String>,
}

/// Authorization policy for a specific service method
#[derive(Debug, Clone)]
pub struct AuthPolicy {
    pub service: String,
    pub method: String,
    pub require_auth: bool,
    pub required_roles: Vec<String>,
    pub cached_at: Instant,
    pub cache_ttl: Duration,
}

impl AuthPolicy {
    /// Check if the cached policy is still valid
    pub fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.cache_ttl
    }
}

/// Cache key for authorization policies
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct PolicyCacheKey {
    pub(crate) service: String,
    pub(crate) method: String,
}
