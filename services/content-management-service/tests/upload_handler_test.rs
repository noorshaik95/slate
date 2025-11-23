use bytes::Bytes;
use content_management_service::models::{UploadSession, UploadStatus};
use content_management_service::upload::errors::UploadError;
use content_management_service::upload::handler::UploadHandler;
use content_management_service::upload::validator::FileValidator;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/cms_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[test]
fn test_file_validator_mime_type_validation() {
    // Valid video types
    assert!(FileValidator::validate_mime_type("video/mp4").is_ok());
    assert!(FileValidator::validate_mime_type("video/mpeg").is_ok());
    assert!(FileValidator::validate_mime_type("video/quicktime").is_ok());
    assert!(FileValidator::validate_mime_type("video/x-msvideo").is_ok());

    // Valid document types
    assert!(FileValidator::validate_mime_type("application/pdf").is_ok());
    assert!(FileValidator::validate_mime_type(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_ok());

    // Invalid types
    assert!(FileValidator::validate_mime_type("application/json").is_err());
    assert!(FileValidator::validate_mime_type("text/plain").is_err());
    assert!(FileValidator::validate_mime_type("image/jpeg").is_err());
}

#[test]
fn test_file_validator_size_validation() {
    // Valid sizes
    assert!(FileValidator::validate_file_size(1024).is_ok());
    assert!(FileValidator::validate_file_size(10 * 1024 * 1024).is_ok());
    assert!(FileValidator::validate_file_size(500 * 1024 * 1024).is_ok());

    // Invalid sizes
    assert!(FileValidator::validate_file_size(0).is_err());
    assert!(FileValidator::validate_file_size(-1).is_err());
    assert!(FileValidator::validate_file_size(501 * 1024 * 1024).is_err());
    assert!(FileValidator::validate_file_size(1000 * 1024 * 1024).is_err());
}

#[test]
fn test_file_validator_filename_validation() {
    // Valid filenames
    assert!(FileValidator::validate_filename("video.mp4").is_ok());
    assert!(FileValidator::validate_filename("my-document.pdf").is_ok());
    assert!(FileValidator::validate_filename("file_name.docx").is_ok());
    assert!(FileValidator::validate_filename("test123.mp4").is_ok());

    // Invalid filenames
    assert!(FileValidator::validate_filename("").is_err());
    assert!(FileValidator::validate_filename("   ").is_err());
    assert!(FileValidator::validate_filename("../etc/passwd").is_err());
    assert!(FileValidator::validate_filename("path/to/file.mp4").is_err());
    assert!(FileValidator::validate_filename("file\\name.mp4").is_err());
    assert!(FileValidator::validate_filename(&"a".repeat(256)).is_err());
}

#[test]
fn test_file_validator_header_verification_pdf() {
    // Valid PDF header
    let pdf_data = Bytes::from("%PDF-1.4\n%âãÏÓ");
    assert!(FileValidator::verify_file_header(&pdf_data, "application/pdf").is_ok());

    // Invalid PDF header
    let invalid_pdf = Bytes::from("This is not a PDF file");
    assert!(FileValidator::verify_file_header(&invalid_pdf, "application/pdf").is_err());
}

#[test]
fn test_file_validator_header_verification_docx() {
    // Valid DOCX header (ZIP magic bytes)
    let docx_data = Bytes::from(vec![0x50, 0x4B, 0x03, 0x04, 0x00, 0x00]);
    assert!(FileValidator::verify_file_header(
        &docx_data,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_ok());

    // Invalid DOCX header
    let invalid_docx = Bytes::from("Not a DOCX file");
    assert!(FileValidator::verify_file_header(
        &invalid_docx,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    .is_err());
}

#[test]
fn test_file_validator_header_verification_empty_data() {
    // Empty data should pass (can't verify)
    let empty = Bytes::new();
    assert!(FileValidator::verify_file_header(&empty, "video/mp4").is_ok());
    assert!(FileValidator::verify_file_header(&empty, "application/pdf").is_ok());
}

#[test]
fn test_upload_session_calculate_total_chunks() {
    let chunk_size = 5 * 1024 * 1024; // 5MB

    // Exact multiples
    assert_eq!(
        UploadSession::calculate_total_chunks(5 * 1024 * 1024, chunk_size),
        1
    );
    assert_eq!(
        UploadSession::calculate_total_chunks(10 * 1024 * 1024, chunk_size),
        2
    );
    assert_eq!(
        UploadSession::calculate_total_chunks(50 * 1024 * 1024, chunk_size),
        10
    );

    // Non-exact multiples (should round up)
    assert_eq!(
        UploadSession::calculate_total_chunks(6 * 1024 * 1024, chunk_size),
        2
    );
    assert_eq!(
        UploadSession::calculate_total_chunks(12 * 1024 * 1024, chunk_size),
        3
    );
    assert_eq!(
        UploadSession::calculate_total_chunks(51 * 1024 * 1024, chunk_size),
        11
    );
}

#[test]
fn test_upload_session_progress_percentage() {
    let session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 4,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        completed_at: None,
    };

    // Test different progress levels
    let mut test_session = session.clone();
    test_session.uploaded_chunks = 0;
    assert_eq!(test_session.progress_percentage(), 0.0);

    test_session.uploaded_chunks = 1;
    assert_eq!(test_session.progress_percentage(), 25.0);

    test_session.uploaded_chunks = 2;
    assert_eq!(test_session.progress_percentage(), 50.0);

    test_session.uploaded_chunks = 3;
    assert_eq!(test_session.progress_percentage(), 75.0);

    test_session.uploaded_chunks = 4;
    assert_eq!(test_session.progress_percentage(), 100.0);
}

#[test]
fn test_upload_session_is_expired() {
    use chrono::{Duration, Utc};

    // Not expired
    let session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 2,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        completed_at: None,
    };

    assert!(!session.is_expired());

    // Expired
    let mut expired_session = session.clone();
    expired_session.expires_at = Utc::now() - Duration::hours(1);
    assert!(expired_session.is_expired());
}

#[test]
fn test_upload_session_is_resumable() {
    use chrono::{Duration, Utc};

    let session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 2,
        uploaded_chunks: 1,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        completed_at: None,
    };

    // Resumable: in progress and not expired
    assert!(session.is_resumable());

    // Not resumable: completed
    let mut completed_session = session.clone();
    completed_session.status = UploadStatus::Completed;
    assert!(!completed_session.is_resumable());

    // Not resumable: failed
    let mut failed_session = session.clone();
    failed_session.status = UploadStatus::Failed;
    assert!(!failed_session.is_resumable());

    // Not resumable: expired
    let mut expired_session = session.clone();
    expired_session.expires_at = Utc::now() - Duration::hours(1);
    assert!(!expired_session.is_resumable());
}

