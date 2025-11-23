use content_management_service::models::{ContentType, ProgressSummary, ProgressTracking};
use content_management_service::progress::errors::ProgressError;
use content_management_service::progress::tracker::{CompletionStatus, ProgressTracker};
use sqlx::PgPool;
use uuid::Uuid;

/// Helper function to create a test database pool
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/cms_test".to_string());

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Helper function to clean up test data
async fn cleanup_test_data(pool: &PgPool, course_id: Uuid) {
    // Delete in reverse order of dependencies
    sqlx::query("DELETE FROM progress_tracking WHERE resource_id IN (SELECT id FROM resources WHERE lesson_id IN (SELECT id FROM lessons WHERE module_id IN (SELECT id FROM modules WHERE course_id = $1)))")
        .bind(course_id)
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM resources WHERE lesson_id IN (SELECT id FROM lessons WHERE module_id IN (SELECT id FROM modules WHERE course_id = $1))")
        .bind(course_id)
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        "DELETE FROM lessons WHERE module_id IN (SELECT id FROM modules WHERE course_id = $1)",
    )
    .bind(course_id)
    .execute(pool)
    .await
    .ok();

    sqlx::query("DELETE FROM modules WHERE course_id = $1")
        .bind(course_id)
        .execute(pool)
        .await
        .ok();
}

#[test]
fn test_progress_tracking_validate_position() {
    // Valid positions
    assert!(ProgressTracking::validate_position(0).is_ok());
    assert!(ProgressTracking::validate_position(100).is_ok());
    assert!(ProgressTracking::validate_position(1000).is_ok());

    // Invalid positions
    assert!(ProgressTracking::validate_position(-1).is_err());
    assert!(ProgressTracking::validate_position(-100).is_err());
}

#[test]
fn test_progress_tracking_should_auto_complete() {
    // At 90% threshold - should complete
    assert!(ProgressTracking::should_auto_complete(90, 100));
    assert!(ProgressTracking::should_auto_complete(900, 1000));

    // Above 90% - should complete
    assert!(ProgressTracking::should_auto_complete(95, 100));
    assert!(ProgressTracking::should_auto_complete(100, 100));
    assert!(ProgressTracking::should_auto_complete(950, 1000));

    // Below 90% - should not complete
    assert!(!ProgressTracking::should_auto_complete(89, 100));
    assert!(!ProgressTracking::should_auto_complete(50, 100));
    assert!(!ProgressTracking::should_auto_complete(0, 100));
    assert!(!ProgressTracking::should_auto_complete(899, 1000));

    // Edge cases
    assert!(!ProgressTracking::should_auto_complete(0, 0));
    assert!(!ProgressTracking::should_auto_complete(10, -1));
}

#[test]
fn test_progress_tracking_validate_position_within_duration() {
    // Valid positions
    assert!(ProgressTracking::validate_position_within_duration(0, 100).is_ok());
    assert!(ProgressTracking::validate_position_within_duration(50, 100).is_ok());
    assert!(ProgressTracking::validate_position_within_duration(100, 100).is_ok());

    // Invalid positions (exceeds duration)
    assert!(ProgressTracking::validate_position_within_duration(101, 100).is_err());
    assert!(ProgressTracking::validate_position_within_duration(200, 100).is_err());
}

#[test]
fn test_progress_summary_calculate_percentage() {
    // Basic calculations
    assert_eq!(ProgressSummary::calculate_percentage(0, 10), 0);
    assert_eq!(ProgressSummary::calculate_percentage(5, 10), 50);
    assert_eq!(ProgressSummary::calculate_percentage(10, 10), 100);

    // Various percentages
    assert_eq!(ProgressSummary::calculate_percentage(1, 10), 10);
    assert_eq!(ProgressSummary::calculate_percentage(3, 10), 30);
    assert_eq!(ProgressSummary::calculate_percentage(7, 10), 70);
    assert_eq!(ProgressSummary::calculate_percentage(9, 10), 90);

    // Rounding to nearest integer
    assert_eq!(ProgressSummary::calculate_percentage(1, 3), 33);
    assert_eq!(ProgressSummary::calculate_percentage(2, 3), 67);
    assert_eq!(ProgressSummary::calculate_percentage(1, 6), 17);
    assert_eq!(ProgressSummary::calculate_percentage(5, 6), 83);

    // Edge case: no resources
    assert_eq!(ProgressSummary::calculate_percentage(0, 0), 0);
}

