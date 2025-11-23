use super::types::{HealthState, HealthStatus, ServiceHealth};
use std::collections::HashMap;

#[test]
fn test_health_state_serialization() {
    let healthy = HealthState::Healthy;
    let json = serde_json::to_string(&healthy).unwrap();
    assert_eq!(json, "\"healthy\"");

    let unhealthy = HealthState::Unhealthy;
    let json = serde_json::to_string(&unhealthy).unwrap();
    assert_eq!(json, "\"unhealthy\"");

    let unknown = HealthState::Unknown;
    let json = serde_json::to_string(&unknown).unwrap();
    assert_eq!(json, "\"unknown\"");
}

#[test]
fn test_service_health_creation() {
    let service_health = ServiceHealth {
        name: "test-service".to_string(),
        status: HealthState::Healthy,
        last_check: "2024-01-01T00:00:00Z".to_string(),
    };

    assert_eq!(service_health.name, "test-service");
    assert_eq!(service_health.status, HealthState::Healthy);
    assert_eq!(service_health.last_check, "2024-01-01T00:00:00Z");
}

#[test]
fn test_health_status_serialization() {
    let mut services = HashMap::new();
    services.insert(
        "service1".to_string(),
        ServiceHealth {
            name: "service1".to_string(),
            status: HealthState::Healthy,
            last_check: "2024-01-01T00:00:00Z".to_string(),
        },
    );

    let health_status = HealthStatus {
        healthy: true,
        services,
    };

    let json = serde_json::to_value(&health_status).unwrap();
    assert_eq!(json["healthy"], true);
    assert_eq!(json["services"]["service1"]["status"], "healthy");
}

#[test]
fn test_health_status_unhealthy() {
    let mut services = HashMap::new();
    services.insert(
        "service1".to_string(),
        ServiceHealth {
            name: "service1".to_string(),
            status: HealthState::Unhealthy,
            last_check: "2024-01-01T00:00:00Z".to_string(),
        },
    );

    let health_status = HealthStatus {
        healthy: false,
        services,
    };

    assert!(!health_status.healthy);
    assert_eq!(
        health_status.services.get("service1").unwrap().status,
        HealthState::Unhealthy
    );
}