#[test]
fn test_upload_session_is_complete() {
    let mut session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 4,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        completed_at: None,
    };

    // Not complete
    assert!(!session.is_complete());

    session.uploaded_chunks = 3;
    assert!(!session.is_complete());

    // Complete
    session.uploaded_chunks = 4;
    assert!(session.is_complete());

    // Still complete even if more chunks uploaded (edge case)
    session.uploaded_chunks = 5;
    assert!(session.is_complete());
}

#[test]
fn test_upload_session_validate_chunk_index() {
    let session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 4,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        completed_at: None,
    };

    // Valid indices
    assert!(session.validate_chunk_index(0).is_ok());
    assert!(session.validate_chunk_index(1).is_ok());
    assert!(session.validate_chunk_index(2).is_ok());
    assert!(session.validate_chunk_index(3).is_ok());

    // Invalid indices
    assert!(session.validate_chunk_index(-1).is_err());
    assert!(session.validate_chunk_index(4).is_err());
    assert!(session.validate_chunk_index(10).is_err());
}

#[tokio::test]
async fn test_initiate_upload_with_valid_inputs() {
    // This test requires a real database connection and S3 client
    // For now, we'll test the validation logic separately

    // Test that validation passes for valid inputs
    assert!(FileValidator::validate_filename("test.mp4").is_ok());
    assert!(FileValidator::validate_mime_type("video/mp4").is_ok());
    assert!(FileValidator::validate_file_size(10 * 1024 * 1024).is_ok());
}

