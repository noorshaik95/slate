use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health status of a component or service
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health information for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Overall service health with component details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
}

impl ServiceHealth {
    /// Create a new service health response
    pub fn new(status: HealthStatus, components: Vec<ComponentHealth>) -> Self {
        Self { status, components }
    }

    /// Create a healthy response with no components
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            components: vec![],
        }
    }
}
