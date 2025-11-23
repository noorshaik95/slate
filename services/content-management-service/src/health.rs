use crate::db::DatabasePool;
use common_rust::circuit_breaker::{CircuitBreaker, CircuitState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, warn};

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub circuit_breaker_state: Option<String>,
}

/// Overall service health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: HealthStatus,
    pub components: Vec<ComponentHealth>,
}

/// Health checker for the content management service
pub struct HealthChecker {
    db_pool: Arc<DatabasePool>,
    elasticsearch_circuit_breaker: Option<Arc<CircuitBreaker>>,
    analytics_circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(db_pool: Arc<DatabasePool>) -> Self {
        Self {
            db_pool,
            elasticsearch_circuit_breaker: None,
            analytics_circuit_breaker: None,
        }
    }

    /// Set the ElasticSearch circuit breaker for monitoring
    pub fn with_elasticsearch_circuit_breaker(mut self, cb: Arc<CircuitBreaker>) -> Self {
        self.elasticsearch_circuit_breaker = Some(cb);
        self
    }

    /// Set the Analytics circuit breaker for monitoring
    pub fn with_analytics_circuit_breaker(mut self, cb: Arc<CircuitBreaker>) -> Self {
        self.analytics_circuit_breaker = Some(cb);
        self
    }

    /// Perform liveness check (is the service running?)
    pub async fn liveness(&self) -> ServiceHealth {
        ServiceHealth {
            status: HealthStatus::Healthy,
            components: vec![ComponentHealth {
                name: "service".to_string(),
                status: HealthStatus::Healthy,
                message: Some("Service is running".to_string()),
                circuit_breaker_state: None,
            }],
        }
    }

    /// Perform readiness check (is the service ready to handle requests?)
    pub async fn readiness(&self) -> ServiceHealth {
        let mut components = Vec::new();

        // Check database
        let db_health = self.check_database().await;
        components.push(db_health);

        // Check ElasticSearch circuit breaker
        if let Some(cb) = &self.elasticsearch_circuit_breaker {
            let es_health = self.check_circuit_breaker("elasticsearch", cb).await;
            components.push(es_health);
        }

        // Check Analytics circuit breaker
        if let Some(cb) = &self.analytics_circuit_breaker {
            let analytics_health = self.check_circuit_breaker("analytics_service", cb).await;
            components.push(analytics_health);
        }

        // Determine overall status
        let overall_status = if components
            .iter()
            .any(|c| c.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else if components
            .iter()
            .any(|c| c.status == HealthStatus::Degraded)
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        ServiceHealth {
            status: overall_status,
            components,
        }
    }

    /// Check database health
    async fn check_database(&self) -> ComponentHealth {
        match self.db_pool.health_check().await {
            Ok(_) => {
                debug!("Database health check passed");
                ComponentHealth {
                    name: "database".to_string(),
                    status: HealthStatus::Healthy,
                    message: Some("Database connection is healthy".to_string()),
                    circuit_breaker_state: None,
                }
            }
            Err(e) => {
                warn!("Database health check failed: {}", e);
                ComponentHealth {
                    name: "database".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: Some(format!("Database connection failed: {}", e)),
                    circuit_breaker_state: None,
                }
            }
        }
    }

    /// Check circuit breaker state
    async fn check_circuit_breaker(
        &self,
        name: &str,
        circuit_breaker: &CircuitBreaker,
    ) -> ComponentHealth {
        let state = circuit_breaker.get_state().await;
        let stats = circuit_breaker.get_stats().await;

        let (status, message) = match state {
            CircuitState::Closed => (HealthStatus::Healthy, format!("{} is healthy", name)),
            CircuitState::HalfOpen => (
                HealthStatus::Degraded,
                format!("{} is recovering (half-open)", name),
            ),
            CircuitState::Open { .. } => (
                HealthStatus::Degraded,
                format!(
                    "{} circuit breaker is open (failures: {})",
                    name, stats.consecutive_failures
                ),
            ),
        };

        ComponentHealth {
            name: name.to_string(),
            status,
            message: Some(message),
            circuit_breaker_state: Some(format!("{:?}", state)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");

        let status = HealthStatus::Degraded;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"degraded\"");

        let status = HealthStatus::Unhealthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"unhealthy\"");
    }

    #[test]
    fn test_component_health_serialization() {
        let component = ComponentHealth {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            message: Some("All good".to_string()),
            circuit_breaker_state: None,
        };

        let json = serde_json::to_string(&component).unwrap();
        assert!(json.contains("database"));
        assert!(json.contains("healthy"));
        assert!(json.contains("All good"));
    }

    #[test]
    fn test_service_health_overall_status() {
        let components = vec![
            ComponentHealth {
                name: "db".to_string(),
                status: HealthStatus::Healthy,
                message: None,
                circuit_breaker_state: None,
            },
            ComponentHealth {
                name: "search".to_string(),
                status: HealthStatus::Degraded,
                message: None,
                circuit_breaker_state: Some("HalfOpen".to_string()),
            },
        ];

        // Should be degraded if any component is degraded
        let has_degraded = components
            .iter()
            .any(|c| c.status == HealthStatus::Degraded);
        assert!(has_degraded);
    }
}
