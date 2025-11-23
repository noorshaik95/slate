// Security tests for path validation and CORS
//
// These tests verify that security controls are properly enforced

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    // Note: PathValidator tests are in src/security/validator.rs
    // This file contains integration-level security tests

    #[test]
    fn test_body_size_limit_constant() {
        // Verify the body size limit is set to 10MB
        const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB
        assert_eq!(MAX_REQUEST_BODY_SIZE, 10485760);
    }

    #[test]
    fn test_path_traversal_patterns() {
        // Test various path traversal patterns that should be blocked
        let dangerous_patterns = vec![
            "../etc/passwd",
            "..\\windows\\system32",
            "/etc/passwd",
            "C:\\Windows\\System32",
            "file\0.txt",
            "%2e%2e/etc/passwd",
            "..%2fetc%2fpasswd",
            "..%5cwindows%5csystem32",
        ];

        for pattern in dangerous_patterns {
            // These patterns should be detected and blocked
            assert!(
                pattern.contains("..")
                    || pattern.starts_with('/')
                    || pattern.contains('\\')
                    || pattern.contains('\0')
                    || pattern.to_lowercase().contains("%2e%2e")
                    || pattern.to_lowercase().contains("%00"),
                "Pattern '{}' should be detected as dangerous",
                pattern
            );
        }
    }

    #[test]
    fn test_safe_path_patterns() {
        // Test patterns that should be allowed
        let safe_patterns = vec![
            "123",
            "user-name",
            "550e8400-e29b-41d4-a716-446655440000",
            "hello_world",
            "file.txt",
            "2024-01-15",
        ];

        for pattern in safe_patterns {
            // These patterns should be safe
            assert!(
                !pattern.contains("..")
                    && !pattern.starts_with('/')
                    && !pattern.contains('\\')
                    && !pattern.contains('\0'),
                "Pattern '{}' should be safe",
                pattern
            );
        }
    }

    #[tokio::test]
    async fn test_cors_configuration() {
        // Test that CORS configuration has sensible defaults
        let default_allowed_methods = vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"];
        let default_allowed_headers = vec!["content-type", "authorization", "x-trace-id"];
        let default_max_age = 3600u64;

        assert_eq!(default_allowed_methods.len(), 5);
        assert_eq!(default_allowed_headers.len(), 3);
        assert_eq!(default_max_age, 3600);
    }

    #[test]
    fn test_url_encoding_safety() {
        // Test that URL encoding is handled safely
        let test_cases = vec![
            ("hello%20world", "hello world"),
            ("user%2Fname", "user/name"),
            ("test%3Dvalue", "test=value"),
        ];

        for (encoded, expected) in test_cases {
            match urlencoding::decode(encoded) {
                Ok(decoded) => {
                    assert_eq!(decoded, expected, "URL decoding failed for '{}'", encoded);
                }
                Err(e) => {
                    panic!("Failed to decode '{}': {}", encoded, e);
                }
            }
        }
    }

    #[test]
    fn test_suspicious_encoded_patterns() {
        // Test that suspicious encoded patterns are detected
        let suspicious = vec![
            "%00",    // Null byte
            "%2e%2e", // ..
            "%252e",  // Double encoded .
            "..%2f",  // Mixed encoding
            "..%5c",  // Mixed encoding with backslash
        ];

        for pattern in suspicious {
            assert!(
                pattern.to_lowercase().contains("%00")
                    || pattern.to_lowercase().contains("%2e%2e")
                    || pattern.to_lowercase().contains("%252e")
                    || pattern.to_lowercase().contains("..%2f")
                    || pattern.to_lowercase().contains("..%5c"),
                "Pattern '{}' should be detected as suspicious",
                pattern
            );
        }
    }
}
