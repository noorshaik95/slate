use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Module represents a top-level organizational container for grouping related lessons
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: Uuid,
    pub course_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

impl Module {
    /// Validates module name length (1-200 characters)
    pub fn validate_name(name: &str) -> Result<(), String> {
        let len = name.trim().len();
        if len == 0 {
            Err("Module name cannot be empty".to_string())
        } else if len > 200 {
            Err("Module name cannot exceed 200 characters".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates display order is non-negative
    pub fn validate_display_order(order: i32) -> Result<(), String> {
        if order < 0 {
            Err("Display order must be non-negative".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates all module fields
    pub fn validate(&self) -> Result<(), String> {
        Self::validate_name(&self.name)?;
        Self::validate_display_order(self.display_order)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_valid() {
        assert!(Module::validate_name("Introduction to Rust").is_ok());
        assert!(Module::validate_name("A").is_ok());
        assert!(Module::validate_name(&"a".repeat(200)).is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(Module::validate_name("").is_err());
        assert!(Module::validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        assert!(Module::validate_name(&"a".repeat(201)).is_err());
    }

    #[test]
    fn test_validate_display_order() {
        assert!(Module::validate_display_order(0).is_ok());
        assert!(Module::validate_display_order(100).is_ok());
        assert!(Module::validate_display_order(-1).is_err());
    }
}
