# Course Management Service - API Routes

This document describes how REST API endpoints are mapped to gRPC methods through the API Gateway.

## Service Discovery

The Course Management Service uses **gRPC Server Reflection** for automatic service discovery by the API Gateway. Methods are auto-discovered and mapped to REST endpoints based on naming conventions.

- **Service Endpoint**: `http://course-service:50052`
- **Auto-Discovery**: Enabled via `@grpc/reflection`
- **Refresh Interval**: 5 minutes

## Routing Conventions

### Auto-Discovered Routes (Standard CRUD)

These methods follow the API Gateway's naming conventions and are automatically mapped:

| HTTP Method | REST Path | gRPC Method | Description |
|-------------|-----------|-------------|-------------|
| **Courses** |
| POST | `/api/courses` | `CreateCourse` | Create a new course |
| GET | `/api/courses/:id` | `GetCourse` | Get course details |
| PUT | `/api/courses/:id` | `UpdateCourse` | Update course |
| DELETE | `/api/courses/:id` | `DeleteCourse` | Delete course |
| GET | `/api/courses` | `ListCourses` | List all courses (with filters) |
| **Templates** |
| POST | `/api/coursetemplates` | `CreateCourseTemplate` | Create course template |
| GET | `/api/coursetemplates/:id` | `GetCourseTemplate` | Get template details |
| GET | `/api/coursetemplates` | `ListCourseTemplates` | List all templates |
| **Sections** |
| POST | `/api/sections` | `CreateSection` | Create course section |
| GET | `/api/sections/:id` | `GetSection` | Get section details |
| PUT | `/api/sections/:id` | `UpdateSection` | Update section |
| DELETE | `/api/sections/:id` | `DeleteSection` | Delete section |

### Manual Route Overrides (Non-Standard Methods)

These methods don't follow the standard naming convention and require explicit route configuration:

| HTTP Method | REST Path | gRPC Method | Description |
|-------------|-----------|-------------|-------------|
| **Course Publishing** |
| POST | `/api/courses/:id/publish` | `PublishCourse` | Publish a course |
| POST | `/api/courses/:id/unpublish` | `UnpublishCourse` | Unpublish a course |
| **Enrollment** |
| POST | `/api/courses/:id/enroll` | `SelfEnroll` | Student self-enrollment |
| POST | `/api/courses/:id/students` | `InstructorAddStudent` | Instructor adds student |
| DELETE | `/api/enrollments/:id` | `RemoveEnrollment` | Remove enrollment |
| GET | `/api/courses/:id/roster` | `GetCourseRoster` | Get course roster |
| GET | `/api/students/:id/enrollments` | `GetStudentEnrollments` | Get student enrollments |
| **Templates** |
| POST | `/api/templates/:id/create-course` | `CreateCourseFromTemplate` | Create course from template |
| **Prerequisites** |
| POST | `/api/courses/:id/prerequisites` | `AddPrerequisite` | Add prerequisite |
| DELETE | `/api/courses/:id/prerequisites/:prerequisite_id` | `RemovePrerequisite` | Remove prerequisite |
| POST | `/api/courses/:id/prerequisites/check` | `CheckPrerequisites` | Check if student meets prerequisites |
| **Co-teaching** |
| POST | `/api/courses/:id/co-instructors` | `AddCoInstructor` | Add co-instructor |
| DELETE | `/api/courses/:id/co-instructors/:co_instructor_id` | `RemoveCoInstructor` | Remove co-instructor |
| **Sections** |
| GET | `/api/courses/:id/sections` | `ListCourseSections` | List course sections |
| **Cross-listing** |
| POST | `/api/courses/cross-list` | `CrossListCourses` | Create cross-listing |
| DELETE | `/api/cross-listings/:id` | `RemoveCrossListing` | Remove cross-listing |
| GET | `/api/courses/:id/cross-listed` | `GetCrossListedCourses` | Get cross-listed courses |

## Request/Response Flow

### Example: Create a Course

**1. Client Request (REST)**
```http
POST /api/courses HTTP/1.1
Host: api-gateway:8080
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Introduction to Computer Science",
  "description": "Learn the basics of CS",
  "term": "Fall 2024",
  "instructor_id": "instructor-123",
  "metadata": {
    "department": "CS",
    "course_code": "CS101",
    "credits": 3
  }
}
```

**2. API Gateway Processing**
- Validates authentication token
- Matches route: `POST /api/courses` → `course.CourseService/CreateCourse`
- Converts JSON to gRPC request
- Adds metadata (trace-id, user-id from token)

**3. gRPC Request to Course Service**
```protobuf
// Method: course.CourseService/CreateCourse
{
  title: "Introduction to Computer Science",
  description: "Learn the basics of CS",
  term: "Fall 2024",
  instructor_id: "instructor-123",
  metadata: {
    department: "CS",
    course_code: "CS101",
    credits: 3
  }
}
```

