# API Gateway

A high-performance API Gateway built with Rust and Axum that provides HTTP-to-gRPC translation, authentication, rate limiting, and observability for microservices.

## Features

- **Auto-Discovery**: Automatically discovers gRPC services and generates HTTP routes
- **Route Overrides**: Custom route mappings for non-standard endpoints
- **Multi-Auth Support**: OAuth 2.0 and SAML 2.0 authentication alongside traditional auth
- **Rate Limiting**: Per-IP rate limiting with Redis backend
- **Circuit Breaker**: Automatic failure detection and recovery
- **Observability**: OpenTelemetry tracing with Tempo integration
- **Connection Pooling**: Efficient gRPC connection management

## Configuration

### Environment Variables

```bash
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Authentication Type
# Should match the AUTH_TYPE configured in user-auth-service
# Valid values: normal, oauth, saml
AUTH_TYPE=normal

# Service Endpoints
USER_AUTH_SERVICE_ENDPOINT=http://user-auth-service:50051

# Observability
TEMPO_ENDPOINT=http://tempo:4317
SERVICE_NAME=api-gateway

# Rate Limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60

# Request Body Limits
MAX_REQUEST_BODY_SIZE=1048576      # 1MB default
MAX_UPLOAD_BODY_SIZE=10485760      # 10MB for uploads
UPLOAD_PATHS=/upload,/api/upload

# Logging
RUST_LOG=info  # or debug, warn, error
```

### Gateway Configuration File

The gateway uses a YAML configuration file (`config/gateway-config.yaml`) for service discovery and route overrides.

#### Basic Configuration

```yaml
server:
  host: "0.0.0.0"
  port: 8080

services:
  user-auth-service:
    name: "user-auth-service"
    endpoint: "http://user-auth-service:50051"
    timeout_ms: 5000
    connection_pool_size: 10
    auto_discover: true
    tls_enabled: false

discovery:
  enabled: true
  refresh_interval_seconds: 300
```

#### Route Overrides for Multi-Auth

When using OAuth 2.0 or SAML 2.0 authentication, you need to configure route overrides for the authentication endpoints:

```yaml
route_overrides:
  # OAuth 2.0 Authentication Routes
  - grpc_method: "user.UserService/GetOAuthAuthorizationURL"
    http_path: "/auth/oauth/authorize"
    http_method: "POST"
    service: "user-auth-service"

  - grpc_method: "user.UserService/HandleOAuthCallback"
    http_path: "/auth/oauth/callback"
    http_method: "GET"
    service: "user-auth-service"

  # SAML 2.0 Authentication Routes
  - grpc_method: "user.UserService/GetSAMLAuthRequest"
    http_path: "/auth/saml/login"
    http_method: "POST"
    service: "user-auth-service"

  - grpc_method: "user.UserService/HandleSAMLAssertion"
    http_path: "/auth/saml/acs"
    http_method: "POST"
    service: "user-auth-service"

  - grpc_method: "user.UserService/GetSAMLMetadata"
    http_path: "/auth/saml/metadata"
    http_method: "GET"
    service: "user-auth-service"
```

#### Public Routes Configuration

Authentication endpoints need to be accessible without authentication. Add them to the public routes list:

```yaml
auth:
  service_endpoint: "http://user-auth-service:50051"
  timeout_ms: 3000
  public_routes:
    # Traditional authentication
    - path: "/api/auth/login"
      method: "POST"
    - path: "/api/auth/register"
      method: "POST"
    - path: "/api/auth/refresh"
      method: "POST"

    # OAuth 2.0 endpoints
    - path: "/auth/oauth/authorize"
      method: "POST"
    - path: "/auth/oauth/callback"
      method: "GET"

    # SAML 2.0 endpoints
    - path: "/auth/saml/login"
      method: "POST"
    - path: "/auth/saml/acs"
      method: "POST"
    - path: "/auth/saml/metadata"
      method: "GET"
```

## Multi-Authentication Support

The API Gateway supports three authentication methods:

### 1. Normal Authentication (Username/Password)

Default authentication method using email and password.

**Endpoints:**
- `POST /api/auth/login` - Login with credentials
- `POST /api/auth/register` - Register new user
- `POST /api/auth/refresh` - Refresh access token

### 2. OAuth 2.0 Authentication

Supports Google, Microsoft, and custom OAuth providers.

**Flow:**
1. Client requests authorization URL: `POST /auth/oauth/authorize`
2. User is redirected to OAuth provider
3. Provider redirects back to: `GET /auth/oauth/callback?code=...&state=...`
4. Gateway exchanges code for tokens and returns JWT

**Configuration:**
Set `AUTH_TYPE=oauth` in both gateway and user-auth-service.

### 3. SAML 2.0 Authentication

Supports Okta, Auth0, ADFS, Shibboleth, and custom SAML providers.

**Flow:**
1. Client requests SAML auth: `POST /auth/saml/login`
2. User is redirected to SAML IdP with SAML request
3. IdP redirects back to: `POST /auth/saml/acs` with SAML assertion
4. Gateway validates assertion and returns JWT

