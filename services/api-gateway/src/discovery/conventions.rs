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

        // Parse the method name to extract operation and resource(s)
        let (method_type, parent_resource, child_resource) = match self.parse_method_name(method_name) {
            Some(result) => {
                tracing::info!(
                    method_name = %method_name,
                    method_type = ?result.0,
                    parent_resource = ?result.1,
                    child_resource = ?result.2,
                    "✅ CONVENTION: Successfully parsed method name"
                );
                result
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
        let http_path = self.generate_path(&parent_resource, child_resource.as_deref(), &method_type);

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

    /// Parse a gRPC method name to extract the operation type and resource name(s)
    ///
    /// # Arguments
    /// * `method_name` - The gRPC method name (e.g., "GetUser", "RemoveGroupMember")
    ///
    /// # Returns
    /// * `Some((MethodType, parent_resource, Option<child_resource>))` if the method matches a convention
    /// * `None` if the method doesn't match any convention
    fn parse_method_name(&self, method_name: &str) -> Option<(MethodType, String, Option<String>)> {
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
                    "Add" => MethodType::Add,
                    "Remove" => MethodType::Remove,
                    "Publish" => MethodType::Publish,
                    "Unpublish" => MethodType::Unpublish,
                    _ => return None,
                };

                // Extract and normalize the resource name(s)
                let (parent_resource, child_resource) = self.extract_resources(resource, &method_type);

                tracing::debug!(
                    method_name = %method_name,
                    method_type = ?method_type,
                    parent_resource = %parent_resource,
                    child_resource = ?child_resource,
                    "✅ CONVENTION: Extracted resource name(s)"
                );

                return Some((method_type, parent_resource, child_resource));
            }
        }

        tracing::warn!(
            method_name = %method_name,
            "❌ CONVENTION: No pattern matched"
        );

        None
    }

    /// Extract and normalize the resource name(s) from the method suffix
    ///
    /// # Arguments
    /// * `resource` - The resource part after the operation prefix (e.g., "User", "GroupMember")
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * `(parent_resource, Option<child_resource>)` - Normalized resource name(s) in lowercase
    fn extract_resources(&self, resource: &str, method_type: &MethodType) -> (String, Option<String>) {
        // For nested resource operations (Add, Remove), try to split the compound name
        match method_type {
            MethodType::Add | MethodType::Remove => {
                // These are always nested resources: Add/Remove{Parent}{Child}
                if let Some((parent, child)) = self.split_compound_resource(resource) {
                    tracing::debug!(
                        resource = %resource,
                        parent = %parent,
                        child = %child,
                        "🔍 CONVENTION: Split compound resource for nested operation"
                    );
                    (parent.to_lowercase(), Some(child.to_lowercase()))
                } else {
                    // Fallback: treat as simple resource (shouldn't happen with correct naming)
                    tracing::warn!(
                        resource = %resource,
                        method_type = ?method_type,
                        "⚠️ CONVENTION: Could not split compound resource, treating as simple"
                    );
                    (resource.to_lowercase(), None)
                }
            }
            MethodType::Get => {
                // Could be either:
                // - Simple: "GetUser" -> (user, None)
                // - Nested collection: "GetUserGroups" -> (user, Some(groups))
                // - Nested member: Would use compound name "GetUserGroup" -> (user, Some(group))
                // Try to split, if successful it's nested, otherwise simple
                if let Some((parent, child)) = self.split_compound_resource(resource) {
                    tracing::debug!(
                        resource = %resource,
                        parent = %parent,
                        child = %child,
                        "🔍 CONVENTION: Detected nested resource pattern in Get operation"
                    );
                    (parent.to_lowercase(), Some(child.to_lowercase()))
                } else {
                    // Simple Get operation
                    (resource.to_lowercase(), None)
                }
            }
            MethodType::List => {
                // Resource is already plural (e.g., "ListUsers", "ListGroupMembers")
                // For nested: "ListGroupMembers" would be better as "GetGroupMembers"
                // but we support it: try to split
                if let Some((parent, child)) = self.split_compound_resource(resource) {
                    tracing::debug!(
                        resource = %resource,
                        parent = %parent,
                        child = %child,
                        "🔍 CONVENTION: Detected nested resource pattern in List operation"
                    );
                    (parent.to_lowercase(), Some(child.to_lowercase()))
                } else {
                    // Simple List operation - already plural
                    (resource.to_lowercase(), None)
                }
            }
            MethodType::Publish | MethodType::Unpublish => {
                // Custom actions operate on a single resource
                // PublishCourse -> "course"
                (resource.to_lowercase(), None)
            }
            _ => {
                // Create, Update, Delete are typically simple resources
                // But we still try to split in case of nested patterns
                if let Some((parent, child)) = self.split_compound_resource(resource) {
                    tracing::debug!(
                        resource = %resource,
                        parent = %parent,
                        child = %child,
                        "🔍 CONVENTION: Detected nested resource pattern"
                    );
                    (parent.to_lowercase(), Some(child.to_lowercase()))
                } else {
                    (resource.to_lowercase(), None)
                }
            }
        }
    }

    /// Split a compound CamelCase resource name into parent and child components
    ///
    /// # Arguments
    /// * `resource` - The compound resource name (e.g., "GroupMember", "UserGroups")
    ///
    /// # Returns
    /// * `Some((parent, child))` if the resource can be split
    /// * `None` if it's a simple resource
    ///
    /// # Examples
    /// * "GroupMember" -> Some(("Group", "Member"))
    /// * "UserGroups" -> Some(("User", "Groups"))
    /// * "CourseAssignment" -> Some(("Course", "Assignment"))
    /// * "User" -> None
    fn split_compound_resource(&self, resource: &str) -> Option<(&str, &str)> {
        // Find the position where we transition from lowercase to uppercase
        // This indicates the boundary between parent and child resources
        let chars: Vec<char> = resource.chars().collect();

        // Need at least 2 characters to split
        if chars.len() < 2 {
            return None;
        }

        // Find the last uppercase letter that has a lowercase letter before it
        // This handles cases like "GroupMember", "UserGroups", etc.
        for i in 1..chars.len() {
            if chars[i].is_uppercase() && chars[i - 1].is_lowercase() {
                let parent = &resource[..i];
                let child = &resource[i..];

                // Both parts must be non-empty
                if !parent.is_empty() && !child.is_empty() {
                    return Some((parent, child));
                }
            }
        }

        None
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
            MethodType::Add => "POST",
            MethodType::Remove => "DELETE",
            MethodType::Publish => "POST",
            MethodType::Unpublish => "POST",
        }
    }

    /// Generate the HTTP path for a given resource and method type
    ///
    /// # Arguments
    /// * `parent_resource` - The parent resource name (lowercase)
    /// * `child_resource` - Optional child resource name (lowercase) for nested resources
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * HTTP path following REST conventions
    fn generate_path(
        &self,
        parent_resource: &str,
        child_resource: Option<&str>,
        method_type: &MethodType,
    ) -> String {
        match child_resource {
            // Nested resource paths
            Some(child) => self.generate_nested_path(parent_resource, child, method_type),
            // Simple resource paths
            None => self.generate_simple_path(parent_resource, method_type),
        }
    }

    /// Generate HTTP path for simple (non-nested) resources
    ///
    /// # Arguments
    /// * `resource` - The resource name (lowercase)
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * HTTP path for simple resource
    fn generate_simple_path(&self, resource: &str, method_type: &MethodType) -> String {
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
            MethodType::Add | MethodType::Remove => {
                // These should have a child resource, but if not, treat as simple
                let plural = self.pluralize(resource);
                format!("{}/{}/:id", DEFAULT_API_PREFIX, plural)
            }
            MethodType::Publish => {
                // POST /api/{resources}/:id/publish
                let plural = self.pluralize(resource);
                format!("{}/{}/:id/publish", DEFAULT_API_PREFIX, plural)
            }
            MethodType::Unpublish => {
                // POST /api/{resources}/:id/unpublish
                let plural = self.pluralize(resource);
                format!("{}/{}/:id/unpublish", DEFAULT_API_PREFIX, plural)
            }
        }
    }

    /// Generate HTTP path for nested resources
    ///
    /// # Arguments
    /// * `parent` - The parent resource name (lowercase)
    /// * `child` - The child resource name (lowercase)
    /// * `method_type` - The type of operation
    ///
    /// # Returns
    /// * HTTP path for nested resource
    ///
    /// # Examples
    /// * parent="group", child="member", method_type=Add
    ///   -> "/api/groups/:id/members"
    /// * parent="group", child="member", method_type=Remove
    ///   -> "/api/groups/:id/members/:user_id"
    fn generate_nested_path(&self, parent: &str, child: &str, method_type: &MethodType) -> String {
        let parent_plural = self.pluralize(parent);
        let child_plural = self.pluralize(child);

        match method_type {
            MethodType::Add => {
                // POST /api/{parent_resources}/:id/{child_resources}
                // Example: POST /api/groups/:id/members
                format!("{}/{}/:id/{}", DEFAULT_API_PREFIX, parent_plural, child_plural)
            }
            MethodType::Remove => {
                // DELETE /api/{parent_resources}/:id/{child_resources}/:child_id
                // Example: DELETE /api/groups/:id/members/:user_id
                // The child_id parameter is named after the child resource
                let child_id_param = format!(":{}_{}", child, "id");
                format!(
                    "{}/{}/:id/{}/{}",
                    DEFAULT_API_PREFIX, parent_plural, child_plural, child_id_param
                )
            }
            MethodType::Get => {
                // Could be either collection or member
                // If child is plural (ends with 's'), it's a collection
                // Otherwise it's a member
                if child.ends_with('s') || child == child_plural {
                    // GET /api/{parent_resources}/:id/{child_resources}
                    // Example: GET /api/users/:id/groups
                    format!("{}/{}/:id/{}", DEFAULT_API_PREFIX, parent_plural, child)
                } else {
                    // GET /api/{parent_resources}/:id/{child_resources}/:child_id
                    // Example: GET /api/groups/:id/members/:user_id
                    let child_id_param = format!(":{}_{}", child, "id");
                    format!(
                        "{}/{}/:id/{}/{}",
                        DEFAULT_API_PREFIX, parent_plural, child_plural, child_id_param
                    )
                }
            }
            MethodType::List => {
                // GET /api/{parent_resources}/:id/{child_resources}
                // Example: GET /api/groups/:id/members
                // Child is already plural from ListGroupMembers
                format!("{}/{}/:id/{}", DEFAULT_API_PREFIX, parent_plural, child)
            }
            MethodType::Update => {
                // PUT /api/{parent_resources}/:id/{child_resources}/:child_id
                let child_id_param = format!(":{}_{}", child, "id");
                format!(
                    "{}/{}/:id/{}/{}",
                    DEFAULT_API_PREFIX, parent_plural, child_plural, child_id_param
                )
            }
            MethodType::Create => {
                // POST /api/{parent_resources}/:id/{child_resources}
                format!("{}/{}/:id/{}", DEFAULT_API_PREFIX, parent_plural, child_plural)
            }
            MethodType::Delete => {
                // DELETE /api/{parent_resources}/:id/{child_resources}/:child_id
                let child_id_param = format!(":{}_{}", child, "id");
                format!(
                    "{}/{}/:id/{}/{}",
                    DEFAULT_API_PREFIX, parent_plural, child_plural, child_id_param
                )
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
        if word.ends_with("ch") || word.ends_with("sh") || word.ends_with("ss") 
            || word.ends_with('x') || word.ends_with('z') {
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
