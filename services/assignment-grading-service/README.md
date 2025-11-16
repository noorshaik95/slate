# Assignment Grading Service

A comprehensive microservice for managing course assignments, student submissions, grading, and gradebooks in the Slate Learning Management System.

## Features

### Assignment Management
- Create, update, and delete course assignments
- Configurable late submission policies with penalty percentages
- Prevent modification of assignments with existing submissions
- Event-driven architecture with Kafka integration

### Submission Management
- File upload support (PDF, DOCX, DOC, TXT, ZIP)
- Automatic late submission detection and tracking
- File size validation (configurable, default 25MB)
- Duplicate submission handling (replaces previous submission)
- Secure file storage with sanitization

### Grading System
- Create and update grades (draft mode)
- Automatic late penalty calculation and application
- Publish grades to make them visible to students
- Grade status tracking (draft/published)
- Feedback management

### Gradebook Features
- Student gradebook view with all assignments and grades
- Course gradebook with roster view
- Grade statistics (mean, median, standard deviation)
- CSV export functionality
- Weighted average and letter grade calculation (A/B/C/D/F)

## Architecture

### Technology Stack
- **Language**: Go 1.23
- **Framework**: gRPC
- **Database**: PostgreSQL 15
- **Message Broker**: Apache Kafka
- **Observability**: OpenTelemetry, Prometheus, Grafana, Tempo
- **Storage**: Local filesystem (extensible to S3)

### Service Components

#### Domain Models
- `Assignment`: Assignment configuration with late policy
- `Submission`: Student submission with file tracking
- `Grade`: Grade record with original and adjusted scores
- `LatePolicy`: Late submission policy configuration

#### Repositories
- `AssignmentRepository`: CRUD operations for assignments
- `SubmissionRepository`: Submission management with sorting
- `GradeRepository`: Grade management with statistics

#### Services
- `AssignmentService`: Assignment business logic
- `SubmissionService`: Submission handling with file storage
- `GradingService`: Grade calculation with late penalties
- `GradebookService`: Gradebook views and exports

### Database Schema

```sql
-- Assignments table
CREATE TABLE assignments (
    id UUID PRIMARY KEY,
    course_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    max_points DECIMAL(10, 2) NOT NULL,
    due_date TIMESTAMP NOT NULL,
    late_penalty_percent INT DEFAULT 0,
    max_late_days INT DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Submissions table (with UNIQUE constraint on assignment_id, student_id)
CREATE TABLE submissions (
    id UUID PRIMARY KEY,
    assignment_id UUID REFERENCES assignments(id),
    student_id VARCHAR(255) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    submitted_at TIMESTAMP NOT NULL,
    status VARCHAR(50) NOT NULL,
    is_late BOOLEAN NOT NULL,
    days_late INT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE(assignment_id, student_id)
);

-- Grades table
CREATE TABLE grades (
    id UUID PRIMARY KEY,
    submission_id UUID REFERENCES submissions(id),
    student_id VARCHAR(255) NOT NULL,
    assignment_id UUID REFERENCES assignments(id),
    score DECIMAL(10, 2) NOT NULL,
    adjusted_score DECIMAL(10, 2) NOT NULL,
    feedback TEXT,
    status VARCHAR(50) NOT NULL,
    graded_at TIMESTAMP,
    published_at TIMESTAMP,
    graded_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE(submission_id)
);
```

### Event Schema

The service publishes domain events to Kafka:

- `assignment.created`: New assignment created
- `assignment.updated`: Assignment modified
- `assignment.deleted`: Assignment removed
- `assignment.submitted`: Student submitted assignment
- `grade.published`: Grade made visible to student

## Configuration

Environment variables (see `.env.example`):