**4. Course Service Response**
```protobuf
{
  course: {
    id: "course-abc123",
    title: "Introduction to Computer Science",
    description: "Learn the basics of CS",
    term: "Fall 2024",
    instructor_id: "instructor-123",
    is_published: false,
    created_at: "2024-01-15T10:30:00Z",
    // ...
  }
}
```

**5. API Gateway Response (REST)**
```http
HTTP/1.1 201 Created
Content-Type: application/json
X-Trace-Id: abc123-def456

{
  "course": {
    "id": "course-abc123",
    "title": "Introduction to Computer Science",
    "description": "Learn the basics of CS",
    "term": "Fall 2024",
    "instructor_id": "instructor-123",
    "is_published": false,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

## Path Parameters

Path parameters (`:id`, `:prerequisite_id`, etc.) are automatically extracted and merged into the gRPC request payload.

**Example:**
```
GET /api/courses/course-123
```
Translates to:
```protobuf
GetCourse({
  course_id: "course-123"
})
```

## Query Parameters

Query parameters are passed through for filtering and pagination:

```
GET /api/courses?instructor_id=instructor-123&term=Fall+2024&page=1&page_size=20
```

Translates to:
```protobuf
ListCourses({
  instructor_id: "instructor-123",
  term: "Fall 2024",
  page: 1,
  page_size: 20
})
```

## Authentication

Most endpoints require authentication. The API Gateway:
1. Validates the JWT token
2. Extracts user_id and roles
3. Passes them as gRPC metadata

Public endpoints (no auth required):
- `GET /api/courses` (list published courses)
- `GET /api/courses/:id` (view published course)

## Error Handling

gRPC status codes are mapped to HTTP status codes:

| gRPC Status | HTTP Status | Description |
|-------------|-------------|-------------|
| OK | 200 | Success |
| INVALID_ARGUMENT | 400 | Bad request |
| UNAUTHENTICATED | 401 | Authentication required |
| PERMISSION_DENIED | 403 | Forbidden |
| NOT_FOUND | 404 | Resource not found |
| ALREADY_EXISTS | 409 | Conflict (e.g., already enrolled) |
| INTERNAL | 500 | Server error |
| UNAVAILABLE | 503 | Service unavailable |

## Rate Limiting

The API Gateway applies rate limiting:
- Default: 100 requests per minute per IP
- Configurable per endpoint
- Returns HTTP 429 when limit exceeded

## Circuit Breaker

The API Gateway includes a circuit breaker for the Course Service:
- **Failure Threshold**: 5 consecutive failures
- **Success Threshold**: 2 consecutive successes to close
- **Timeout**: 30 seconds before retry

## Observability

### Distributed Tracing
- Every request gets a unique trace ID
- Trace context is propagated via gRPC metadata
- Full request flow visible in Tempo/Grafana

### Metrics
- Request duration histograms
- Error rates by endpoint
- Circuit breaker status

### Logging
- Structured JSON logs
- Correlation via trace ID
- Sensitive data redacted

## Configuration

The routing configuration is located in:
- **Docker**: `/config/gateway-config.docker.yaml`
- **Local**: `/config/gateway-config.yaml`

To add new routes:
1. Add method to `course.proto`
2. Implement in Course Service
3. If method follows conventions → auto-discovered
4. If method doesn't follow conventions → add to `route_overrides` in gateway config
5. Restart API Gateway to pick up changes

## Testing Routes

### Using cURL
```bash
# Create a course
curl -X POST http://localhost:8080/api/courses \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Test Course",
    "description": "Test",
    "term": "Fall 2024",
    "instructor_id": "instructor-1"
  }'

# Get course
curl http://localhost:8080/api/courses/course-123 \
  -H "Authorization: Bearer <token>"

# Enroll in course
curl -X POST http://localhost:8080/api/courses/course-123/enroll \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"student_id": "student-1"}'
```

### Using grpcurl (direct gRPC)
```bash
# List services
grpcurl -plaintext localhost:50052 list

# Describe service
grpcurl -plaintext localhost:50052 describe course.CourseService

# Call method
grpcurl -plaintext -d '{"course_id": "course-123"}' \
  localhost:50052 course.CourseService/GetCourse
```

## Summary

- ✅ **12 auto-discovered routes** (standard CRUD patterns)
- ✅ **21 manually configured routes** (custom operations)
- ✅ **33 total REST endpoints** mapped to gRPC
- ✅ Full authentication and authorization
- ✅ Rate limiting and circuit breaker
- ✅ Complete observability (logs, metrics, traces)
