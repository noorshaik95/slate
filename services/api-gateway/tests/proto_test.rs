// Test to verify proto definitions are properly generated and accessible

#[cfg(test)]
mod tests {
    // Include the generated proto code
    pub mod auth {
        tonic::include_proto!("auth");
    }

    pub mod gateway {
        tonic::include_proto!("gateway");
    }

    #[test]
    fn test_auth_proto_structs_exist() {
        // Verify ValidateTokenRequest can be instantiated
        let request = auth::ValidateTokenRequest {
            token: "test_token".to_string(),
        };
        assert_eq!(request.token, "test_token");

        // Verify ValidateTokenResponse can be instantiated
        let response = auth::ValidateTokenResponse {
            valid: true,
            user_id: "user123".to_string(),
            roles: vec!["admin".to_string()],
            error: String::new(),
        };
        assert!(response.valid);
        assert_eq!(response.user_id, "user123");
        assert_eq!(response.roles.len(), 1);
    }

    #[test]
    fn test_gateway_proto_structs_exist() {
        // Verify AuthPolicyRequest can be instantiated
        let request = gateway::AuthPolicyRequest {
            grpc_method: "user.UserService/GetUser".to_string(),
        };
        assert_eq!(request.grpc_method, "user.UserService/GetUser");

        // Verify AuthPolicyResponse can be instantiated
        let response = gateway::AuthPolicyResponse {
            require_auth: true,
            required_roles: vec!["admin".to_string()],
            cache_ttl_seconds: 300,
        };
        assert!(response.require_auth);
        assert_eq!(response.required_roles.len(), 1);
        assert_eq!(response.cache_ttl_seconds, 300);
    }
}
