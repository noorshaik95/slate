# OpenAPI Path Files

This directory contains modular OpenAPI path definitions that are referenced by the main `openapi.yaml` file.

## Structure

Each file contains related endpoint definitions:

- **course-templates.yaml** - Course template management endpoints
  - `POST /api/coursetemplates` - Create course template
  - `GET /api/coursetemplates` - List course templates
  - `GET /api/coursetemplates/{template_id}` - Get course template
  - `POST /api/coursetemplates/{template_id}/courses` - Create course from template

- **course-prerequisites.yaml** - Course prerequisite management endpoints
  - `POST /api/courses/{course_id}/prerequisites` - Add prerequisite
  - `DELETE /api/courses/{course_id}/prerequisites/{prerequisite_id}` - Remove prerequisite
  - `POST /api/courses/{course_id}/prerequisites/check` - Check prerequisites

- **course-co-instructors.yaml** - Course co-instructor management endpoints
  - `POST /api/courses/{course_id}/co-instructors` - Add co-instructor
  - `DELETE /api/courses/{course_id}/co-instructors/{co_instructor_id}` - Remove co-instructor

## Usage

These files are referenced in the main `openapi.yaml` using JSON Pointer syntax:

```yaml
paths:
  /api/coursetemplates:
    $ref: './paths/course-templates.yaml#/~1api~1coursetemplates'
```

The `~1` encoding represents `/` in JSON Pointer notation.

## Adding New Endpoints

When adding new endpoints:

1. Create a new YAML file in this directory (e.g., `course-sections.yaml`)
2. Define your paths using the same structure as existing files
3. Reference them in the main `openapi.yaml` file
4. Update this README with the new endpoints

## Schema References

All path files reference schemas from the main `openapi.yaml` file using:

```yaml
$ref: '../openapi.yaml#/components/schemas/SchemaName'
```

## Validation

To validate the complete OpenAPI specification with all references:

```bash
# Using swagger-cli (if installed)
swagger-cli validate services/api-gateway/openapi/openapi.yaml

# Using openapi-generator-cli (if installed)
openapi-generator-cli validate -i services/api-gateway/openapi/openapi.yaml
```
