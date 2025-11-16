use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// ContentType represents the type of content resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum ContentType {
    #[sqlx(rename = "video")]
    Video,
    #[sqlx(rename = "pdf")]
    Pdf,
    #[sqlx(rename = "docx")]
    Docx,
}

impl ContentType {
    /// Validates if a MIME type is allowed for this content type
    pub fn validate_mime_type(&self, mime_type: &str) -> bool {
        match self {
            ContentType::Video => {
                mime_type.starts_with("video/")
            }
            ContentType::Pdf => {
                mime_type == "application/pdf"
            }
            ContentType::Docx => {
                mime_type == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
        }
    }

    /// Returns the list of allowed MIME types for this content type
    pub fn allowed_mime_types(&self) -> Vec<&'static str> {
        match self {
            ContentType::Video => vec!["video/mp4", "video/quicktime", "video/x-msvideo"],
            ContentType::Pdf => vec!["application/pdf"],
            ContentType::Docx => vec!["application/vnd.openxmlformats-officedocument.wordprocessingml.document"],
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Video => write!(f, "video"),
            ContentType::Pdf => write!(f, "pdf"),
            ContentType::Docx => write!(f, "docx"),
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "video" => Ok(ContentType::Video),
            "pdf" => Ok(ContentType::Pdf),
            "docx" => Ok(ContentType::Docx),
            _ => Err(format!("Invalid content type: {}", s)),
        }
    }
}

/// CopyrightSetting represents copyright restrictions on content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum CopyrightSetting {
    #[sqlx(rename = "unrestricted")]
    Unrestricted,
    #[sqlx(rename = "educational-use-only")]
    EducationalUseOnly,
    #[sqlx(rename = "no-download")]
    NoDownload,
}

impl std::fmt::Display for CopyrightSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CopyrightSetting::Unrestricted => write!(f, "unrestricted"),
            CopyrightSetting::EducationalUseOnly => write!(f, "educational-use-only"),
            CopyrightSetting::NoDownload => write!(f, "no-download"),
        }
    }
}

impl std::str::FromStr for CopyrightSetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unrestricted" => Ok(CopyrightSetting::Unrestricted),
            "educational-use-only" => Ok(CopyrightSetting::EducationalUseOnly),
            "no-download" => Ok(CopyrightSetting::NoDownload),
            _ => Err(format!("Invalid copyright setting: {}", s)),
        }
    }
}

