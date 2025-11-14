# API Documentation

This directory contains the OpenAPI 3.0 specification for the User Authentication Service API.

## Viewing the Documentation

### Local Development

When the API Gateway is running, you can view the interactive API documentation at:

```
http://localhost:8080/docs
```

This will display a Swagger UI interface where you can:
- Browse all available endpoints
- View request/response schemas
- Try out API calls directly from the browser
- See authentication requirements
- View error response formats

### Direct OpenAPI Spec Access

The raw OpenAPI specification is available at:

```
http://localhost:8080/docs/openapi.yaml
```

## Generating the OpenAPI Spec

To regenerate the OpenAPI specification from proto files:

```bash
./scripts/generate-openapi.sh
```

### Prerequisites

- `protoc` - Protocol Buffer compiler
- `protoc-gen-openapi` - OpenAPI generator plugin for protoc

Install protoc-gen-openapi:
```bash
go install github.com/google/gnostic/cmd/protoc-gen-openapi@latest
```

## API Overview

### Authentication

Most endpoints require authentication using Bearer tokens:

```
Authorization: Bearer <access_token>
```

Get tokens by calling:
- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/register` - Register new account

### Rate Limiting

- **Login**: 5 attempts per 15 minutes per IP
- **Register**: 3 attempts per hour per IP
- **Other endpoints**: 60 requests per minute per IP

### Endpoint Categories

1. **Authentication** (`/api/auth/*`)
   - Login, Register, Refresh Token, Logout

2. **Users** (`/api/users/*`)
   - CRUD operations (admin only)
   - List, Create, Get, Update, Delete users

3. **Profile** (`/api/profile/*`)
   - Get and update user profile
   - Change password

4. **Roles** (`/api/users/{user_id}/roles/*`)
   - Assign and remove roles
   - Check permissions

### Error Responses

All errors follow this format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": []
}
```

Common error codes:
- `UNAUTHORIZED` - Authentication required or invalid token
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `VALIDATION_ERROR` - Invalid request parameters
- `RATE_LIMIT_EXCEEDED` - Too many requests

## Using the API

### Example: Login

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePass123!"
  }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "first_name": "John",
    "last_name": "Doe",
    "roles": ["user"],
    "is_active": true
  },
  "expires_in": 900
}
```

### Example: Get Profile (Authenticated)

```bash
curl -X GET http://localhost:8080/api/profile \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
```

### Example: List Users (Admin Only)

```bash
curl -X GET "http://localhost:8080/api/users?page=1&page_size=20" \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIs..."
```

## Updating the Documentation

When you add new endpoints or modify existing ones:

1. Update the proto files in `proto/`
2. Run `./scripts/generate-openapi.sh` to regenerate the spec
3. Or manually update `openapi.yaml` if not using proto generation
4. The changes will be automatically available at `/docs` when the gateway restarts

## Integration with Tools

### Postman

Import the OpenAPI spec into Postman:
1. Open Postman
2. Click "Import"
3. Enter URL: `http://localhost:8080/docs/openapi.yaml`
4. Or upload the `openapi.yaml` file directly

### Insomnia

Import the OpenAPI spec into Insomnia:
1. Open Insomnia
2. Click "Create" → "Import From" → "URL"
3. Enter: `http://localhost:8080/docs/openapi.yaml`

### Code Generation

Generate client SDKs using OpenAPI Generator:

```bash
# Generate TypeScript client
openapi-generator-cli generate \
  -i http://localhost:8080/docs/openapi.yaml \
  -g typescript-axios \
  -o ./clients/typescript

# Generate Python client
openapi-generator-cli generate \
  -i http://localhost:8080/docs/openapi.yaml \
  -g python \
  -o ./clients/python
```

## Troubleshooting

### Documentation not loading

1. Ensure the gateway is running: `docker-compose up api-gateway`
2. Check the OpenAPI file exists: `ls services/api-gateway/openapi/openapi.yaml`
3. Check gateway logs for errors

### Swagger UI not displaying

1. Check browser console for JavaScript errors
2. Ensure you can access: `http://localhost:8080/docs/openapi.yaml`
3. Verify the YAML file is valid: `yamllint openapi.yaml`

### API calls failing from Swagger UI

1. Ensure you're authenticated (click "Authorize" button)
2. Enter your Bearer token in the format: `Bearer <token>`
3. Check CORS settings if calling from different origin

## Additional Resources

- [OpenAPI Specification](https://swagger.io/specification/)
- [Swagger UI Documentation](https://swagger.io/tools/swagger-ui/)
- [gRPC to OpenAPI](https://github.com/google/gnostic)
