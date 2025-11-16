# Course Management Service

A comprehensive course management microservice built with NestJS, MongoDB, and gRPC. This service handles all course-related operations including course CRUD, enrollment management, templates, prerequisites, co-teaching, sections, and cross-listing.

## Features

### Course Management
- **CRUD Operations**: Create, read, update, and delete courses
- **Publishing**: Publish and unpublish courses
- **Course Metadata**: Department, course code, credits, max students, and tags
- **Course Templates**: Create reusable course templates
- **Create from Template**: Quickly create new courses from templates

### Enrollment Management
- **Self-Enrollment**: Students can enroll themselves in published courses
- **Instructor-Add**: Instructors can add students to their courses
- **Roster Management**: View course rosters with filtering by section and status
- **Student Enrollments**: View all courses a student is enrolled in
- **Enrollment Status**: Active, Dropped, Completed, Waitlisted

### Advanced Features
- **Prerequisites**: Define and check course prerequisites with circular dependency detection
- **Co-Teaching**: Support for multiple instructors per course
- **Course Sections**: Multiple sections per course with schedules and capacity management
- **Cross-Listing**: Link courses across departments

## Technology Stack

- **Framework**: NestJS (TypeScript)
- **Database**: MongoDB with Mongoose ODM
- **Communication**: gRPC for microservice communication
- **Observability**:
  - Logging: Pino (structured JSON logging)
  - Metrics: Prometheus
  - Tracing: OpenTelemetry with Tempo

## Architecture

```
src/
├── course/                 # Course domain
│   ├── schemas/           # Mongoose schemas
│   ├── repositories/      # Data access layer
│   ├── course.service.ts  # Business logic
│   └── course.controller.ts # gRPC handlers
├── enrollment/            # Enrollment domain
│   ├── schemas/
│   ├── repositories/
│   ├── enrollment.service.ts
│   └── enrollment.module.ts
├── observability/         # Observability setup
│   ├── logger.config.ts
│   ├── metrics.service.ts
│   └── tracing.ts
├── config/               # Configuration
├── health/               # Health checks
└── main.ts              # Application entry point
```

## API (gRPC)

### Course Operations
- `CreateCourse` - Create a new course
- `GetCourse` - Get course details
- `UpdateCourse` - Update course information
- `DeleteCourse` - Delete a course
- `ListCourses` - List courses with filtering
- `PublishCourse` - Publish a course
- `UnpublishCourse` - Unpublish a course

### Enrollment Operations
- `SelfEnroll` - Student self-enrollment
- `InstructorAddStudent` - Instructor adds a student
- `RemoveEnrollment` - Remove an enrollment
- `GetCourseRoster` - Get course roster
- `GetStudentEnrollments` - Get student's enrollments

### Template Operations
- `CreateCourseTemplate` - Create a course template
- `GetCourseTemplate` - Get template details
- `ListCourseTemplates` - List all templates
- `CreateCourseFromTemplate` - Create course from template

### Prerequisite Operations
- `AddPrerequisite` - Add a prerequisite to a course
- `RemovePrerequisite` - Remove a prerequisite
- `CheckPrerequisites` - Check if student meets prerequisites

### Co-Teaching Operations
- `AddCoInstructor` - Add a co-instructor
- `RemoveCoInstructor` - Remove a co-instructor

### Section Operations
- `CreateSection` - Create a course section
- `GetSection` - Get section details
- `UpdateSection` - Update section information
- `DeleteSection` - Delete a section
- `ListCourseSections` - List all sections for a course

### Cross-Listing Operations
- `CrossListCourses` - Create a cross-listing group
- `RemoveCrossListing` - Remove a cross-listing
- `GetCrossListedCourses` - Get all cross-listed courses

## Environment Variables

```bash
# Server Configuration
PORT=3001
GRPC_HOST=0.0.0.0
GRPC_PORT=50052

# MongoDB Configuration
MONGO_URI=mongodb://mongodb:27017/courses
MONGO_DB_NAME=courses

# Observability
OTEL_EXPORTER_OTLP_ENDPOINT=http://tempo:4317
LOG_LEVEL=info
METRICS_PORT=9090
SERVICE_VERSION=1.0.0

# Environment
NODE_ENV=production
```

## Running the Service

### Development
```bash
cd services/course-service
npm install
npm run start:dev
```