**Metadata Endpoint:**
- `GET /auth/saml/metadata` - Service Provider metadata for IdP configuration

**Configuration:**
Set `AUTH_TYPE=saml` in both gateway and user-auth-service.

## Route Override Behavior

Route overrides take precedence over auto-discovered routes. This allows you to:

1. **Customize HTTP paths** for specific gRPC methods
2. **Change HTTP methods** (e.g., GET instead of POST)
3. **Add routes** that don't follow standard naming conventions

Auto-discovered routes remain available unless explicitly overridden.

## Testing Route Overrides

Use the provided test script to verify route configuration:

```bash
# Start the gateway
docker-compose up api-gateway

# Run the test script
./scripts/test_multi_auth_routes.sh
```

The script tests:
- OAuth authorization and callback endpoints
- SAML login, ACS, and metadata endpoints
- Auto-discovered routes still work

## Building and Running

### Development

```bash
# Build
cargo build

# Run with config file
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run with specific config
CONFIG_PATH=./config/gateway-config.yaml cargo run
```

### Docker

```bash
# Build image
docker build -t api-gateway -f services/api-gateway/Dockerfile .

# Run container
docker run -p 8080:8080 \
  -v $(pwd)/config/gateway-config.yaml:/app/config/gateway-config.yaml \
  -e AUTH_TYPE=normal \
  api-gateway
```

### Docker Compose

```bash
# Start all services
docker-compose up

# Start only gateway and dependencies
docker-compose up api-gateway
```

## Troubleshooting

### Route Override Issues

**Problem:** Routes return 404 Not Found

**Solutions:**
1. Check that `route_overrides` section exists in config file
2. Verify YAML syntax is correct (proper indentation)
3. Ensure `service` field matches the service name in `services` section
4. Check gateway logs for route registration messages
5. Verify the gRPC method name matches exactly (case-sensitive)

**Problem:** Routes work but return wrong responses

**Solutions:**
1. Verify the gRPC method signature matches the HTTP request
2. Check that the service is running and accessible
3. Review service logs for errors
4. Verify request body format matches gRPC message structure

### Authentication Issues

**Problem:** OAuth/SAML routes not accessible

**Solutions:**
1. Ensure routes are in `public_routes` list
2. Verify `AUTH_TYPE` environment variable is set correctly
3. Check that user-auth-service has matching `AUTH_TYPE`
4. Review gateway logs for authentication middleware errors

### Connection Issues

**Problem:** Gateway can't connect to services

**Solutions:**
1. Verify service endpoints in config file
2. Check that services are running: `docker-compose ps`
3. Test service connectivity: `curl http://user-auth-service:50051`
4. Review network configuration in docker-compose.yml
5. Check firewall rules if running outside Docker

### Performance Issues

**Problem:** Slow response times

**Solutions:**
1. Increase `connection_pool_size` in service configuration
2. Adjust `timeout_ms` values
3. Enable connection pooling for all services
4. Review circuit breaker settings
5. Check service health and resource usage

### Observability Issues

**Problem:** Traces not appearing in Tempo

**Solutions:**
1. Verify `TEMPO_ENDPOINT` is correct
2. Check Tempo is running: `docker-compose ps tempo`
3. Ensure OpenTelemetry is initialized in services
4. Review trace sampling configuration
5. Check network connectivity to Tempo

## Monitoring

### Health Check

```bash
curl http://localhost:8080/health
```

### Metrics

Prometheus metrics are exposed on the configured metrics port:

```bash
curl http://localhost:9090/metrics
```

### Logs

View gateway logs:

```bash
# Docker Compose
docker-compose logs -f api-gateway

# Docker
docker logs -f <container-id>
```

### Traces

View traces in Grafana:
1. Open http://localhost:3000
2. Navigate to Explore
3. Select Tempo data source
4. Search for traces by service name: `api-gateway`

## Architecture

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ HTTP
       ▼
┌─────────────────────────────────────┐
│         API Gateway                 │
│  ┌──────────────────────────────┐  │
│  │   Route Discovery            │  │
│  │   - Auto-discover gRPC       │  │
│  │   - Apply overrides          │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │   Middleware Stack           │  │
│  │   - Authentication           │  │
│  │   - Rate Limiting            │  │
│  │   - Tracing                  │  │
│  │   - CORS                     │  │
│  └──────────────────────────────┘  │
│  ┌──────────────────────────────┐  │
│  │   gRPC Client Pool           │  │
│  │   - Connection pooling       │  │
│  │   - Circuit breaker          │  │
│  │   - Load balancing           │  │
│  └──────────────────────────────┘  │
└─────────────┬───────────────────────┘
              │ gRPC
              ▼
┌─────────────────────────────────────┐
│      Microservices                  │
│  - user-auth-service                │
│  - content-management-service       │
│  - ...                              │
└─────────────────────────────────────┘
```

## Contributing

When adding new route overrides:

1. Add the override to `config/gateway-config.yaml`
2. Add the route to `public_routes` if it should be publicly accessible
3. Update this README with the new route
4. Add tests to `scripts/test_multi_auth_routes.sh`
5. Document the endpoint in the API documentation

## License

See the main project LICENSE file.
