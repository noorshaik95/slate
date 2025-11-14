#!/bin/bash

# Script to generate OpenAPI 3.0 specification from proto files
# This script uses protoc with the openapi plugin to generate API documentation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== OpenAPI Specification Generator ===${NC}"

# Check if protoc is installed
if ! command -v protoc &> /dev/null; then
    echo -e "${RED}Error: protoc is not installed${NC}"
    echo "Please install protoc: https://grpc.io/docs/protoc-installation/"
    exit 1
fi

# Check if protoc-gen-openapi is installed
if ! command -v protoc-gen-openapi &> /dev/null; then
    echo -e "${YELLOW}Warning: protoc-gen-openapi not found${NC}"
    echo "Installing protoc-gen-openapi..."
    go install github.com/google/gnostic/cmd/protoc-gen-openapi@latest
    
    if ! command -v protoc-gen-openapi &> /dev/null; then
        echo -e "${RED}Error: Failed to install protoc-gen-openapi${NC}"
        echo "Please ensure \$GOPATH/bin is in your PATH"
        exit 1
    fi
fi

# Create output directory
OUTPUT_DIR="services/api-gateway/openapi"
mkdir -p "$OUTPUT_DIR"

echo -e "${GREEN}Generating OpenAPI specification from proto files...${NC}"

# Generate OpenAPI spec from user.proto
protoc \
    --proto_path=proto \
    --openapi_out="$OUTPUT_DIR" \
    --openapi_opt=naming=json \
    --openapi_opt=output_mode=merged \
    proto/user.proto

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Generated OpenAPI spec from user.proto${NC}"
else
    echo -e "${RED}✗ Failed to generate OpenAPI spec from user.proto${NC}"
    exit 1
fi

# Post-process the generated OpenAPI spec to add authentication
OPENAPI_FILE="$OUTPUT_DIR/openapi.yaml"

if [ -f "$OPENAPI_FILE" ]; then
    echo -e "${GREEN}Post-processing OpenAPI specification...${NC}"
    
    # Create a temporary file with authentication configuration
    cat > "$OUTPUT_DIR/openapi_enhanced.yaml" << 'EOF'
openapi: 3.0.3
info:
  title: User Authentication Service API
  description: |
    RESTful API for user authentication and management.
    
    ## Authentication
    Most endpoints require authentication using Bearer tokens.
    Include the access token in the Authorization header:
    ```
    Authorization: Bearer <access_token>
    ```
    
    ## Rate Limiting
    - Login: 5 attempts per 15 minutes per IP
    - Register: 3 attempts per hour per IP
    - Other endpoints: 60 requests per minute per IP
    
    ## Error Responses
    All error responses follow this format:
    ```json
    {
      "error": "Error message",
      "code": "ERROR_CODE"
    }
    ```
  version: 1.0.0
  contact:
    name: API Support
    email: support@example.com
  license:
    name: MIT
    url: https://opensource.org/licenses/MIT

servers:
  - url: http://localhost:8080
    description: Local development server
  - url: https://api.example.com
    description: Production server

security:
  - BearerAuth: []

components:
  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: |
        JWT access token obtained from /api/v1/auth/login or /api/v1/auth/register.
        Tokens expire after 15 minutes by default.

  responses:
    UnauthorizedError:
      description: Authentication required or token invalid/expired
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Authentication required"
              code:
                type: string
                example: "UNAUTHORIZED"
    
    ForbiddenError:
      description: Insufficient permissions
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Permission denied"
              code:
                type: string
                example: "FORBIDDEN"
    
    NotFoundError:
      description: Resource not found
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Resource not found"
              code:
                type: string
                example: "NOT_FOUND"
    
    RateLimitError:
      description: Rate limit exceeded
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Rate limit exceeded"
              code:
                type: string
                example: "RATE_LIMIT_EXCEEDED"
              retry_after:
                type: integer
                description: Seconds until rate limit resets
                example: 300
    
    ValidationError:
      description: Invalid request parameters
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
                example: "Invalid request parameters"
              code:
                type: string
                example: "VALIDATION_ERROR"
              details:
                type: array
                items:
                  type: object
                  properties:
                    field:
                      type: string
                    message:
                      type: string

tags:
  - name: Authentication
    description: User authentication and token management
  - name: Users
    description: User CRUD operations (admin only)
  - name: Profile
    description: User profile management
  - name: Roles
    description: Role and permission management (admin only)

EOF

    # Merge the generated spec with our enhancements
    # Note: This is a simplified merge. In production, use a proper YAML merge tool
    cat "$OPENAPI_FILE" >> "$OUTPUT_DIR/openapi_enhanced.yaml"
    mv "$OUTPUT_DIR/openapi_enhanced.yaml" "$OPENAPI_FILE"
    
    echo -e "${GREEN}✓ Enhanced OpenAPI spec with authentication and error responses${NC}"
else
    echo -e "${RED}✗ OpenAPI file not found at $OPENAPI_FILE${NC}"
    exit 1
fi

# Generate a JSON version as well
if command -v yq &> /dev/null; then
    yq eval -o=json "$OPENAPI_FILE" > "$OUTPUT_DIR/openapi.json"
    echo -e "${GREEN}✓ Generated JSON version: $OUTPUT_DIR/openapi.json${NC}"
else
    echo -e "${YELLOW}Note: Install 'yq' to generate JSON version${NC}"
fi

echo -e "${GREEN}=== OpenAPI Generation Complete ===${NC}"
echo -e "OpenAPI spec location: ${YELLOW}$OPENAPI_FILE${NC}"
echo -e "View documentation at: ${YELLOW}http://localhost:8080/docs${NC} (after starting the gateway)"

exit 0
