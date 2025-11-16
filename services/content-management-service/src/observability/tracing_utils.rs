use tracing::Span;
use uuid::Uuid;

/// Add common attributes to a span for upload operations
pub fn add_upload_span_attributes(
    span: &Span,
    user_id: &Uuid,
    file_size: i64,
    content_type: &str,
) {
    span.record("user_id", user_id.to_string().as_str());
    span.record("file_size", file_size);
    span.record("content_type", content_type);
}

/// Add resource attributes to a span
pub fn add_resource_span_attributes(
    span: &Span,
    resource_id: &Uuid,
    user_id: &Uuid,
) {
    span.record("resource_id", resource_id.to_string().as_str());
    span.record("user_id", user_id.to_string().as_str());
}

/// Add operation status to a span
pub fn add_operation_status(span: &Span, status: &str) {
    span.record("operation_status", status);
}

/// Add error information to a span
pub fn add_error_to_span(span: &Span, error: &str) {
    span.record("error", error);
    span.record("operation_status", "failure");
}

/// Add search query attributes to a span
pub fn add_search_span_attributes(
    span: &Span,
    query: &str,
    user_id: &Uuid,
    result_count: usize,
) {
    span.record("query", query);
    span.record("user_id", user_id.to_string().as_str());
    span.record("result_count", result_count as i64);
}

/// Add transcoding job attributes to a span
pub fn add_transcoding_span_attributes(
    span: &Span,
    resource_id: &Uuid,
    format: &str,
) {
    span.record("resource_id", resource_id.to_string().as_str());
    span.record("format", format);
}

/// Add progress calculation attributes to a span
pub fn add_progress_span_attributes(
    span: &Span,
    user_id: &Uuid,
    course_id: &Uuid,
    progress_percentage: i32,
) {
    span.record("user_id", user_id.to_string().as_str());
    span.record("course_id", course_id.to_string().as_str());
    span.record("progress_percentage", progress_percentage as i64);
}

/// Add download attributes to a span
pub fn add_download_span_attributes(
    span: &Span,
    resource_id: &Uuid,
    user_id: &Uuid,
    content_type: &str,
) {
    span.record("resource_id", resource_id.to_string().as_str());
    span.record("user_id", user_id.to_string().as_str());
    span.record("content_type", content_type);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::{info_span, field};

    #[test]
    fn test_add_upload_span_attributes() {
        let span = info_span!(
            "test_upload",
            user_id = field::Empty,
            file_size = field::Empty,
            content_type = field::Empty
        );
        let user_id = Uuid::new_v4();
        
        add_upload_span_attributes(&span, &user_id, 1024, "video/mp4");
        // Span attributes are recorded successfully
    }

    #[test]
    fn test_add_operation_status() {
        let span = info_span!("test_operation", operation_status = field::Empty);
        add_operation_status(&span, "success");
        // Status is recorded successfully
    }

    #[test]
    fn test_add_error_to_span() {
        let span = info_span!(
            "test_error",
            error = field::Empty,
            operation_status = field::Empty
        );
        add_error_to_span(&span, "Test error message");
        // Error is recorded successfully
    }
}
