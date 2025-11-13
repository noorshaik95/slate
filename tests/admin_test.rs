use api_gateway::handlers::RefreshResponse;

#[test]
fn test_refresh_response_serialization() {
    let response = RefreshResponse {
        success: true,
        routes_discovered: 10,
        services_queried: 2,
        errors: vec![],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"routes_discovered\":10"));
    assert!(json.contains("\"services_queried\":2"));
}

#[test]
fn test_refresh_response_with_errors() {
    let response = RefreshResponse {
        success: false,
        routes_discovered: 0,
        services_queried: 3,
        errors: vec![
            "Service user-service unreachable".to_string(),
            "Service auth-service does not support reflection".to_string(),
        ],
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"success\":false"));
    assert!(json.contains("\"errors\""));
}
