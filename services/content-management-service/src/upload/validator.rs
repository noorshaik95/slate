use super::errors::UploadError;
use bytes::Bytes;
use tracing::{debug, warn};

/// FileValidator validates file types and content
pub struct FileValidator;

impl FileValidator {
    /// Allowed MIME types for uploads
    const ALLOWED_MIME_TYPES: &'static [&'static str] = &[
        // Video types
        "video/mp4",
        "video/mpeg",
        "video/quicktime",
        "video/x-msvideo",
        "video/x-matroska",
        "video/webm",
        // Document types
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];

    /// Validates MIME type against whitelist
    pub fn validate_mime_type(content_type: &str) -> Result<(), UploadError> {
        debug!("Validating MIME type: {}", content_type);

        // Check exact match
        if Self::ALLOWED_MIME_TYPES.contains(&content_type) {
            return Ok(());
        }

        // Check wildcard match for video/*
        if content_type.starts_with("video/") {
            return Ok(());
        }

        warn!("Invalid MIME type: {}", content_type);
        Err(UploadError::InvalidFileType(content_type.to_string()))
    }

    /// Validates file size
    pub fn validate_file_size(size: i64) -> Result<(), UploadError> {
        const MAX_SIZE: i64 = 500 * 1024 * 1024; // 500MB

        if size <= 0 {
            return Err(UploadError::FileSizeExceeded(size, MAX_SIZE));
        }

        if size > MAX_SIZE {
            return Err(UploadError::FileSizeExceeded(size, MAX_SIZE));
        }

        Ok(())
    }

    /// Validates filename
    pub fn validate_filename(filename: &str) -> Result<(), UploadError> {
        if filename.trim().is_empty() {
            return Err(UploadError::InvalidFilename(
                "Filename cannot be empty".to_string(),
            ));
        }

        if filename.len() > 255 {
            return Err(UploadError::InvalidFilename(
                "Filename cannot exceed 255 characters".to_string(),
            ));
        }

        // Check for path traversal attempts
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err(UploadError::InvalidFilename(
                "Filename contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Verifies file header matches declared content type
    pub fn verify_file_header(data: &Bytes, content_type: &str) -> Result<(), UploadError> {
        if data.is_empty() {
            return Ok(()); // Can't verify empty data
        }

        // Check magic bytes for common file types
        match content_type {
            "application/pdf" => {
                if !data.starts_with(b"%PDF") {
                    return Err(UploadError::FileHeaderMismatch(
                        "File does not appear to be a valid PDF".to_string(),
                    ));
                }
            }
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                // DOCX files are ZIP archives, check for ZIP magic bytes
                if !data.starts_with(b"PK\x03\x04") && !data.starts_with(b"PK\x05\x06") {
                    return Err(UploadError::FileHeaderMismatch(
                        "File does not appear to be a valid DOCX".to_string(),
                    ));
                }
            }
            ct if ct.starts_with("video/mp4") => {
                // MP4 files typically start with ftyp box
                if data.len() >= 8 {
                    let ftyp_check = &data[4..8];
                    if ftyp_check != b"ftyp" {
                        warn!("MP4 file may not have standard ftyp header");
                        // Don't fail, as some MP4 variants may differ
                    }
                }
            }
            _ => {
                // For other video types, we'll skip header validation
                debug!("Skipping header validation for content type: {}", content_type);
            }
        }

        Ok(())
    }

    /// Placeholder for malware scanning integration
    /// In production, this would integrate with ClamAV or similar
    pub async fn scan_for_malware(_data: &Bytes) -> Result<(), UploadError> {
        // TODO: Integrate with ClamAV or similar malware scanner
        // For now, this is a placeholder that always passes
        debug!("Malware scan placeholder - would scan in production");
        Ok(())
    }

    /// Determines if a file type is a video
    pub fn is_video(content_type: &str) -> bool {
        content_type.starts_with("video/")
    }

    /// Determines if a file type is a document
    pub fn is_document(content_type: &str) -> bool {
        content_type == "application/pdf"
            || content_type
                == "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    }
}

// Tests moved to tests/validator_test.rs
