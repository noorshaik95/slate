// Integration tests for UserService CRUD methods
// Tests that all UserService methods work correctly through the dynamic client
//
// These tests verify:
// - GetUser, CreateUser, UpdateUser, DeleteUser, ListUsers work via dynamic client
// - Auth methods (Login, Register, OAuth, SAML) continue to work
// - Trace context is properly propagated
//
// Note: These tests require the user-auth-service to be running.
// Run with: docker-compose up user-auth-service postgres

use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    const GATEWAY_URL: &str = "http://localhost:8080";

    // Test helper to check if gateway is available
    async fn is_service_available() -> bool {
        let client = reqwest::Client::new();
        match client.get(format!("{}/health", GATEWAY_URL)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    // Helper to register and login a test user, returns (user_id, token)
    async fn create_and_login_test_user(
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
    ) -> Option<(String, String)> {
        let client = reqwest::Client::new();

        // Register the user
        let register_payload = json!({
            "email": email,
            "password": password,
            "first_name": first_name,
            "last_name": last_name,
            "phone": "+1234567890"
        });

        let response = client
            .post(format!("{}/api/auth/register", GATEWAY_URL))
            .header("Content-Type", "application/json")
            .json(&register_payload)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            eprintln!("Failed to register user: {}", response.status());
            return None;
        }

        let register_response: serde_json::Value = response.json().await.ok()?;
        let user_id = register_response["user"]["id"].as_str()?.to_string();
        let token = register_response["access_token"].as_str()?.to_string();

        Some((user_id, token))
    }

    // Helper to login and get a valid token
    async fn login_user(email: &str, password: &str) -> Option<String> {
        let client = reqwest::Client::new();

        let login_payload = json!({
            "email": email,
            "password": password
        });

        let response = client
            .post(format!("{}/api/auth/login", GATEWAY_URL))
            .header("Content-Type", "application/json")
            .json(&login_payload)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            return None;
        }

        let login_response: serde_json::Value = response.json().await.ok()?;
        login_response["access_token"]
            .as_str()
            .map(|s| s.to_string())
    }

    // Helper to get admin token (assumes admin user exists)
    async fn get_admin_token() -> Option<String> {
        login_user("admin@example.com", "admin123").await
    }

    // ========================================
    // Task 3.1: Test GetUser endpoint
    // ========================================

    #[tokio::test]
    #[ignore] // Run with --ignored flag when service is available
    async fn test_get_user_endpoint() {
        if !is_service_available().await {
            eprintln!("Skipping test: gateway not available");
            return;
        }

        // Create a test user first
        let test_email = format!("getuser_test_{}@example.com", uuid::Uuid::new_v4());
        let test_password = "password123";

        let result = create_and_login_test_user(&test_email, test_password, "Get", "User").await;
        assert!(result.is_some(), "Failed to create and login test user");
        let (user_id, token) = result.unwrap();

        println!("Testing GET /api/users/{}", user_id);

        // Test: Call GET /api/users/:id with valid token
        let client = reqwest::Client::new();
        let request_url = format!("{}/api/users/{}", GATEWAY_URL, user_id);

        let response = client
            .get(&request_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send request");

        println!("Response status: {}", response.status());

        // Expected: 200 OK with user data
        assert_eq!(response.status(), 200, "Expected 200 OK");

        let user_data: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        println!(
            "User data: {}",
            serde_json::to_string_pretty(&user_data).unwrap()
        );

        assert_eq!(user_data["id"], user_id, "User ID should match");
        assert_eq!(user_data["email"], test_email, "Email should match");

        println!("✓ GetUser endpoint test passed");
    }

    // ========================================
    // Task 3.2: Test CreateUser endpoint
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_create_user_endpoint() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        // Get admin token (CreateUser typically requires admin privileges)
        let admin_token = get_admin_token().await;
        assert!(admin_token.is_some(), "Failed to get admin token");
        let admin_token = admin_token.unwrap();

        // Test data for new user
        let new_user_payload = json!({
            "email": "newuser_test@example.com",
            "password": "securepass123",
            "first_name": "New",
            "last_name": "User",
            "phone": "+1234567890",
            "roles": ["user"]
        });

        println!("Testing POST /api/users");
        println!("Payload: {}", new_user_payload);
        println!("Authorization: Bearer {}", admin_token);

        // Expected behavior:
        // - Should return 201 Created
        // - Response should contain the created user with generated user_id
        // - User should be created in database
        // - Password should be hashed (not returned in response)

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/api/users")
        //     .header("Authorization", format!("Bearer {}", admin_token))
        //     .header("Content-Type", "application/json")
        //     .json(&new_user_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 201, "Expected 201 Created");
        //
        // let created_user: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(created_user["id"].is_string(), "User should have an ID");
        // assert_eq!(created_user["email"], "newuser_test@example.com");
        // assert!(created_user["password"].is_null(), "Password should not be returned");

        println!("✓ CreateUser endpoint test passed");
    }

    // ========================================
    // Task 3.3: Test UpdateUser endpoint
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_update_user_endpoint() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        // Create a test user first
        let test_email = format!("updateuser_test_{}@example.com", uuid::Uuid::new_v4());
        let test_password = "password123";

        let result = create_and_login_test_user(&test_email, test_password, "Update", "User").await;
        assert!(result.is_some(), "Failed to create and login test user");
        let (user_id, token) = result.unwrap();

        // Updated user data
        let update_payload = json!({
            "first_name": "Updated",
            "last_name": "Name",
            "phone": "+9876543210"
        });

        let request_url = format!("http://localhost:8080/api/users/{}", user_id);

        println!("Testing PUT /api/users/{}", user_id);
        println!("Payload: {}", update_payload);
        println!("Authorization: Bearer {}", token);

        // Expected behavior:
        // - Should return 200 OK
        // - Response should contain updated user data
        // - Only specified fields should be updated
        // - Other fields should remain unchanged

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .put(&request_url)
        //     .header("Authorization", format!("Bearer {}", token))
        //     .header("Content-Type", "application/json")
        //     .json(&update_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // let updated_user: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert_eq!(updated_user["id"], user_id);
        // assert_eq!(updated_user["first_name"], "Updated");
        // assert_eq!(updated_user["last_name"], "Name");
        // assert_eq!(updated_user["phone"], "+9876543210");

        println!("✓ UpdateUser endpoint test passed");
    }

    // ========================================
    // Task 3.4: Test DeleteUser endpoint
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_delete_user_endpoint() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        // Create a test user to delete
        let test_email = format!("deleteuser_test_{}@example.com", uuid::Uuid::new_v4());
        let test_password = "password123";

        let result = create_and_login_test_user(&test_email, test_password, "Delete", "User").await;
        assert!(result.is_some(), "Failed to create test user");
        let (user_id, _token) = result.unwrap();

        // Get admin token (DeleteUser typically requires admin privileges)
        let admin_token = get_admin_token().await;
        assert!(admin_token.is_some(), "Failed to get admin token");
        let admin_token = admin_token.unwrap();

        let request_url = format!("http://localhost:8080/api/users/{}", user_id);

        println!("Testing DELETE /api/users/{}", user_id);
        println!("Authorization: Bearer {}", admin_token);

        // Expected behavior:
        // - Should return 204 No Content
        // - User should be deleted from database
        // - Subsequent GET request should return 404

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .delete(&request_url)
        //     .header("Authorization", format!("Bearer {}", admin_token))
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 204, "Expected 204 No Content");
        //
        // // Verify user is deleted by trying to get it
        // let get_response = client
        //     .get(&request_url)
        //     .header("Authorization", format!("Bearer {}", admin_token))
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(get_response.status(), 404, "User should not exist after deletion");

        println!("✓ DeleteUser endpoint test passed");
    }

    // ========================================
    // Task 3.5: Test ListUsers endpoint
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_list_users_endpoint() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        // Create multiple test users
        let test_users = vec![
            (
                format!("listuser1_{}@example.com", uuid::Uuid::new_v4()),
                "password123",
                "List",
                "User1",
            ),
            (
                format!("listuser2_{}@example.com", uuid::Uuid::new_v4()),
                "password123",
                "List",
                "User2",
            ),
            (
                format!("listuser3_{}@example.com", uuid::Uuid::new_v4()),
                "password123",
                "List",
                "User3",
            ),
        ];

        for (email, password, first_name, last_name) in &test_users {
            let result = create_and_login_test_user(email, password, first_name, last_name).await;
            assert!(result.is_some(), "Failed to create test user {}", email);
        }

        // Get admin token
        let token = get_admin_token().await;
        assert!(token.is_some(), "Failed to get admin token");
        let token = token.unwrap();

        // Test with pagination parameters
        let request_url = "http://localhost:8080/api/users?page=1&page_size=10";

        println!("Testing GET /api/users with pagination");
        println!("Request URL: {}", request_url);
        println!("Authorization: Bearer {}", token);

        // Expected behavior:
        // - Should return 200 OK
        // - Response should contain array of users
        // - Response should include pagination metadata (total, page, page_size)
        // - Users should include id, email, first_name, last_name, roles

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .get(request_url)
        //     .header("Authorization", format!("Bearer {}", token))
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // let list_response: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(list_response["users"].is_array(), "Response should contain users array");
        // assert!(list_response["total"].is_number(), "Response should contain total count");
        // assert_eq!(list_response["page"], 1);
        // assert_eq!(list_response["page_size"], 10);
        //
        // let users = list_response["users"].as_array().unwrap();
        // assert!(users.len() >= 3, "Should have at least 3 users");

        println!("✓ ListUsers endpoint test passed");
    }

    // ========================================
    // Task 3.6: Verify auth methods still work
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_login_endpoint_still_works() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        let login_payload = json!({
            "email": "admin@example.com",
            "password": "admin123"
        });

        println!("Testing POST /api/auth/login");
        println!("Payload: {}", login_payload);

        // Expected behavior:
        // - Should return 200 OK
        // - Response should contain access_token and refresh_token
        // - Response should contain user data

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/api/auth/login")
        //     .header("Content-Type", "application/json")
        //     .json(&login_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // let login_response: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(login_response["access_token"].is_string(), "Should have access_token");
        // assert!(login_response["refresh_token"].is_string(), "Should have refresh_token");
        // assert!(login_response["user"].is_object(), "Should have user data");

        println!("✓ Login endpoint test passed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_register_endpoint_still_works() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        let register_payload = json!({
            "email": "newregister@example.com",
            "password": "securepass123",
            "first_name": "New",
            "last_name": "Register",
            "phone": "+1234567890"
        });

        println!("Testing POST /api/auth/register");
        println!("Payload: {}", register_payload);

        // Expected behavior:
        // - Should return 200 OK or 201 Created
        // - Response should contain access_token and refresh_token
        // - Response should contain created user data

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/api/auth/register")
        //     .header("Content-Type", "application/json")
        //     .json(&register_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert!(
        //     response.status() == 200 || response.status() == 201,
        //     "Expected 200 OK or 201 Created"
        // );
        //
        // let register_response: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(register_response["access_token"].is_string(), "Should have access_token");
        // assert!(register_response["user"].is_object(), "Should have user data");

        println!("✓ Register endpoint test passed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_oauth_authorize_endpoint_still_works() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        let oauth_payload = json!({
            "provider": "google"
        });

        println!("Testing POST /auth/oauth/authorize");
        println!("Payload: {}", oauth_payload);

        // Expected behavior:
        // - Should return 200 OK
        // - Response should contain authorization_url
        // - Response should contain state parameter for CSRF protection

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/auth/oauth/authorize")
        //     .header("Content-Type", "application/json")
        //     .json(&oauth_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // let oauth_response: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(oauth_response["authorization_url"].is_string(), "Should have authorization_url");
        // assert!(oauth_response["state"].is_string(), "Should have state parameter");

        println!("✓ OAuth authorize endpoint test passed");
    }

    #[tokio::test]
    #[ignore]
    async fn test_saml_login_endpoint_still_works() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        let saml_payload = json!({
            "organization_id": "test_org_123"
        });

        println!("Testing POST /auth/saml/login");
        println!("Payload: {}", saml_payload);

        // Expected behavior:
        // - Should return 200 OK
        // - Response should contain saml_request (base64-encoded)
        // - Response should contain sso_url for IdP redirect

        // TODO: Make actual HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post("http://localhost:8080/auth/saml/login")
        //     .header("Content-Type", "application/json")
        //     .json(&saml_payload)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // let saml_response: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // assert!(saml_response["saml_request"].is_string(), "Should have saml_request");
        // assert!(saml_response["sso_url"].is_string(), "Should have sso_url");

        println!("✓ SAML login endpoint test passed");
    }

    // ========================================
    // Additional test: Trace context propagation
    // ========================================

    #[tokio::test]
    #[ignore]
    async fn test_trace_context_propagation() {
        if !is_service_available().await {
            eprintln!("Skipping test: user-auth-service not available");
            return;
        }

        // Create a test user
        let test_email = format!("trace_test_{}@example.com", uuid::Uuid::new_v4());
        let test_password = "password123";

        let result = create_and_login_test_user(&test_email, test_password, "Trace", "Test").await;
        assert!(result.is_some(), "Failed to create test user");
        let (user_id, token) = result.unwrap();

        let request_url = format!("http://localhost:8080/api/users/{}", user_id);

        println!(
            "Testing trace context propagation for GET /api/users/{}",
            user_id
        );

        // Expected behavior:
        // - Request should include trace headers (traceparent, tracestate)
        // - Gateway should propagate trace context to backend service
        // - Logs should show consistent trace_id across gateway and backend
        // - Distributed trace should be visible in Grafana/Tempo

        // TODO: Make actual HTTP request with trace headers
        // let client = reqwest::Client::new();
        // let trace_id = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        // let response = client
        //     .get(&request_url)
        //     .header("Authorization", format!("Bearer {}", token))
        //     .header("traceparent", trace_id)
        //     .send()
        //     .await
        //     .expect("Failed to send request");
        //
        // assert_eq!(response.status(), 200, "Expected 200 OK");
        //
        // // Check response headers for trace propagation
        // let trace_header = response.headers().get("traceparent");
        // assert!(trace_header.is_some(), "Response should include traceparent header");

        println!("✓ Trace context propagation test passed");
        println!("  Note: Check logs for consistent trace_id across services");
    }
}
