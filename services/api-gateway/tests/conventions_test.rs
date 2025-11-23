use api_gateway::discovery::{
    constants::{CONVENTION_PATTERNS, DEFAULT_API_PREFIX},
    conventions::ConventionMapper,
};

#[test]
fn test_map_get_method() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "GetUser", "user.UserService/GetUser");

    assert!(result.is_some());
    let mapping = result.unwrap();
    assert_eq!(mapping.http_method, "GET");
    assert_eq!(
        mapping.http_path,
        format!("{}/users/:id", DEFAULT_API_PREFIX)
    );
    assert_eq!(mapping.grpc_method, "user.UserService/GetUser");
}

#[test]
fn test_map_list_method() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "ListUsers", "user.UserService/ListUsers");

    assert!(result.is_some());
    let mapping = result.unwrap();
    assert_eq!(mapping.http_method, "GET");
    assert_eq!(mapping.http_path, format!("{}/users", DEFAULT_API_PREFIX));
    assert_eq!(mapping.grpc_method, "user.UserService/ListUsers");
}

#[test]
fn test_map_create_method() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "CreateUser", "user.UserService/CreateUser");

    assert!(result.is_some());
    let mapping = result.unwrap();
    assert_eq!(mapping.http_method, "POST");
    assert_eq!(mapping.http_path, format!("{}/users", DEFAULT_API_PREFIX));
    assert_eq!(mapping.grpc_method, "user.UserService/CreateUser");
}

#[test]
fn test_map_update_method() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "UpdateUser", "user.UserService/UpdateUser");

    assert!(result.is_some());
    let mapping = result.unwrap();
    assert_eq!(mapping.http_method, "PUT");
    assert_eq!(
        mapping.http_path,
        format!("{}/users/:id", DEFAULT_API_PREFIX)
    );
    assert_eq!(mapping.grpc_method, "user.UserService/UpdateUser");
}

#[test]
fn test_map_delete_method() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "DeleteUser", "user.UserService/DeleteUser");

    assert!(result.is_some());
    let mapping = result.unwrap();
    assert_eq!(mapping.http_method, "DELETE");
    assert_eq!(
        mapping.http_path,
        format!("{}/users/:id", DEFAULT_API_PREFIX)
    );
    assert_eq!(mapping.grpc_method, "user.UserService/DeleteUser");
}

#[test]
fn test_map_unknown_pattern() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "FetchUser", "user.UserService/FetchUser");

    assert!(result.is_none());
}

#[test]
fn test_map_empty_resource() {
    let mapper = ConventionMapper::new();
    let result = mapper.map_method("UserService", "Get", "user.UserService/Get");

    assert!(result.is_none());
}

#[test]
fn test_convention_patterns_coverage() {
    let mapper = ConventionMapper::new();

    // Verify all patterns from CONVENTION_PATTERNS are handled
    for pattern in CONVENTION_PATTERNS {
        let method_name = format!("{}User", pattern);
        let full_method = format!("user.UserService/{}", method_name);
        let result = mapper.map_method("UserService", &method_name, &full_method);

        assert!(
            result.is_some(),
            "Pattern '{}' should be recognized",
            pattern
        );
    }
}

#[test]
fn test_resource_extraction_with_different_names() {
    let mapper = ConventionMapper::new();

    // Test with different resource names
    let result = mapper.map_method(
        "ProductService",
        "GetProduct",
        "product.ProductService/GetProduct",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/products/:id", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method(
        "OrderService",
        "ListOrders",
        "order.OrderService/ListOrders",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/orders", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_case_insensitive_resource() {
    let mapper = ConventionMapper::new();

    // Resource names should be converted to lowercase
    let result = mapper.map_method("UserService", "GetUser", "user.UserService/GetUser");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/users/:id", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_pluralization_simple() {
    let mapper = ConventionMapper::new();

    // Test simple pluralization through the full mapping
    let result = mapper.map_method("ItemService", "GetItem", "item.ItemService/GetItem");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/items/:id", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method("PostService", "CreatePost", "post.PostService/CreatePost");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/posts", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_pluralization_y_ending() {
    let mapper = ConventionMapper::new();

    // Test y -> ies conversion
    let result = mapper.map_method(
        "CategoryService",
        "GetCategory",
        "category.CategoryService/GetCategory",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/categories/:id", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method(
        "CompanyService",
        "ListCompanies",
        "company.CompanyService/ListCompanies",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/companies", DEFAULT_API_PREFIX)
    );

    // Vowel before y - just add s
    let result = mapper.map_method("KeyService", "GetKey", "key.KeyService/GetKey");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/keys/:id", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_pluralization_special_endings() {
    let mapper = ConventionMapper::new();

    // Test special endings
    let result = mapper.map_method("BoxService", "GetBox", "box.BoxService/GetBox");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/boxes/:id", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method(
        "ClassService",
        "CreateClass",
        "class.ClassService/CreateClass",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/classes", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method("DishService", "ListDishes", "dish.DishService/ListDishes");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/dishes", DEFAULT_API_PREFIX)
    );

    let result = mapper.map_method(
        "ChurchService",
        "GetChurch",
        "church.ChurchService/GetChurch",
    );
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/churches/:id", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_pluralization_already_plural() {
    let mapper = ConventionMapper::new();

    // ListUsers already has plural form
    let result = mapper.map_method("UserService", "ListUsers", "user.UserService/ListUsers");
    assert!(result.is_some());
    assert_eq!(
        result.unwrap().http_path,
        format!("{}/users", DEFAULT_API_PREFIX)
    );
}

#[test]
fn test_http_method_mapping() {
    let mapper = ConventionMapper::new();

    // Verify correct HTTP methods for each operation type
    let get_result = mapper.map_method("UserService", "GetUser", "user.UserService/GetUser");
    assert_eq!(get_result.unwrap().http_method, "GET");

    let list_result = mapper.map_method("UserService", "ListUsers", "user.UserService/ListUsers");
    assert_eq!(list_result.unwrap().http_method, "GET");

    let create_result =
        mapper.map_method("UserService", "CreateUser", "user.UserService/CreateUser");
    assert_eq!(create_result.unwrap().http_method, "POST");

    let update_result =
        mapper.map_method("UserService", "UpdateUser", "user.UserService/UpdateUser");
    assert_eq!(update_result.unwrap().http_method, "PUT");

    let delete_result =
        mapper.map_method("UserService", "DeleteUser", "user.UserService/DeleteUser");
    assert_eq!(delete_result.unwrap().http_method, "DELETE");
}

#[test]
fn test_path_patterns_with_id() {
    let mapper = ConventionMapper::new();

    // Operations that should include :id
    let get_result = mapper.map_method("UserService", "GetUser", "user.UserService/GetUser");
    assert!(get_result.unwrap().http_path.ends_with("/:id"));

    let update_result =
        mapper.map_method("UserService", "UpdateUser", "user.UserService/UpdateUser");
    assert!(update_result.unwrap().http_path.ends_with("/:id"));

    let delete_result =
        mapper.map_method("UserService", "DeleteUser", "user.UserService/DeleteUser");
    assert!(delete_result.unwrap().http_path.ends_with("/:id"));

    // Operations that should NOT include :id
    let list_result = mapper.map_method("UserService", "ListUsers", "user.UserService/ListUsers");
    assert!(!list_result.unwrap().http_path.ends_with("/:id"));

    let create_result =
        mapper.map_method("UserService", "CreateUser", "user.UserService/CreateUser");
    assert!(!create_result.unwrap().http_path.ends_with("/:id"));
}