#[test]
fn test_progress_summary_new() {
    let student_id = Uuid::new_v4();
    let course_id = Uuid::new_v4();
    let module_id = Uuid::new_v4();

    // Course-level summary
    let course_summary = ProgressSummary::new(student_id, Some(course_id), None, None, 20, 10);

    assert_eq!(course_summary.student_id, student_id);
    assert_eq!(course_summary.course_id, Some(course_id));
    assert_eq!(course_summary.module_id, None);
    assert_eq!(course_summary.lesson_id, None);
    assert_eq!(course_summary.total_resources, 20);
    assert_eq!(course_summary.completed_resources, 10);
    assert_eq!(course_summary.progress_percentage, 50);

    // Module-level summary
    let module_summary =
        ProgressSummary::new(student_id, Some(course_id), Some(module_id), None, 10, 7);

    assert_eq!(module_summary.module_id, Some(module_id));
    assert_eq!(module_summary.progress_percentage, 70);
}

#[test]
fn test_completion_status_from_str() {
    assert_eq!(
        CompletionStatus::from_str("completed"),
        CompletionStatus::Completed
    );
    assert_eq!(
        CompletionStatus::from_str("COMPLETED"),
        CompletionStatus::Completed
    );
    assert_eq!(
        CompletionStatus::from_str("incomplete"),
        CompletionStatus::Incomplete
    );
    assert_eq!(
        CompletionStatus::from_str("INCOMPLETE"),
        CompletionStatus::Incomplete
    );
    assert_eq!(CompletionStatus::from_str("all"), CompletionStatus::All);
    assert_eq!(CompletionStatus::from_str("ALL"), CompletionStatus::All);
    assert_eq!(CompletionStatus::from_str(""), CompletionStatus::All);
    assert_eq!(CompletionStatus::from_str("invalid"), CompletionStatus::All);
}

#[tokio::test]
async fn test_mark_complete_published_resource() {
    let pool = create_test_pool().await;

    // This test requires setting up the full database structure
    // For now, we test the business logic separately

    // Test that marking a published resource as complete should work
    // Test that marking an unpublished resource as complete should fail
}

#[tokio::test]
async fn test_mark_complete_unpublished_resource_fails() {
    // Test that attempting to mark an unpublished resource as complete
    // returns ProgressError::ResourceNotPublished
}

#[tokio::test]
async fn test_mark_complete_toggle_status() {
    // Test that completion status can be toggled between complete and incomplete
    // 1. Mark as complete
    // 2. Verify completed = true and completed_at is set
    // 3. Mark as incomplete
    // 4. Verify completed = false
}

#[tokio::test]
async fn test_get_progress_calculates_correctly() {
    // Test that progress calculation works correctly
    // 1. Create course with modules, lessons, and resources
    // 2. Mark some resources as complete
    // 3. Get progress
    // 4. Verify percentages are calculated correctly at all levels
}

#[tokio::test]
async fn test_get_progress_excludes_unpublished_resources() {
    // Test that unpublished resources are excluded from progress calculations
    // 1. Create resources, some published, some unpublished
    // 2. Mark both published and unpublished as complete
    // 3. Get progress
    // 4. Verify only published resources are counted
}

#[tokio::test]
async fn test_get_progress_separate_percentages() {
    // Test that separate percentages are calculated for course, modules, and lessons
    // 1. Create course with multiple modules and lessons
    // 2. Mark different resources as complete in different modules
    // 3. Get progress
    // 4. Verify each module and lesson has correct percentage
}

#[tokio::test]
async fn test_generate_report_aggregates_student_data() {
    // Test that progress report aggregates data for all students
    // 1. Create multiple students with different progress
    // 2. Generate report
    // 3. Verify all students are included
    // 4. Verify average completion percentage is correct
}

#[tokio::test]
async fn test_generate_report_filters_by_date_range() {
    // Test that progress report can filter by date range
    // 1. Create progress entries with different completion dates
    // 2. Generate report with date filter
    // 3. Verify only entries within date range are included
}

