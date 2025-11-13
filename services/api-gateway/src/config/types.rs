use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::circuit_breaker::CircuitBreakerConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub services: HashMap<String, ServiceConfig>,
    pub auth: AuthConfig,
    pub rate_limit: Option<RateLimitConfig>,
    pub observability: ObservabilityConfig,
    pub discovery: DiscoveryConfig,
    #[serde(default)]
    pub route_overrides: Vec<RouteOverride>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_request_timeout_ms")]
    pub request_timeout_ms: u64,
}

fn default_request_timeout_ms() -> u64 {
    30000 // 30 seconds
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteConfig {
    pub path: String,
    pub method: String,
    pub service: String,
    pub grpc_method: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub connection_pool_size: usize,
    #[serde(default)]
    pub auto_discover: bool,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default)]
    pub tls_domain: Option<String>,
    #[serde(default)]
    pub tls_ca_cert_path: Option<String>,
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub service_endpoint: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub window_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    pub tempo_endpoint: String,
    pub service_name: String,
    #[serde(default = "default_otlp_timeout_secs")]
    pub otlp_timeout_secs: u64,
    #[serde(default = "default_max_events_per_span")]
    pub max_events_per_span: u32,
    #[serde(default = "default_max_attributes_per_span")]
    pub max_attributes_per_span: u32,
}

fn default_otlp_timeout_secs() -> u64 {
    3 // 3 seconds
}

fn default_max_events_per_span() -> u32 {
    64
}

fn default_max_attributes_per_span() -> u32 {
    16
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoveryConfig {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_seconds: u64,
    #[serde(default)]
    pub enabled: bool,
}

fn default_refresh_interval() -> u64 {
    300 // 5 minutes
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RouteOverride {
    pub grpc_method: String,
    pub http_path: Option<String>,
    pub http_method: Option<String>,
}
