use super::*;
use std::collections::HashMap;

#[test]
fn test_validate_empty_server_host() {
    let mut config = create_valid_config();
    config.server.host = String::new();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("host cannot be empty"));
}

#[test]
fn test_validate_zero_port() {
    let mut config = create_valid_config();
    config.server.port = 0;

    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("port must be greater than 0"));
}

// Routes are now discovered automatically, so these tests are no longer applicable
// #[test]
// fn test_validate_empty_routes() {
//     let mut config = create_valid_config();
//     config.routes.clear();
//
//     let result = config.validate();
//     assert!(result.is_err());
//     assert!(result.unwrap_err().to_string().contains("At least one route"));
// }

// #[test]
// fn test_validate_route_references_unknown_service() {
//     let mut config = create_valid_config();
//     config.routes[0].service = "unknown-service".to_string();
//
//     let result = config.validate();
//     assert!(result.is_err());
//     assert!(result.unwrap_err().to_string().contains("unknown service"));
// }

#[test]
fn test_validate_valid_config() {
    let config = create_valid_config();
    assert!(config.validate().is_ok());
}

fn create_valid_config() -> GatewayConfig {
    use common_rust::circuit_breaker::CircuitBreakerConfig;

    let mut services = HashMap::new();
    services.insert(
        "test-service".to_string(),
        ServiceConfig {
            name: "test-service".to_string(),
            endpoint: "http://localhost:50051".to_string(),
            timeout_ms: 5000,
            connection_pool_size: 5,
            auto_discover: true,
            tls_enabled: false,
            tls_domain: None,
            tls_ca_cert_path: None,
            circuit_breaker: CircuitBreakerConfig::default(),
        },
    );

    GatewayConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            request_timeout_ms: 30000,
        },
        services,
        auth: AuthConfig {
            service_endpoint: "http://localhost:50052".to_string(),
            timeout_ms: 2000,
            public_routes: vec![],
        },
        rate_limit: Some(RateLimitConfig {
            enabled: true,
            requests_per_minute: 100,
            window_seconds: 60,
        }),
        observability: ObservabilityConfig {
            tempo_endpoint: "http://localhost:4317".to_string(),
            service_name: "api-gateway".to_string(),
            otlp_timeout_secs: 3,
            max_events_per_span: 64,
            max_attributes_per_span: 16,
        },
        discovery: DiscoveryConfig {
            enabled: true,
            refresh_interval_seconds: 300,
        },
        route_overrides: vec![],
        cors: None,
        body_limit: None,
        trusted_proxies: vec![],
    }
}
