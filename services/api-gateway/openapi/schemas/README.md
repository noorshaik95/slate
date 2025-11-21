# OpenAPI Schemas Directory

This directory contains modular schema definitions for the Slate LMS API, organized by service.

## Structure

```
schemas/
└── user-auth-schemas.yaml    # User Auth Service schemas from proto/user.proto
```

## user-auth-schemas.yaml

Contains all schema definitions for the User Auth Service, generated from `proto/user.proto`.

### Authentication Schemas
- **LoginRequest** - User login credentials
- **LoginResponse** - Login response with tokens and user info
- **RegisterRequest** - New user registration data
- **RegisterResponse** - Registration response with tokens
- **RefreshTokenRequest** - Token refresh request
- **RefreshTokenResponse** - New access token response
- **ValidateTokenRequest** - Token validation request
- **ValidateTokenResponse** - Token validation result

### OAuth Schemas
- **OAuthProviderInfo** - OAuth provider information
- **OAuthAuthRequest** - OAuth authorization request
- **OAuthAuthResponse** - OAuth authorization URL response
- **OAuthCallbackRequest** - OAuth callback parameters
- **OAuthCallbackResponse** - OAuth callback response with tokens

### SAML Schemas
- **SAMLAuthRequest** - SAML authentication request
- **SAMLAuthResponse** - SAML authentication response
- **SAMLAssertionRequest** - SAML assertion from IdP
- **SAMLMetadataRequest** - Service Provider metadata request
- **SAMLMetadataResponse** - Service Provider metadata XML

### MFA Schemas
- **SetupMFARequest** - MFA setup request
- **SetupMFAResponse** - MFA setup response with secret and QR code
- **MFAStatus** - MFA configuration status
- **ValidateMFACodeRequest** - MFA code validation request
- **ValidateMFACodeResponse** - MFA code validation result

### Group Schemas
- **Group** - User group information
- **GroupMember** - Group member details
- **CreateGroupRequest** - Create new group request
- **UpdateGroupRequest** - Update group request
- **ListGroupsRequest** - List groups with pagination
- **ListGroupsResponse** - List of groups response

### Parent-Child Relationship Schemas
- **ParentChildRelationship** - Parent-child account relationship with permissions

## Usage

These schemas are referenced in the main `openapi.yaml` file using `$ref`:

```yaml
components:
  schemas:
    LoginRequest:
      $ref: './schemas/user-auth-schemas.yaml#/LoginRequest'
```

## Adding New Schemas

When adding new schemas:

1. Create a new YAML file in this directory (e.g., `course-schemas.yaml`)
2. Define your schemas following the OpenAPI 3.0 specification
3. Reference schemas from the main `openapi.yaml` file
4. Update this README with the new schema file documentation
5. Validate the complete specification:
   ```bash
   npx @apidevtools/swagger-cli validate services/api-gateway/openapi/openapi.yaml
   ```

## Schema References

Schemas in this directory can reference:
- Other schemas in the same file: `$ref: '#/SchemaName'`
- Schemas in the main file: `$ref: '../openapi.yaml#/components/schemas/SchemaName'`
- Schemas in other schema files: `$ref: './other-schemas.yaml#/SchemaName'`

## Proto to OpenAPI Mapping

These schemas are generated from protobuf definitions with the following type mappings:

| Proto Type | OpenAPI Type | Format |
|------------|--------------|--------|
| string | string | - |
| int32, int64 | integer | int32, int64 |
| bool | boolean | - |
| google.protobuf.Timestamp | string | date-time |
| repeated T | array | items: {type: T} |
| map<K,V> | object | additionalProperties: {type: V} |
| enum | string | enum: [values] |
| message | object | $ref or inline |

## Validation

All schemas are validated as part of the complete OpenAPI specification. Run validation with:

```bash
npx @apidevtools/swagger-cli validate services/api-gateway/openapi/openapi.yaml
```
