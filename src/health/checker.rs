use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info};

use crate::grpc::client::GrpcClientPool;
use super::types::{HealthState, HealthStatus, ServiceHealth};

/// Health checker for monitoring backend services
pub struct HealthChecker {
    grpc_pool: Arc<GrpcClientPool>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(grpc_pool: Arc<GrpcClientPool>) -> Self {
        info!("Initializing health checker");
        Self { grpc_pool }
    }

    /// Check health of all backend services
    pub async fn check_health(&self) -> HealthStatus {
        debug!("Starting health check for all services");
        
        let services = self.grpc_pool.services();
        let mut service_health_map = HashMap::new();
        let mut all_healthy = true;
        let check_time = Instant::now();

        for service_name in services {
            debug!(service = %service_name, "Checking service health");
            
            let health_state = match self.grpc_pool.health_check(&service_name).await {
                Ok(true) => {
                    debug!(service = %service_name, "Service is healthy");
                    HealthState::Healthy
                }
                Ok(false) => {
                    error!(service = %service_name, "Service is unhealthy");
                    all_healthy = false;
                    HealthState::Unhealthy
                }
                Err(e) => {
                    error!(service = %service_name, error = %e, "Health check failed");
                    all_healthy = false;
                    HealthState::Unhealthy
                }
            };

            let service_health = ServiceHealth {
                name: service_name.clone(),
                status: health_state,
                last_check: chrono::Utc::now().to_rfc3339(),
            };

            service_health_map.insert(service_name, service_health);
        }

        let elapsed = check_time.elapsed();
        info!(
            healthy = all_healthy,
            services_checked = service_health_map.len(),
            duration_ms = elapsed.as_millis(),
            "Health check completed"
        );

        HealthStatus {
            healthy: all_healthy,
            services: service_health_map,
        }
    }
}