### Production (Docker)
```bash
# From repository root
docker-compose up course-service
```

## Health Checks

The service exposes HTTP health check endpoints:

- `GET /health` - Overall health status
- `GET /health/liveness` - Liveness probe
- `GET /health/readiness` - Readiness probe (checks DB connection)

## Metrics

Prometheus metrics are exposed at `http://localhost:9090/metrics`

### Custom Metrics
- `courses_created_total` - Total courses created
- `courses_published_total` - Total courses published
- `courses_unpublished_total` - Total courses unpublished
- `enrollments_created_total` - Total enrollments created
- `enrollments_removed_total` - Total enrollments removed
- `self_enrollments_total` - Total self-enrollments
- `instructor_enrollments_total` - Total instructor-added enrollments
- `templates_created_total` - Total templates created
- `courses_from_templates_total` - Total courses created from templates
- `grpc_request_duration_seconds` - gRPC request duration histogram
- `mongodb_connections` - MongoDB connection status

## Database Schema

### Course
- Title, description, term, syllabus
- Instructor and co-instructors
- Published status
- Prerequisites
- Template reference
- Cross-listing group
- Metadata (department, course code, credits, max students, tags)

### Enrollment
- Course and student references
- Enrollment type (self, instructor, admin)
- Status (active, dropped, completed, waitlisted)
- Section reference
- Enrolled by and timestamp

### CourseTemplate
- Name, description
- Syllabus template
- Default metadata

### Section
- Course reference
- Section number
- Instructor
- Schedule (days, times)
- Location
- Capacity and enrollment count

### CrossListing
- Group ID
- Course references
- Created by

## Development

### Running Tests

**Unit Tests**:
```bash
npm test                 # Run all unit tests
npm run test:watch       # Run tests in watch mode
npm run test:cov         # Run with coverage report
```

**Integration Tests**:
```bash
# Ensure MongoDB is running
docker run -d -p 27017:27017 mongo:7-jammy

# Run integration tests
npm run test:e2e
```

**Coverage**: The project maintains **80%+ test coverage** across:
- 199 unit test cases
- 26 integration test cases
- Services, repositories, controllers, and interceptors

See [TESTING.md](./TESTING.md) for detailed testing documentation.

### Linting & Formatting
```bash
npm run lint             # Check for linting errors
npm run lint:fix         # Auto-fix linting issues
npm run format           # Format code with Prettier
npm run format:check     # Check formatting without changes
```

### Building
```bash
npm run build            # Compile TypeScript to dist/
```

### CI/CD Pipeline

The service includes a comprehensive GitHub Actions pipeline that runs on every push/PR:

**Pipeline Jobs**:
1. **Test** - Runs unit and integration tests on Node.js 18.x & 20.x
2. **Lint** - Validates code quality and formatting
3. **Build** - Compiles TypeScript and verifies build
4. **Docker** - Builds and tests Docker image
5. **Security** - Scans for vulnerabilities

**Quality Gates**:
- ✅ All tests must pass
- ✅ 80% minimum code coverage
- ✅ No linting errors
- ✅ Successful build
- ✅ Security scan passes

See [CI-CD.md](./CI-CD.md) for complete pipeline documentation.

## Integration with API Gateway

The Course Service is designed to work behind the API Gateway, which handles:
- REST to gRPC translation
- Authentication and authorization
- Rate limiting
- Request validation

## Observability

### Logging
- Structured JSON logs via Pino
- Automatic request/response logging
- Sensitive data redaction (passwords, tokens)
- Ingested by Loki for querying in Grafana

### Metrics
- Business metrics (enrollments, course creation)
- Performance metrics (request duration)
- System metrics (database connections)
- Scraped by Prometheus

### Tracing
- Distributed tracing with OpenTelemetry
- Automatic instrumentation for HTTP, gRPC, and MongoDB
- Traces sent to Tempo
- Full request flow visibility

## Security Considerations

- Input validation on all gRPC endpoints
- Authorization checks for instructor operations
- Sensitive data redaction in logs
- Circular prerequisite detection
- Section capacity enforcement

## Future Enhancements

- Assignment and grading integration
- Course materials and resources
- Discussion forums per course
- Course calendar and scheduling
- Student performance analytics
- Bulk enrollment operations
- Course archival and restoration
