use std::collections::HashMap;
use tracing::{debug, warn};

/// Security validator for path parameters and request validation
pub struct PathValidator;

/// Security-related errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    
    #[error("Invalid path parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Suspicious pattern detected: {0}")]
    SuspiciousPattern(String),
}

impl PathValidator {
    /// Validate and sanitize path parameters to prevent directory traversal attacks
    ///
    /// This function checks for:
    /// - Directory traversal patterns (../, ..\)
    /// - Absolute paths (/, \, C:\)
    /// - Null bytes
    /// - Suspicious patterns
    pub fn sanitize_path_params(
        params: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, SecurityError> {
        let mut sanitized = HashMap::new();

        for (key, value) in params {
            debug!(key = %key, value = %value, "Validating path parameter");

            // URL decode the value first
            let decoded = match urlencoding::decode(value) {
                Ok(decoded) => decoded.to_string(),
                Err(e) => {
                    warn!(key = %key, value = %value, error = %e, "Failed to URL decode parameter");
                    return Err(SecurityError::InvalidParameter(format!(
                        "Failed to decode parameter '{}': {}",
                        key, e
                    )));
                }
            };

            // Check for directory traversal patterns
            if decoded.contains("..") {
                warn!(key = %key, value = %decoded, "Directory traversal attempt detected");
                return Err(SecurityError::PathTraversal(format!(
                    "Parameter '{}' contains '..' pattern",
                    key
                )));
            }

            // Check for absolute paths (Unix)
            if decoded.starts_with('/') {
                warn!(key = %key, value = %decoded, "Absolute path detected");
                return Err(SecurityError::PathTraversal(format!(
                    "Parameter '{}' contains absolute path",
                    key
                )));
            }

            // Check for absolute paths (Windows)
            if decoded.len() >= 2 && decoded.chars().nth(1) == Some(':') {
                warn!(key = %key, value = %decoded, "Windows absolute path detected");
                return Err(SecurityError::PathTraversal(format!(
                    "Parameter '{}' contains Windows absolute path",
                    key
                )));
            }

            // Check for backslashes (Windows path separators)
            if decoded.contains('\\') {
                warn!(key = %key, value = %decoded, "Backslash detected in path");
                return Err(SecurityError::PathTraversal(format!(
                    "Parameter '{}' contains backslash",
                    key
                )));
            }

            // Check for null bytes
            if decoded.contains('\0') {
                warn!(key = %key, value = %decoded, "Null byte detected");
                return Err(SecurityError::SuspiciousPattern(format!(
                    "Parameter '{}' contains null byte",
                    key
                )));
            }

            // Check for suspicious patterns
            let suspicious_patterns = [
                "%00", // Null byte encoded
                "%2e%2e", // .. encoded
                "%252e", // Double encoded .
                "..%2f", // Mixed encoding
                "..%5c", // Mixed encoding with backslash
            ];

            for pattern in &suspicious_patterns {
                if value.to_lowercase().contains(pattern) {
                    warn!(key = %key, value = %value, pattern = %pattern, "Suspicious pattern detected");
                    return Err(SecurityError::SuspiciousPattern(format!(
                        "Parameter '{}' contains suspicious pattern: {}",
                        key, pattern
                    )));
                }
            }

            debug!(key = %key, value = %decoded, "Path parameter validated successfully");
            sanitized.insert(key.clone(), decoded);
        }

        Ok(sanitized)
    }

    /// Validate a single path parameter
    #[allow(dead_code)]
    pub fn validate_param(key: &str, value: &str) -> Result<String, SecurityError> {
        let mut params = HashMap::new();
        params.insert(key.to_string(), value.to_string());
        
        let sanitized = Self::sanitize_path_params(&params)?;
        
        Ok(sanitized.get(key).unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_parameters() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("name".to_string(), "user-name".to_string());
        params.insert("uuid".to_string(), "550e8400-e29b-41d4-a716-446655440000".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_ok());
        
        let sanitized = result.unwrap();
        assert_eq!(sanitized.get("id").unwrap(), "123");
        assert_eq!(sanitized.get("name").unwrap(), "user-name");
    }

    #[test]
    fn test_directory_traversal_double_dot() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "../etc/passwd".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal(_)));
    }

    #[test]
    fn test_directory_traversal_encoded() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "%2e%2e/etc/passwd".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        // After URL decoding, %2e%2e becomes .. which is caught as PathTraversal
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal(_)));
    }

    #[test]
    fn test_absolute_path_unix() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "/etc/passwd".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal(_)));
    }

    #[test]
    fn test_absolute_path_windows() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "C:\\Windows\\System32".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal(_)));
    }

    #[test]
    fn test_backslash_in_path() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "..\\etc\\passwd".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::PathTraversal(_)));
    }

    #[test]
    fn test_null_byte() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "file\0.txt".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::SuspiciousPattern(_)));
    }

    #[test]
    fn test_url_encoded_valid() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), "hello%20world".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_ok());
        
        let sanitized = result.unwrap();
        assert_eq!(sanitized.get("name").unwrap(), "hello world");
    }

    #[test]
    fn test_suspicious_pattern_null_byte_encoded() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), "file%00.txt".to_string());

        let result = PathValidator::sanitize_path_params(&params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SecurityError::SuspiciousPattern(_)));
    }

    #[test]
    fn test_validate_single_param() {
        let result = PathValidator::validate_param("id", "123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "123");

        let result = PathValidator::validate_param("path", "../etc/passwd");
        assert!(result.is_err());
    }
}
