use std::collections::{HashMap, HashSet};
use tracing::{error, warn};

use super::DiscoveryError;
use crate::config::RouteConfig;

/// Validator for route configurations
pub struct RouteValidator;

impl RouteValidator {
    /// Check for duplicate routes and return an error if found
    ///
    /// # Arguments
    /// * `routes` - List of routes to check
    ///
    /// # Returns
    /// * `Ok(())` if no duplicates found
    /// * `Err(DiscoveryError::DuplicateRoute)` if duplicates detected
    pub fn check_duplicates(routes: &[RouteConfig]) -> Result<(), DiscoveryError> {
        let mut seen: HashMap<String, String> = HashMap::new();
        let mut duplicates = Vec::new();

        for route in routes {
            let key = format!("{} {}", route.method, route.path);

            if let Some(existing_method) = seen.get(&key) {
                // Duplicate found
                error!(
                    method1 = %existing_method,
                    method2 = %route.grpc_method,
                    http_route = %key,
                    "Duplicate route detected"
                );

                duplicates.push((
                    existing_method.clone(),
                    route.grpc_method.clone(),
                    key.clone(),
                ));
            } else {
                seen.insert(key, route.grpc_method.clone());
            }
        }

        if !duplicates.is_empty() {
            // Return the first duplicate as an error
            let (method1, method2, http_route) = duplicates.into_iter().next().unwrap();
            return Err(DiscoveryError::DuplicateRoute {
                method1,
                method2,
                http_route,
            });
        }

        Ok(())
    }

    /// Remove duplicate routes, keeping the first occurrence
    ///
    /// # Arguments
    /// * `routes` - List of routes that may contain duplicates
    ///
    /// # Returns
    /// * List of routes with duplicates removed
    pub fn deduplicate_routes(routes: Vec<RouteConfig>) -> Vec<RouteConfig> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for route in routes {
            let key = format!("{} {}", route.method, route.path);

            if seen.insert(key.clone()) {
                // First occurrence, keep it
                result.push(route);
            } else {
                // Duplicate, skip it
                warn!(
                    grpc_method = %route.grpc_method,
                    http_route = %key,
                    "Skipping duplicate route (keeping first occurrence)"
                );
            }
        }

        result
    }
}