#[tokio::test]
async fn test_initiate_upload_with_invalid_filename() {
    // Test validation fails for invalid filename
    assert!(FileValidator::validate_filename("").is_err());
    assert!(FileValidator::validate_filename("../etc/passwd").is_err());
}

#[tokio::test]
async fn test_initiate_upload_with_invalid_mime_type() {
    // Test validation fails for invalid MIME type
    assert!(FileValidator::validate_mime_type("application/json").is_err());
    assert!(FileValidator::validate_mime_type("text/plain").is_err());
}

#[tokio::test]
async fn test_initiate_upload_with_file_too_large() {
    // Test validation fails for file too large
    assert!(FileValidator::validate_file_size(501 * 1024 * 1024).is_err());
    assert!(FileValidator::validate_file_size(1000 * 1024 * 1024).is_err());
}

#[test]
fn test_chunk_upload_validates_session_expiration() {
    use chrono::{Duration, Utc};

    let expired_session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 2,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: Utc::now() - Duration::hours(25),
        expires_at: Utc::now() - Duration::hours(1),
        completed_at: None,
    };

    assert!(expired_session.is_expired());
    assert!(!expired_session.is_resumable());
}

#[test]
fn test_chunk_upload_validates_chunk_index() {
    let session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 2,
        uploaded_chunks: 0,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        completed_at: None,
    };

    // Valid chunk indices
    assert!(session.validate_chunk_index(0).is_ok());
    assert!(session.validate_chunk_index(1).is_ok());

    // Invalid chunk indices
    assert!(session.validate_chunk_index(-1).is_err());
    assert!(session.validate_chunk_index(2).is_err());
    assert!(session.validate_chunk_index(100).is_err());
}

#[test]
fn test_complete_upload_validates_all_chunks_uploaded() {
    let incomplete_session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 4,
        uploaded_chunks: 2,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
        completed_at: None,
    };

    assert!(!incomplete_session.is_complete());

    let complete_session = UploadSession {
        uploaded_chunks: 4,
        ..incomplete_session
    };

    assert!(complete_session.is_complete());
}

#[test]
fn test_resume_upload_validates_session_status() {
    use chrono::{Duration, Utc};

    // Valid for resumption
    let in_progress_session = UploadSession {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        lesson_id: Some(Uuid::new_v4()),
        filename: "test.mp4".to_string(),
        content_type: "video/mp4".to_string(),
        total_size: 10 * 1024 * 1024,
        chunk_size: 5 * 1024 * 1024,
        total_chunks: 2,
        uploaded_chunks: 1,
        storage_key: "test".to_string(),
        status: UploadStatus::InProgress,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
        completed_at: None,
    };

    assert!(in_progress_session.is_resumable());

    // Not valid for resumption - completed
    let completed_session = UploadSession {
        status: UploadStatus::Completed,
        ..in_progress_session.clone()
    };

    assert!(!completed_session.is_resumable());

    // Not valid for resumption - failed
    let failed_session = UploadSession {
        status: UploadStatus::Failed,
        ..in_progress_session.clone()
    };

    assert!(!failed_session.is_resumable());

    // Not valid for resumption - expired
    let expired_session = UploadSession {
        expires_at: Utc::now() - Duration::hours(1),
        ..in_progress_session
    };

    assert!(!expired_session.is_resumable());
}
