use content_management_service::content::errors::ContentError;
use content_management_service::content::manager::ContentManager;
use content_management_service::models::{ContentType, CopyrightSetting};
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

#[tokio::test]
async fn test_create_module_with_valid_name() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    let result = manager
        .create_module(
            course_id,
            "Introduction to Rust".to_string(),
            Some("Learn the basics of Rust programming".to_string()),
            0,
            created_by,
        )
        .await;

    assert!(result.is_ok());
    let module = result.unwrap();
    assert_eq!(module.name, "Introduction to Rust");
    assert_eq!(module.course_id, course_id);
    assert_eq!(module.display_order, 0);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_create_module_with_empty_name() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool);

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    let result = manager
        .create_module(course_id, "".to_string(), None, 0, created_by)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Validation(msg) => {
            assert!(msg.contains("empty"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[tokio::test]
async fn test_create_module_with_name_too_long() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool);

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();
    let long_name = "a".repeat(201);

    let result = manager
        .create_module(course_id, long_name, None, 0, created_by)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Validation(msg) => {
            assert!(msg.contains("200 characters"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[tokio::test]
async fn test_create_lesson_with_valid_parent() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create parent module
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    // Create lesson
    let result = manager
        .create_lesson(
            module.id,
            "Lesson 1".to_string(),
            Some("First lesson".to_string()),
            0,
        )
        .await;

    assert!(result.is_ok());
    let lesson = result.unwrap();
    assert_eq!(lesson.name, "Lesson 1");
    assert_eq!(lesson.module_id, module.id);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_create_lesson_with_invalid_parent() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool);

    let non_existent_module_id = Uuid::new_v4();

    let result = manager
        .create_lesson(non_existent_module_id, "Lesson 1".to_string(), None, 0)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::NotFound(_) => {}
        _ => panic!("Expected not found error"),
    }
}

#[tokio::test]
async fn test_create_resource_with_valid_parent() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module and lesson
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    // Create resource
    let result = manager
        .create_resource(
            lesson.id,
            "Video 1".to_string(),
            Some("Introduction video".to_string()),
            ContentType::Video,
            1024 * 1024,
            "videos/test.mp4".to_string(),
            0,
        )
        .await;

    assert!(result.is_ok());
    let resource = result.unwrap();
    assert_eq!(resource.name, "Video 1");
    assert_eq!(resource.lesson_id, lesson.id);
    assert_eq!(resource.content_type, ContentType::Video);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_delete_module_with_children() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module with lesson
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    // Try to delete module
    let result = manager.delete_module(module.id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Conflict(msg) => {
            assert!(msg.contains("lessons"));
        }
        _ => panic!("Expected conflict error"),
    }

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_delete_lesson_with_children() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module, lesson, and resource
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    manager
        .create_resource(
            lesson.id,
            "Resource 1".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test.pdf".to_string(),
            0,
        )
        .await
        .unwrap();

    // Try to delete lesson
    let result = manager.delete_lesson(lesson.id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Conflict(msg) => {
            assert!(msg.contains("resources"));
        }
        _ => panic!("Expected conflict error"),
    }

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_reorder_modules_valid() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create three modules
    let module1 = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let module2 = manager
        .create_module(course_id, "Module 2".to_string(), None, 1, created_by)
        .await
        .unwrap();

    let module3 = manager
        .create_module(course_id, "Module 3".to_string(), None, 2, created_by)
        .await
        .unwrap();

    // Reorder: swap module1 and module3
    let reorder_items = vec![(module3.id, 0), (module2.id, 1), (module1.id, 2)];

    let result = manager.reorder_modules(course_id, reorder_items).await;
    assert!(result.is_ok());

    // Verify new order
    let modules = manager.list_modules(course_id).await.unwrap();
    assert_eq!(modules[0].id, module3.id);
    assert_eq!(modules[1].id, module2.id);
    assert_eq!(modules[2].id, module1.id);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_reorder_with_duplicate_positions() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create two modules
    let module1 = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let module2 = manager
        .create_module(course_id, "Module 2".to_string(), None, 1, created_by)
        .await
        .unwrap();

    // Try to reorder with duplicate positions
    let reorder_items = vec![(module1.id, 0), (module2.id, 0)];

    let result = manager.reorder_modules(course_id, reorder_items).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Validation(msg) => {
            assert!(msg.contains("Duplicate"));
        }
        _ => panic!("Expected validation error"),
    }

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_reorder_with_non_sequential_positions() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create two modules
    let module1 = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let module2 = manager
        .create_module(course_id, "Module 2".to_string(), None, 1, created_by)
        .await
        .unwrap();

    // Try to reorder with non-sequential positions (0, 2 instead of 0, 1)
    let reorder_items = vec![(module1.id, 0), (module2.id, 2)];

    let result = manager.reorder_modules(course_id, reorder_items).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        ContentError::Validation(msg) => {
            assert!(msg.contains("sequential"));
        }
        _ => panic!("Expected validation error"),
    }

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_publish_unpublish_resource() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module, lesson, and resource
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    let resource = manager
        .create_resource(
            lesson.id,
            "Resource 1".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test.pdf".to_string(),
            0,
        )
        .await
        .unwrap();

    // Resource should be unpublished by default
    assert!(!resource.published);

    // Publish resource
    manager.publish_resource(resource.id).await.unwrap();
    let updated = manager.get_resource(resource.id).await.unwrap();
    assert!(updated.published);

    // Unpublish resource
    manager.unpublish_resource(resource.id).await.unwrap();
    let updated = manager.get_resource(resource.id).await.unwrap();
    assert!(!updated.published);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_list_published_resources_filters_correctly() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module and lesson
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    // Create two resources
    let resource1 = manager
        .create_resource(
            lesson.id,
            "Resource 1".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test1.pdf".to_string(),
            0,
        )
        .await
        .unwrap();

    let resource2 = manager
        .create_resource(
            lesson.id,
            "Resource 2".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test2.pdf".to_string(),
            1,
        )
        .await
        .unwrap();

    // Publish only resource1
    manager.publish_resource(resource1.id).await.unwrap();

    // List published resources (student view)
    let published = manager.list_published_resources(lesson.id).await.unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].id, resource1.id);

    // List all resources (instructor view)
    let all = manager.list_all_resources(lesson.id).await.unwrap();
    assert_eq!(all.len(), 2);

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_get_content_structure_filters_by_role() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module, lesson, and resources
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    let resource1 = manager
        .create_resource(
            lesson.id,
            "Published Resource".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test1.pdf".to_string(),
            0,
        )
        .await
        .unwrap();

    let _resource2 = manager
        .create_resource(
            lesson.id,
            "Unpublished Resource".to_string(),
            None,
            ContentType::Pdf,
            1024,
            "docs/test2.pdf".to_string(),
            1,
        )
        .await
        .unwrap();

    // Publish only resource1
    manager.publish_resource(resource1.id).await.unwrap();

    // Get structure as student (should see only published)
    let student_structure = manager
        .get_content_structure(course_id, false)
        .await
        .unwrap();
    assert_eq!(student_structure.modules.len(), 1);
    assert_eq!(student_structure.modules[0].lessons.len(), 1);
    assert_eq!(student_structure.modules[0].lessons[0].resources.len(), 1);

    // Get structure as instructor (should see all)
    let instructor_structure = manager
        .get_content_structure(course_id, true)
        .await
        .unwrap();
    assert_eq!(instructor_structure.modules.len(), 1);
    assert_eq!(instructor_structure.modules[0].lessons.len(), 1);
    assert_eq!(
        instructor_structure.modules[0].lessons[0].resources.len(),
        2
    );

    cleanup_test_data(&pool, course_id).await;
}

#[tokio::test]
async fn test_update_resource_copyright_and_downloadable() {
    let pool = create_test_pool().await;
    let manager = ContentManager::new(pool.clone());

    let course_id = Uuid::new_v4();
    let created_by = Uuid::new_v4();

    // Create module, lesson, and resource
    let module = manager
        .create_module(course_id, "Module 1".to_string(), None, 0, created_by)
        .await
        .unwrap();

    let lesson = manager
        .create_lesson(module.id, "Lesson 1".to_string(), None, 0)
        .await
        .unwrap();

    let resource = manager
        .create_resource(
            lesson.id,
            "Video 1".to_string(),
            None,
            ContentType::Video,
            1024 * 1024,
            "videos/test.mp4".to_string(),
            0,
        )
        .await
        .unwrap();

    // Update downloadable and copyright settings
    let updated = manager
        .update_resource(
            resource.id,
            None,
            None,
            Some(true),
            Some(CopyrightSetting::EducationalUseOnly),
        )
        .await
        .unwrap();

    assert!(updated.downloadable);
    assert_eq!(
        updated.copyright_setting,
        CopyrightSetting::EducationalUseOnly
    );

    cleanup_test_data(&pool, course_id).await;
}
