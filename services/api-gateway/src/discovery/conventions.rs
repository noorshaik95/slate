use super::constants::{CONVENTION_PATTERNS, DEFAULT_API_PREFIX};
use super::types::{MethodType, RouteMapping};

/// Maps gRPC method names to HTTP routes based on naming conventions
pub struct ConventionMapper;

impl ConventionMapper {
    /// Create a new ConventionMapper
    pub fn new() -> Self {
        Self
    }

    /// Map a gRPC method to HTTP route based on naming conventions
    ///
    /// # Arguments
    /// * `service_name` - The name of the gRPC service (e.g., "UserService")
    /// * `method_name` - The name of the gRPC method (e.g., "GetUser")
    /// * `full_method` - The full gRPC method path (e.g., "user.UserService/GetUser")
    ///
    /// # Returns
    /// * `Some(RouteMapping)` if the method matches a convention
    /// * `None` if the method doesn't match any convention
    pub fn map_method(
        &self,
        _service_name: &str,
        method_name: &str,
        full_method: &str,
    ) -> Option<RouteMapping> {
        tracing::info!(
            method_name = %method_name,
            full_method = %full_method,
            "🔍 CONVENTION: Attempting to map gRPC method to HTTP route"
        );

        // Parse the method name to extract operation and resource
        let (method_type, resource) = match self.parse_method_name(method_name) {
            Some((mt, res)) => {
                tracing::info!(
                    method_name = %method_name,
                    method_type = ?mt,
                    resource = %res,
                    "✅ CONVENTION: Successfully parsed method name"
                );
                (mt, res)
            }
            None => {
                tracing::warn!(
                    method_name = %method_name,
                    "❌ CONVENTION: Method name does not match any convention pattern"
                );
                return None;
            }
        };

        // Generate HTTP method and path based on the method type
        let http_method = self.determine_http_method(&method_type);
        let http_path = self.generate_path(&resource, &method_type);

        tracing::info!(
            method_name = %method_name,
            http_method = %http_method,
            http_path = %http_path,
            grpc_method = %full_method,
            "✅ CONVENTION: Generated HTTP route mapping"
        );

        Some(RouteMapping {
            http_method: http_method.to_string(),
            http_path,
            grpc_method: full_method.to_string(),
        })
    }

    /// Parse a gRPC method name to extract the operation type and resource name
    ///
    /// # Arguments
    /// * `method_name` - The gRPC method name (e.g., "GetUser", "ListUsers")
    ///
    /// # Returns
    /// * `Some((MethodType, resource_name))` if the method matches a convention
    /// * `None` if the method doesn't match any convention
    fn parse_method_name(&self, method_name: &str) -> Option<(MethodType, String)> {
        tracing::debug!(
            method_name = %method_name,
            patterns = ?CONVENTION_PATTERNS,
            "🔍 CONVENTION: Parsing method name against patterns"
        );

        for pattern in CONVENTION_PATTERNS {
            if method_name.starts_with(pattern) {
                let resource = method_name.strip_prefix(pattern)?;

                tracing::debug!(
                    method_name = %method_name,
                    pattern = %pattern,
                    resource = %resource,
                    "🔍 CONVENTION: Method matches pattern"
                );

                // Resource must not be empty
                if resource.is_empty() {
                    tracing::warn!(
                        method_name = %method_name,
                        pattern = %pattern,
                        "❌ CONVENTION: Resource name is empty after stripping pattern"
                    );
                    return None;
                }

                let method_type = match *pattern {
                    "Get" => MethodType::Get,
                    "List" => MethodType::List,
                    "Create" => MethodType::Create,
                    "Update" => MethodType::Update,
                    "Delete" => MethodType::Delete,
                    _ => return None,
                };

                // Extract and normalize the resource name
                let resource_name = self.extract_resource(resource, &method_type);

                tracing::debug!(
                    method_name = %method_name,
                    method_type = ?method_type,
                    resource_name = %resource_name,
                    "✅ CONVENTION: Extracted resource name"
                );

                return Some((method_type, resource_name));
            }
        }

        tracing::warn!(
            method_name = %method_name,
            "❌ CONVENTION: No pattern matched"
        );

        None
    }

