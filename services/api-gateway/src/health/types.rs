use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health state of a service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Unhealthy,
    Unknown,
}

/// Health information for a single service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthState,
    pub last_check: String, // ISO 8601 timestamp
}

/// Overall health status of the gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub services: HashMap<String, ServiceHealth>,
}
