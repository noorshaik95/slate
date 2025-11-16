# Content Management Service Tests

This directory contains tests for the Content Management Service, organized by test type and scope.

## Test Files

### Unit Tests

#### `content_manager_test.rs`
Tests for the ContentManager component covering:
- Module/lesson/resource creation with validation
- Hierarchy enforcement (max 3 levels)
- Deletion with child checking
- Content reordering logic
- Publication control
- Content structure retrieval with role-based filtering

**Requirements Covered**: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2, 3.3, 3.4, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4, 5.5

#### `upload_handler_test.rs`
Tests for the UploadHandler component covering:
- File type and size validation
- Chunked upload flow
- Upload resumption
- Session expiration
- File header verification

**Requirements Covered**: 2.1, 2.2, 2.3, 2.4, 2.5, 2.7

#### `progress_tracker_test.rs`
Tests for the ProgressTracker component covering:
- Completion marking
- Progress calculation
- Automatic video completion at 90%
- Concurrent updates handling
- Progress percentage rounding

**Requirements Covered**: 8.1, 8.2, 8.3, 8.4, 8.5, 9.1, 9.2, 9.3, 9.4, 9.5, 10.4, 10.5

### Integration Tests

#### `database_integration_test.rs`
Tests for database integration covering:
- Repository layer with PostgreSQL
- Database migrations
- Connection pooling
- CRUD operations for all repositories

**Requirements Covered**: 19.1, 19.2, 19.5, 19.6

**Note**: These tests are marked with `#[ignore]` and require a test database. Set `TEST_DATABASE_URL` environment variable to run them.

#### `minio_integration_test.rs`
Tests for MinIO/S3 integration covering:
- File upload and download
- Presigned URL generation
- Chunked upload assembly
- Server-side encryption

**Requirements Covered**: 19.3, 19.4

**Note**: These tests are marked with `#[ignore]` and require MinIO to be running. Set the following environment variables:
- `TEST_S3_ENDPOINT`
- `TEST_S3_BUCKET`
- `TEST_S3_ACCESS_KEY`
- `TEST_S3_SECRET_KEY`

### End-to-End Tests

#### `end_to_end_test.rs`
Tests for complete workflows covering:
- Complete upload flow (initiate → chunks → complete → verify)
- Video transcoding workflow (upload → transcode → manifest)
- Progress tracking workflow (mark complete → calculate → report)
- Search workflow (index → search → results)
- Download workflow (request → validate → generate URL → track)

**Requirements Covered**: All requirements

**Note**: These tests are marked with `#[ignore]` and require the full service stack:
- PostgreSQL database
- MinIO/S3
- Redis
- ElasticSearch
- Content Management Service

## Running Tests

### Run all unit tests (no external dependencies required)
```bash
cargo test --lib
```

### Run specific test file
```bash
cargo test --test content_manager_test
cargo test --test upload_handler_test
cargo test --test progress_tracker_test
```

### Run integration tests (requires test infrastructure)
```bash
# Set up test environment first
export TEST_DATABASE_URL="postgresql://postgres:postgres@localhost:5432/cms_test"
export TEST_S3_ENDPOINT="http://localhost:9000"
export TEST_S3_BUCKET="test-bucket"
export TEST_S3_ACCESS_KEY="minioadmin"
export TEST_S3_SECRET_KEY="minioadmin"
export TEST_REDIS_URL="redis://localhost:6379"
export TEST_ELASTICSEARCH_URL="http://localhost:9200"

# Run ignored tests
cargo test --test database_integration_test -- --ignored
cargo test --test minio_integration_test -- --ignored
cargo test --test end_to_end_test -- --ignored
```

### Run all tests including ignored ones
```bash
cargo test -- --include-ignored
```

## Test Database Setup

To run integration tests, you need a test database:

```bash
# Create test database
createdb cms_test

# Run migrations
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/cms_test"
sqlx migrate run
```

## Test Coverage

The tests cover the following aspects:

1. **Business Logic Validation**
   - Name length validation (1-200 characters)
   - File size limits (500MB max)
   - Hierarchy enforcement (3 levels max)
   - Display order validation

2. **Data Integrity**
   - Parent-child relationships
   - Deletion constraints
   - Unique constraints
   - Sequential ordering

3. **Progress Tracking**
   - Completion marking
   - Progress calculation
   - Auto-completion at 90% for videos
   - Report generation

4. **File Upload**
   - Chunked upload flow
   - Session management
   - File validation
   - Resume capability

5. **Publication Control**
   - Default unpublished status
   - Role-based visibility
   - Publication status toggling

6. **Content Reordering**
   - Position validation
   - Sequential ordering
   - Duplicate detection

## Notes

- Unit tests focus on business logic and don't require external dependencies
- Integration tests require actual database and storage infrastructure
- End-to-end tests require the full service stack to be running
- Tests marked with `#[ignore]` are skipped by default and must be explicitly run
- Some test functions are placeholders documenting what needs to be tested but not fully implemented

## Future Improvements

- Add more comprehensive integration tests with actual database operations
- Implement full end-to-end test scenarios
- Add performance benchmarks
- Add load testing for concurrent operations
- Add chaos testing for resilience validation