    /// Extract and normalize the resource name from the method suffix
    ///
    /// # Arguments
    /// * `resource` - The resource part after the operation prefix (e.g., "User", "Users")
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * Normalized resource name in lowercase
    fn extract_resource(&self, resource: &str, method_type: &MethodType) -> String {
        // For List operations, the resource is already plural (e.g., "ListUsers")
        // For other operations, we need to pluralize (e.g., "GetUser" -> "users")
        let normalized = resource.to_lowercase();

        match method_type {
            MethodType::List => {
                // Already plural, just return lowercase
                normalized
            }
            _ => {
                // Singular form, will be pluralized in generate_path
                normalized
            }
        }
    }

    /// Determine the HTTP method for a given method type
    ///
    /// # Arguments
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * HTTP method as a string
    fn determine_http_method(&self, method_type: &MethodType) -> &'static str {
        match method_type {
            MethodType::Get => "GET",
            MethodType::List => "GET",
            MethodType::Create => "POST",
            MethodType::Update => "PUT",
            MethodType::Delete => "DELETE",
        }
    }

    /// Generate the HTTP path for a given resource and method type
    ///
    /// # Arguments
    /// * `resource` - The resource name (lowercase)
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * HTTP path following REST conventions
    fn generate_path(&self, resource: &str, method_type: &MethodType) -> String {
        match method_type {
            MethodType::Get => {
                // GET /api/{resources}/:id
                let plural = self.pluralize(resource);
                format!("{}/{}/:id", DEFAULT_API_PREFIX, plural)
            }
            MethodType::List => {
                // GET /api/{resources}
                // Resource is already plural from ListUsers
                format!("{}/{}", DEFAULT_API_PREFIX, resource)
            }
            MethodType::Create => {
                // POST /api/{resources}
                let plural = self.pluralize(resource);
                format!("{}/{}", DEFAULT_API_PREFIX, plural)
            }
            MethodType::Update => {
                // PUT /api/{resources}/:id
                let plural = self.pluralize(resource);
                format!("{}/{}/:id", DEFAULT_API_PREFIX, plural)
            }
            MethodType::Delete => {
                // DELETE /api/{resources}/:id
                let plural = self.pluralize(resource);
                format!("{}/{}/:id", DEFAULT_API_PREFIX, plural)
            }
        }
    }

    /// Pluralize a resource name using simple English rules
    ///
    /// # Arguments
    /// * `word` - The singular resource name
    ///
    /// # Returns
    /// * Pluralized resource name
    fn pluralize(&self, word: &str) -> String {
        // Simple pluralization rules
        if word.is_empty() {
            return word.to_string();
        }

        // Words ending in 'ch', 'sh', 'ss', 'x', 'z' -> add 'es'
        if word.ends_with("ch")
            || word.ends_with("sh")
            || word.ends_with("ss")
            || word.ends_with('x')
            || word.ends_with('z')
        {
            return format!("{}es", word);
        }

        // Already plural (ends with 's')
        if word.ends_with('s') {
            return word.to_string();
        }

        // Words ending in 'y' -> 'ies' (e.g., category -> categories)
        if word.ends_with('y') {
            // Check if the character before 'y' is a consonant
            if word.len() > 1 {
                let before_y = word.chars().nth(word.len() - 2).unwrap();
                if !matches!(before_y, 'a' | 'e' | 'i' | 'o' | 'u') {
                    return format!("{}ies", &word[..word.len() - 1]);
                }
            }
        }

        // Default: just add 's'
        format!("{}s", word)
    }
}

impl Default for ConventionMapper {
    fn default() -> Self {
        Self::new()
    }
}
