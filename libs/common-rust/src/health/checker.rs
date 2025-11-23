use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{ComponentHealth, HealthStatus, ServiceHealth};

/// Trait for implementing health checks
pub trait HealthCheck: Send + Sync {
    fn check(&self) -> Pin<Box<dyn Future<Output = ComponentHealth> + Send + '_>>;
}

/// Health checker for managing service health
pub struct HealthChecker {
    service_name: String,
    components: Arc<RwLock<Vec<Box<dyn HealthCheck>>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            components: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a health check component
    pub async fn register(&self, check: Box<dyn HealthCheck>) {
        self.components.write().await.push(check);
    }

    /// Liveness check - always returns healthy if service is running
    pub async fn liveness(&self) -> ServiceHealth {
        ServiceHealth::healthy()
    }

    /// Readiness check - checks all registered components
    pub async fn readiness(&self) -> ServiceHealth {
        let components = self.components.read().await;

        let mut component_results = Vec::new();
        for component in components.iter() {
            component_results.push(component.check().await);
        }

        // Aggregate status: unhealthy > degraded > healthy
        let overall_status = if component_results
            .iter()
            .any(|c| c.status == HealthStatus::Unhealthy)
        {
            HealthStatus::Unhealthy
        } else if component_results
            .iter()
            .any(|c| c.status == HealthStatus::Degraded)
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        ServiceHealth::new(overall_status, component_results)
    }
}

// Example health check implementations

/// Circuit breaker health check
pub struct CircuitBreakerHealthCheck {
    name: String,
    breaker: crate::circuit_breaker::CircuitBreaker,
}

impl CircuitBreakerHealthCheck {
    pub fn new(name: String, breaker: crate::circuit_breaker::CircuitBreaker) -> Self {
        Self { name, breaker }
    }
}

impl HealthCheck for CircuitBreakerHealthCheck {
    fn check(&self) -> Pin<Box<dyn Future<Output = ComponentHealth> + Send + '_>> {
        Box::pin(async {
            let state = self.breaker.get_state().await;
            let status = match state {
                crate::circuit_breaker::CircuitState::Closed => HealthStatus::Healthy,
                crate::circuit_breaker::CircuitState::HalfOpen => HealthStatus::Degraded,
                crate::circuit_breaker::CircuitState::Open { .. } => HealthStatus::Unhealthy,
            };

            ComponentHealth {
                name: self.name.clone(),
                status,
                message: Some(format!("Circuit breaker state: {:?}", state)),
                details: None,
            }
        })
    }
}
