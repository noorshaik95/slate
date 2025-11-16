// Unit tests for configuration
// Extracted from src/config/mod.rs

use content_management_service::config::Config;
use std::time::Duration;

#[test]
fn test_default_config_is_valid() {
    let config = Config::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_invalid_server_port() {
    let mut config = Config::default();
    config.server.port = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_database_url() {
    let mut config = Config::default();
    config.database.url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_database_connections() {
    let mut config = Config::default();
    config.database.min_connections = 10;
    config.database.max_connections = 5;
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_s3_endpoint() {
    let mut config = Config::default();
    config.s3.endpoint = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_elasticsearch_url() {
    let mut config = Config::default();
    config.elasticsearch.url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_redis_url() {
    let mut config = Config::default();
    config.redis.url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_upload_config() {
    let mut config = Config::default();
    config.upload.max_file_size_bytes = 0;
    assert!(config.validate().is_err());
}

#[test]
fn test_duration_conversions() {
    let config = Config::default();
    assert_eq!(
        config.database_connection_timeout(),
        Duration::from_secs(30)
    );
    assert_eq!(config.shutdown_timeout(), Duration::from_secs(30));
    assert_eq!(config.s3_presigned_url_expiry(), Duration::from_secs(3600));
    assert_eq!(
        config.upload_session_expiry(),
        Duration::from_secs(24 * 3600)
    );
}

#[test]
fn test_empty_allowed_types() {
    let mut config = Config::default();
    config.upload.allowed_video_types = vec![];
    assert!(config.validate().is_err());

    let mut config = Config::default();
    config.upload.allowed_document_types = vec![];
    assert!(config.validate().is_err());
}

#[test]
fn test_transcoding_validation() {
    let mut config = Config::default();
    config.transcoding.worker_count = 0;
    assert!(config.validate().is_err());

    let mut config = Config::default();
    config.transcoding.bitrate_variants = vec![];
    assert!(config.validate().is_err());
}
