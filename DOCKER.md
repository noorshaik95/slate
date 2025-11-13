# Docker Setup for API Gateway

This document describes the Docker configuration for the API Gateway.

## Services

### Gateway (Port 8080)
The main API Gateway service built from the Rust application.

**Environment Variables:**
- `RUST_LOG` - Log level (default: info,debug)
- `GATEWAY_SERVER_HOST` - Server host (default: 0.0.0.0)
- `GATEWAY_SERVER_PORT` - Server port (default: 8080)
- `GATEWAY_AUTH_SERVICE_ENDPOINT` - Auth service endpoint
- `GATEWAY_OBSERVABILITY_TEMPO_ENDPOINT` - Tempo tracing endpoint

### Mock Services
- **mock-auth-service** (Port 50051) - Mock authentication service
- **mock-backend-service** (Port 50052) - Mock backend service with auth policies

### Observability Stack
- **Prometheus** (Port 9090) - Metrics collection
- **Tempo** (Port 4317) - Distributed tracing
- **Loki** (Port 3100) - Log aggregation
- **Grafana** (Port 3000) - Visualization dashboard
- **Promtail** - Log shipping to Loki

## Building and Running

### Build the Gateway
```bash
docker-compose build gateway
```

### Start All Services
```bash
docker-compose up
```

### Start in Detached Mode
```bash
docker-compose up -d
```

### View Logs
```bash
# All services
docker-compose logs -f

# Gateway only
docker-compose logs -f gateway

# Mock services
docker-compose logs -f mock-auth-service mock-backend-service
```

### Stop Services
```bash
docker-compose down
```

### Clean Up (including volumes)
```bash
docker-compose down -v
```

## Configuration

### Gateway Configuration
The gateway configuration is mounted from `config/gateway-config.yaml` and can be modified without rebuilding the image.

### Mock Service Configuration
Mock service responses are defined in:
- `mock-services/auth-service.yaml` - Auth service responses
- `mock-services/backend-service.yaml` - Backend service responses

See `mock-services/README.md` for details on test tokens and policies.

## Dockerfile Details

The Dockerfile uses a multi-stage build:

1. **Build Stage**: 
   - Installs protobuf compiler
   - Compiles proto files via build.rs
   - Builds the Rust application in release mode

2. **Runtime Stage**:
   - Uses minimal debian:bookworm-slim base
   - Copies compiled binary and configuration
   - Exposes port 8080

## Accessing Services

- Gateway: http://localhost:8080
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin/admin)
- Tempo: http://localhost:3200

## Troubleshooting

### Proto Compilation Errors
If you see proto compilation errors during build, ensure:
- Proto files exist in `proto/` directory
- `build.rs` is properly configured
- protobuf-compiler is installed in the builder stage

### Connection Errors
If the gateway cannot connect to mock services:
- Ensure all services are running: `docker-compose ps`
- Check service logs: `docker-compose logs mock-auth-service`
- Verify network connectivity: `docker-compose exec gateway ping mock-auth-service`

### Configuration Not Loading
If configuration changes aren't reflected:
- Restart the gateway: `docker-compose restart gateway`
- Check volume mount: `docker-compose exec gateway ls -la /app/config/`
- Verify YAML syntax in configuration files
