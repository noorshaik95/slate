use tracing::{error, info, warn};
use uuid::Uuid;

/// Log service startup with configuration details
pub fn log_service_startup(
    service_name: &str,
    version: &str,
    grpc_port: u16,
    metrics_port: u16,
) {
    info!(
        service_name = service_name,
        version = version,
        grpc_port = grpc_port,
        metrics_port = metrics_port,
        "Content Management Service starting"
    );
}

/// Log configuration loaded successfully
pub fn log_configuration_loaded(
    database_url: &str,
    s3_endpoint: &str,
    elasticsearch_url: &str,
    redis_url: &str,
) {
    info!(
        database_url = mask_password(database_url),
        s3_endpoint = s3_endpoint,
        elasticsearch_url = elasticsearch_url,
        redis_url = redis_url,
        "Configuration loaded successfully"
    );
}

/// Log upload completion
pub fn log_upload_complete(
    user_id: &Uuid,
    resource_id: &Uuid,
    file_size: i64,
    content_type: &str,
    duration_ms: u64,
) {
    info!(
        user_id = %user_id,
        resource_id = %resource_id,
        file_size = file_size,
        content_type = content_type,
        duration_ms = duration_ms,
        operation = "upload_complete",
        "File upload completed successfully"
    );
}

/// Log transcoding completion
pub fn log_transcoding_complete(
    resource_id: &Uuid,
    formats: &[String],
    duration_ms: u64,
) {
    info!(
        resource_id = %resource_id,
        formats = ?formats,
        duration_ms = duration_ms,
        operation = "transcoding_complete",
        "Video transcoding completed successfully"
    );
}

/// Log transcoding failure
pub fn log_transcoding_failure(
    resource_id: &Uuid,
    error: &str,
    retry_count: i32,
) {
    error!(
        resource_id = %resource_id,
        error = error,
        retry_count = retry_count,
        operation = "transcoding_failed",
        "Video transcoding failed"
    );
}

/// Log search query
pub fn log_search_query(
    user_id: &Uuid,
    query: &str,
    result_count: usize,
    duration_ms: u64,
) {
    info!(
        user_id = %user_id,
        query = query,
        result_count = result_count,
        duration_ms = duration_ms,
        operation = "search_query",
        "Search query executed"
    );
}

/// Log progress calculation
pub fn log_progress_calculation(
    user_id: &Uuid,
    course_id: &Uuid,
    progress_percentage: i32,
    duration_ms: u64,
) {
    info!(
        user_id = %user_id,
        course_id = %course_id,
        progress_percentage = progress_percentage,
        duration_ms = duration_ms,
        operation = "progress_calculation",
        "Progress calculated"
    );
}

/// Log download URL generation
pub fn log_download_url_generated(
    user_id: &Uuid,
    resource_id: &Uuid,
    content_type: &str,
) {
    info!(
        user_id = %user_id,
        resource_id = %resource_id,
        content_type = content_type,
        operation = "download_url_generated",
        "Download URL generated"
    );
}

/// Log database error
pub fn log_database_error(
    operation: &str,
    error: &str,
) {
    error!(
        operation = operation,
        error = error,
        error_type = "database",
        "Database operation failed"
    );
}

/// Log S3 error
pub fn log_s3_error(
    operation: &str,
    error: &str,
) {
    error!(
        operation = operation,
        error = error,
        error_type = "s3",
        "S3 operation failed"
    );
}

/// Log ElasticSearch error
pub fn log_elasticsearch_error(
    operation: &str,
    error: &str,
) {
    error!(
        operation = operation,
        error = error,
        error_type = "elasticsearch",
        "ElasticSearch operation failed"
    );
}

/// Log circuit breaker state change
pub fn log_circuit_breaker_state_change(
    service: &str,
    old_state: &str,
    new_state: &str,
) {
    warn!(
        service = service,
        old_state = old_state,
        new_state = new_state,
        "Circuit breaker state changed"
    );
}

/// Log retry attempt
pub fn log_retry_attempt(
    operation: &str,
    attempt: i32,
    max_attempts: i32,
) {
    warn!(
        operation = operation,
        attempt = attempt,
        max_attempts = max_attempts,
        "Retrying operation"
    );
}

/// Log graceful shutdown
pub fn log_graceful_shutdown() {
    info!("Initiating graceful shutdown");
}

/// Log shutdown complete
pub fn log_shutdown_complete() {
    info!("Shutdown complete");
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://user:****@localhost:5432/db");
    }

    #[test]
    fn test_mask_password_no_password() {
        let url = "postgresql://localhost:5432/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgresql://localhost:5432/db");
    }
}