```bash
# gRPC Configuration
GRPC_HOST=0.0.0.0
GRPC_PORT=50053

# Database
DB_HOST=assignment-grading-db
DB_PORT=5432
DB_USER=postgres
DB_PASSWORD=postgres
DB_NAME=assignment_grading

# Kafka
KAFKA_ENABLED=true
KAFKA_BROKERS=kafka:9092
KAFKA_TOPIC=assignment-events

# Storage
STORAGE_TYPE=local
STORAGE_LOCAL_PATH=/app/storage
STORAGE_MAX_SIZE=26214400  # 25MB

# Observability
LOG_LEVEL=info
OTEL_EXPORTER_OTLP_ENDPOINT=tempo:4317
METRICS_PORT=9090
```

## Development

### Prerequisites
- Go 1.23+
- Protocol Buffers compiler (protoc)
- Docker and Docker Compose

### Local Development

1. Generate protobuf code:
```bash
make proto
```

2. Run tests:
```bash
make test
```

3. Build the service:
```bash
make build
```

4. Run locally:
```bash
make run
```

### Docker Development

Build and run with Docker Compose:
```bash
docker-compose up assignment-grading-service
```

## API Documentation

The service exposes four gRPC services:

### AssignmentService
- `CreateAssignment`: Create a new assignment
- `GetAssignment`: Retrieve assignment by ID
- `UpdateAssignment`: Modify an existing assignment
- `DeleteAssignment`: Remove an assignment
- `ListAssignments`: List assignments for a course (paginated)

### SubmissionService
- `SubmitAssignment`: Submit assignment with file upload
- `GetSubmission`: Retrieve submission by ID
- `ListSubmissions`: List submissions for an assignment
- `ListStudentSubmissions`: List all submissions for a student

### GradingService
- `CreateGrade`: Create a new grade (draft)
- `UpdateGrade`: Update a draft grade
- `PublishGrade`: Publish grade to student
- `GetGrade`: Retrieve grade by ID

### GradebookService
- `GetStudentGradebook`: Get student's complete gradebook
- `GetCourseGradebook`: Get gradebook for entire course
- `GetGradeStatistics`: Get statistics for an assignment
- `ExportGrades`: Export grades to CSV

## Security Features

- Input validation on all requests
- File type whitelist (PDF, DOCX, DOC, TXT, ZIP)
- File size limits (configurable, default 25MB)
- Filename sanitization to prevent path traversal
- SQL injection prevention with parameterized queries
- Authorization checks (instructor/student roles)
- Rate limiting support

## Observability

### Metrics
- Request counts and latencies
- Database query performance
- Kafka publish success/failure rates
- File storage operations
- Exposed on port 9090 for Prometheus scraping

### Tracing
- OpenTelemetry integration
- Distributed tracing with Tempo
- Trace context propagation
- gRPC interceptors for automatic span creation

### Logging
- Structured logging with zerolog
- Configurable log levels
- Request/response logging
- Error tracking

## Late Penalty Calculation

The service automatically calculates and applies late penalties:

1. **Calculate days late**:
   ```
   days_late = ceil((submitted_at - due_date) / 24 hours)
   ```

2. **Apply penalty**:
   ```
   total_penalty = days_late * penalty_percent_per_day
   adjusted_score = score * (1 - total_penalty / 100)
   ```

3. **Handle edge cases**:
   - If `max_late_days = 0`: No late submissions allowed
   - If `days_late > max_late_days`: Score = 0
   - If submitted before due date: No penalty

## Testing

The service includes comprehensive test coverage:

- Unit tests for models, services, and repositories
- Integration tests for database operations
- End-to-end tests for complete workflows
- Mock implementations for external dependencies

Run tests with:
```bash
go test -v ./...
```

## Deployment

The service is deployed via Docker Compose with:
- PostgreSQL database (dedicated instance)
- Kafka message broker
- Observability stack (Prometheus, Grafana, Tempo, Loki)
- API Gateway for HTTP/REST access

## Future Enhancements

- [ ] S3 integration for file storage
- [ ] Rubric-based grading
- [ ] Peer review functionality
- [ ] Plagiarism detection integration
- [ ] Assignment templates
- [ ] Group assignments
- [ ] Multiple graders per assignment
- [ ] Grade curves and normalization
- [ ] Export to LMS formats (Canvas, Blackboard)

## License

Copyright © 2025 Slate LMS
