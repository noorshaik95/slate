use std::collections::HashMap;
use tracing::info;

use crate::config::{RouteConfig, RouteOverride};

/// Handler for applying manual route overrides
pub struct OverrideHandler;

impl OverrideHandler {
    /// Apply manual overrides to discovered routes
    ///
    /// # Arguments
    /// * `discovered` - List of discovered routes
    /// * `overrides` - List of manual route overrides
    ///
    /// # Returns
    /// * List of routes with overrides applied
    pub fn apply_overrides(
        discovered: Vec<RouteConfig>,
        overrides: &[RouteOverride],
    ) -> Vec<RouteConfig> {
        if overrides.is_empty() {
            return discovered;
        }

        info!(
            discovered_routes = discovered.len(),
            overrides = overrides.len(),
            "Applying route overrides"
        );

        // Build a map of overrides by grpc_method for quick lookup
        let override_map: HashMap<String, &RouteOverride> = overrides
            .iter()
            .map(|o| (o.grpc_method.clone(), o))
            .collect();

        let mut result = Vec::new();

        for route in discovered {
            if let Some(override_config) = override_map.get(&route.grpc_method) {
                // Apply override
                let overridden_route = RouteConfig {
                    path: override_config
                        .http_path
                        .clone()
                        .unwrap_or_else(|| route.path.clone()),
                    method: override_config
                        .http_method
                        .clone()
                        .unwrap_or_else(|| route.method.clone()),
                    service: route.service.clone(),
                    grpc_method: route.grpc_method.clone(),
                };

                info!(
                    grpc_method = %route.grpc_method,
                    original_path = %route.path,
                    original_method = %route.method,
                    override_path = %overridden_route.path,
                    override_method = %overridden_route.method,
                    "Applied route override"
                );

                result.push(overridden_route);
            } else {
                // No override, use discovered route as-is
                result.push(route);
            }
        }

        result
    }
}
