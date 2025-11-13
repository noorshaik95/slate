# Mock gRPC Services

This directory contains mock gRPC service configurations for testing the API Gateway.

## Services

### Mock Auth Service (Port 50051)
Provides mock authentication and authorization responses.

**Valid Test Tokens:**
- `valid-token-123` - Returns user-123 with roles: user, admin
- `valid-user-token` - Returns user-456 with role: user
- `expired-token` - Returns invalid token error
- Any other token - Returns invalid token error

### Mock Backend Service (Port 50052)
Provides mock backend service responses and authorization policies.

**Authorization Policies:**
- `user.UserService/GetPublicStatus` - No auth required
- `user.UserService/ListUsers` - Auth required, no specific roles
- `user.UserService/GetUser` - Auth required, admin role required
- Other methods - Auth required by default

## Usage

These mock services are automatically started with `docker-compose up` and are used by the gateway for testing.

## Testing Examples

```bash
# Test public endpoint (no auth)
curl http://localhost:8080/api/public/status

# Test authenticated endpoint
curl -H "Authorization: Bearer valid-token-123" http://localhost:8080/api/users

# Test admin-only endpoint
curl -H "Authorization: Bearer valid-token-123" http://localhost:8080/api/users/user-123
```