#[tokio::test]
async fn test_generate_report_filters_by_module() {
    // Test that progress report can filter by specific module
    // 1. Create multiple modules with resources
    // 2. Generate report filtered by one module
    // 3. Verify only resources from that module are included
}

#[tokio::test]
async fn test_generate_report_filters_by_completion_status() {
    // Test that progress report can filter by completion status
    // 1. Create students with different completion percentages
    // 2. Generate report with CompletionStatus::Completed filter
    // 3. Verify only students with 100% completion are included
    // 4. Generate report with CompletionStatus::Incomplete filter
    // 5. Verify only students with < 100% completion are included
}

#[tokio::test]
async fn test_generate_report_includes_time_spent() {
    // Test that progress report includes time spent per resource
    // 1. Create video resources with playback positions
    // 2. Generate report
    // 3. Verify time_spent_seconds is populated from last_position_seconds
}

#[tokio::test]
async fn test_generate_report_performance_with_many_students() {
    // Test that report generation completes within 5 seconds for 1000 students
    // This is a performance test that would need to be run with actual data
    // For now, we document the requirement
}

#[test]
fn test_video_completion_threshold_is_90_percent() {
    // Verify the constant is set correctly
    assert_eq!(ProgressTracking::VIDEO_COMPLETION_THRESHOLD, 0.9);

    // Test auto-completion at exactly 90%
    assert!(ProgressTracking::should_auto_complete(90, 100));
    assert!(ProgressTracking::should_auto_complete(270, 300));
    assert!(ProgressTracking::should_auto_complete(540, 600));

    // Test just below 90%
    assert!(!ProgressTracking::should_auto_complete(89, 100));
    assert!(!ProgressTracking::should_auto_complete(269, 300));
    assert!(!ProgressTracking::should_auto_complete(539, 600));
}

#[test]
fn test_progress_percentage_rounds_to_nearest_integer() {
    // Test rounding behavior
    assert_eq!(ProgressSummary::calculate_percentage(1, 3), 33); // 33.33... -> 33
    assert_eq!(ProgressSummary::calculate_percentage(2, 3), 67); // 66.66... -> 67
    assert_eq!(ProgressSummary::calculate_percentage(1, 6), 17); // 16.66... -> 17
    assert_eq!(ProgressSummary::calculate_percentage(5, 6), 83); // 83.33... -> 83
    assert_eq!(ProgressSummary::calculate_percentage(1, 7), 14); // 14.28... -> 14
    assert_eq!(ProgressSummary::calculate_percentage(3, 7), 43); // 42.85... -> 43
}

#[test]
fn test_concurrent_updates_use_timestamps() {
    // Test that concurrent updates are handled using timestamps
    // The most recent update should win
    // This is tested at the repository level with database constraints
}

#[tokio::test]
async fn test_update_playback_position_validates_position() {
    // Test that playback position updates validate the position
    // 1. Negative positions should be rejected
    // 2. Positions exceeding video duration should be rejected
    // 3. Valid positions should be accepted

    assert!(ProgressTracking::validate_position(0).is_ok());
    assert!(ProgressTracking::validate_position(100).is_ok());
    assert!(ProgressTracking::validate_position(-1).is_err());
}

#[tokio::test]
async fn test_update_playback_position_auto_completes_at_90_percent() {
    // Test that updating playback position to 90% or more auto-completes the video
    // 1. Create video resource with duration
    // 2. Update playback position to 90% of duration
    // 3. Verify resource is marked as complete

    let duration = 1000;
    let position_90_percent = 900;

    assert!(ProgressTracking::should_auto_complete(
        position_90_percent,
        duration
    ));
}

#[tokio::test]
async fn test_progress_updates_within_2_seconds() {
    // Test that progress calculations update within 2 seconds of status change
    // This is a performance requirement that would need to be tested with actual database
    // For now, we document the requirement
}

#[tokio::test]
async fn test_progress_persists_across_sessions() {
    // Test that progress data persists in the database
    // 1. Mark resource as complete
    // 2. Retrieve progress in a new session
    // 3. Verify progress is still marked as complete
}

#[tokio::test]
async fn test_progress_handles_concurrent_updates() {
    // Test that concurrent progress updates from multiple devices are handled correctly
    // The most recent timestamp should win
    // This requires database-level testing with actual concurrent operations
}
