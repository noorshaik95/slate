use api_gateway::discovery::MethodDescriptor;

#[test]
fn test_method_descriptor_creation() {
    let descriptor = MethodDescriptor {
        name: "GetUser".to_string(),
        full_name: "user.UserService/GetUser".to_string(),
        input_type: ".user.GetUserRequest".to_string(),
        output_type: ".user.GetUserResponse".to_string(),
    };

    assert_eq!(descriptor.name, "GetUser");
    assert_eq!(descriptor.full_name, "user.UserService/GetUser");
    assert!(!descriptor.input_type.is_empty());
    assert!(!descriptor.output_type.is_empty());
}

#[test]
fn test_method_descriptor_with_empty_types() {
    let descriptor = MethodDescriptor {
        name: "TestMethod".to_string(),
        full_name: "test.Service/TestMethod".to_string(),
        input_type: String::new(),
        output_type: String::new(),
    };

    assert_eq!(descriptor.name, "TestMethod");
    assert!(descriptor.input_type.is_empty());
    assert!(descriptor.output_type.is_empty());
}
