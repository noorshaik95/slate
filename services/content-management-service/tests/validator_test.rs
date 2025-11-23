// Unit tests for file validator
// Extracted from src/upload/validator.rs

use bytes::Bytes;
use content_management_service::upload::FileValidator;

#[test]
fn test_validate_mime_type() {
    assert!(FileValidator::validate_mime_type("video/mp4").is_ok());
    assert!(FileValidator::validate_mime_type("video/mpeg").is_ok());
    assert!(FileValidator::validate_mime_type("video/custom").is_ok()); // video/* wildcard
    assert!(FileValidator::validate_mime_type("application/pdf").is_ok());
    assert!(FileValidator::validate_mime_type(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_ok());
    assert!(FileValidator::validate_mime_type("application/json").is_err());
    assert!(FileValidator::validate_mime_type("text/plain").is_err());
}

#[test]
fn test_validate_file_size() {
    assert!(FileValidator::validate_file_size(1024).is_ok());
    assert!(FileValidator::validate_file_size(500 * 1024 * 1024).is_ok());
    assert!(FileValidator::validate_file_size(0).is_err());
    assert!(FileValidator::validate_file_size(-1).is_err());
    assert!(FileValidator::validate_file_size(501 * 1024 * 1024).is_err());
}

#[test]
fn test_validate_filename() {
    assert!(FileValidator::validate_filename("test.mp4").is_ok());
    assert!(FileValidator::validate_filename("my-video.mp4").is_ok());
    assert!(FileValidator::validate_filename("").is_err());
    assert!(FileValidator::validate_filename("../etc/passwd").is_err());
    assert!(FileValidator::validate_filename("path/to/file.mp4").is_err());
    assert!(FileValidator::validate_filename(&"a".repeat(256)).is_err());
}

#[test]
fn test_verify_file_header() {
    // PDF header
    let pdf_data = Bytes::from("%PDF-1.4\n");
    assert!(FileValidator::verify_file_header(&pdf_data, "application/pdf").is_ok());

    let invalid_pdf = Bytes::from("not a pdf");
    assert!(FileValidator::verify_file_header(&invalid_pdf, "application/pdf").is_err());

    // ZIP/DOCX header
    let zip_data = Bytes::from(vec![0x50, 0x4B, 0x03, 0x04]);
    assert!(FileValidator::verify_file_header(
        &zip_data,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_ok());

    // Empty data should pass
    let empty = Bytes::new();
    assert!(FileValidator::verify_file_header(&empty, "video/mp4").is_ok());
}

#[test]
fn test_is_video() {
    assert!(FileValidator::is_video("video/mp4"));
    assert!(FileValidator::is_video("video/mpeg"));
    assert!(!FileValidator::is_video("application/pdf"));
}

#[test]
fn test_is_document() {
    assert!(FileValidator::is_document("application/pdf"));
    assert!(FileValidator::is_document(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    ));
    assert!(!FileValidator::is_document("video/mp4"));
}