/// Resource represents an individual content item (video, PDF, DOCX file)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: Uuid,
    pub lesson_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub content_type: ContentType,
    pub file_size: i64,
    pub storage_key: String,
    pub manifest_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub published: bool,
    pub downloadable: bool,
    pub copyright_setting: CopyrightSetting,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Resource {
    /// Maximum file size in bytes (500MB)
    pub const MAX_FILE_SIZE: i64 = 500 * 1024 * 1024;

    /// Validates resource name length (1-200 characters)
    pub fn validate_name(name: &str) -> Result<(), String> {
        let len = name.trim().len();
        if len == 0 {
            Err("Resource name cannot be empty".to_string())
        } else if len > 200 {
            Err("Resource name cannot exceed 200 characters".to_string())
        } else {
            Ok(())
        }
    }

    /// Validates file size does not exceed maximum
    pub fn validate_file_size(size: i64) -> Result<(), String> {
        if size <= 0 {
            Err("File size must be positive".to_string())
        } else if size > Self::MAX_FILE_SIZE {
            Err(format!(
                "File size exceeds maximum of {} bytes",
                Self::MAX_FILE_SIZE
            ))
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

    /// Validates video duration is positive
    pub fn validate_duration(duration: Option<i32>) -> Result<(), String> {
        if let Some(d) = duration {
            if d <= 0 {
                return Err("Video duration must be positive".to_string());
            }
        }
        Ok(())
    }

    /// Validates all resource fields
    pub fn validate(&self) -> Result<(), String> {
        Self::validate_name(&self.name)?;
        Self::validate_file_size(self.file_size)?;
        Self::validate_display_order(self.display_order)?;
        Self::validate_duration(self.duration_seconds)?;

        // Video-specific validations
        if self.content_type == ContentType::Video {
            if self.duration_seconds.is_none() {
                return Err("Video resources must have duration".to_string());
            }
        }

        Ok(())
    }

    /// Checks if download is allowed based on copyright settings
    pub fn is_download_allowed(&self) -> bool {
        match self.copyright_setting {
            CopyrightSetting::NoDownload => false,
            _ => self.downloadable,
        }
    }

    /// Returns copyright notice if applicable
    pub fn copyright_notice(&self) -> Option<&'static str> {
        match self.copyright_setting {
            CopyrightSetting::EducationalUseOnly => {
                Some("This content is for educational use only. Unauthorized distribution is prohibited.")
            }
            CopyrightSetting::NoDownload => {
                Some("This content cannot be downloaded due to copyright restrictions.")
            }
            CopyrightSetting::Unrestricted => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_validate_mime_type() {
        assert!(ContentType::Video.validate_mime_type("video/mp4"));
        assert!(ContentType::Video.validate_mime_type("video/quicktime"));
        assert!(!ContentType::Video.validate_mime_type("application/pdf"));

        assert!(ContentType::Pdf.validate_mime_type("application/pdf"));
        assert!(!ContentType::Pdf.validate_mime_type("video/mp4"));

        assert!(ContentType::Docx.validate_mime_type(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ));
    }

    #[test]
    fn test_content_type_from_str() {
        assert_eq!("video".parse::<ContentType>().unwrap(), ContentType::Video);
        assert_eq!("pdf".parse::<ContentType>().unwrap(), ContentType::Pdf);
        assert_eq!("docx".parse::<ContentType>().unwrap(), ContentType::Docx);
        assert!("invalid".parse::<ContentType>().is_err());
    }

    #[test]
    fn test_copyright_setting_from_str() {
        assert_eq!(
            "unrestricted".parse::<CopyrightSetting>().unwrap(),
            CopyrightSetting::Unrestricted
        );
        assert_eq!(
            "educational-use-only".parse::<CopyrightSetting>().unwrap(),
            CopyrightSetting::EducationalUseOnly
        );
        assert_eq!(
            "no-download".parse::<CopyrightSetting>().unwrap(),
            CopyrightSetting::NoDownload
        );
    }

    #[test]
    fn test_validate_name() {
        assert!(Resource::validate_name("Introduction Video").is_ok());
        assert!(Resource::validate_name("A").is_ok());
        assert!(Resource::validate_name(&"a".repeat(200)).is_ok());
        assert!(Resource::validate_name("").is_err());
        assert!(Resource::validate_name(&"a".repeat(201)).is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(Resource::validate_file_size(1024).is_ok());
        assert!(Resource::validate_file_size(500 * 1024 * 1024).is_ok());
        assert!(Resource::validate_file_size(0).is_err());
        assert!(Resource::validate_file_size(-1).is_err());
        assert!(Resource::validate_file_size(501 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_is_download_allowed() {
        let mut resource = Resource {
            id: Uuid::new_v4(),
            lesson_id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            content_type: ContentType::Video,
            file_size: 1024,
            storage_key: "test".to_string(),
            manifest_url: None,
            duration_seconds: Some(60),
            published: true,
            downloadable: true,
            copyright_setting: CopyrightSetting::Unrestricted,
            display_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(resource.is_download_allowed());

        resource.copyright_setting = CopyrightSetting::NoDownload;
        assert!(!resource.is_download_allowed());

        resource.copyright_setting = CopyrightSetting::EducationalUseOnly;
        assert!(resource.is_download_allowed());

        resource.downloadable = false;
        assert!(!resource.is_download_allowed());
    }

    #[test]
    fn test_copyright_notice() {
        let mut resource = Resource {
            id: Uuid::new_v4(),
            lesson_id: Uuid::new_v4(),
            name: "Test".to_string(),
            description: None,
            content_type: ContentType::Video,
            file_size: 1024,
            storage_key: "test".to_string(),
            manifest_url: None,
            duration_seconds: Some(60),
            published: true,
            downloadable: true,
            copyright_setting: CopyrightSetting::Unrestricted,
            display_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(resource.copyright_notice().is_none());

        resource.copyright_setting = CopyrightSetting::EducationalUseOnly;
        assert!(resource.copyright_notice().is_some());

        resource.copyright_setting = CopyrightSetting::NoDownload;
        assert!(resource.copyright_notice().is_some());
    }
}
