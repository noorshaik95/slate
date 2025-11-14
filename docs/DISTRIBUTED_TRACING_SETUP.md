# Distributed Tracing Setup

This document explains the distributed tracing implementation that connects the API Gateway (Rust) with the User Auth Service (Go) using OpenTelemetry and Grafana Tempo.

## Architecture

```
┌─────────────┐      HTTP       ┌──────────────┐      gRPC       ┌──────────────────┐
│   Client    │ ──────────────> │ API Gateway  │ ──────────────> │ User Auth Service│
│             │                 │   (Rust)     │                 │      (Go)        │
└─────────────┘                 └──────────────┘                 └──────────────────┘
                                       │                                   │
                                       │ OTLP                              │ OTLP
                                       ▼                                   ▼
                                ┌──────────────────────────────────────────┐
                                │         Grafana Tempo                    │
                                │    (Trace Storage & Query)               │
                                └──────────────────────────────────────────┘
                                                   │
                                                   │ Query
                                                   ▼
                                            ┌─────────────┐
                                            │   Grafana   │
                                            │     UI      │
                                            └─────────────┘
```

## Components

### 1. API Gateway (Rust/Axum)

**Location**: `services/api-gateway/`

**Tracing Implementation**:
- Uses `opentelemetry` and `opentelemetry-otlp` crates
- Exports traces to Tempo via OTLP/gRPC on port 4317
- Service name: `api-gateway`
- Propagates W3C Trace Context headers to downstream services

**Key Files**:
- `src/main.rs`: Initializes OpenTelemetry tracer provider
- `src/handlers/user_service.rs`: Injects trace context into gRPC calls
- `src/observability/tracing_utils.rs`: Trace context utilities

**Configuration** (docker-compose.yml):
```yaml
environment:
  GATEWAY_OBSERVABILITY_TEMPO_ENDPOINT: "http://tempo:4317"
```

### 2. User Auth Service (Go)

**Location**: `services/user-auth-service/`

**Tracing Implementation**:
- Uses OpenTelemetry Go SDK
- Exports traces to Tempo via OTLP/gRPC on port 4317
- Service name: `user-auth-service`
- gRPC server instrumented with `otelgrpc` interceptors

**Key Files**:
- `pkg/tracing/tracing.go`: OpenTelemetry initialization
- `cmd/server/main.go`: Tracer setup and gRPC interceptors

**Configuration** (docker-compose.yml):
```yaml
environment:
  OTEL_EXPORTER_OTLP_ENDPOINT: "tempo:4317"
```

### 3. Grafana Tempo

**Location**: `config/tempo.yaml`

**Configuration**:
- Receives traces via OTLP on ports 4317 (gRPC) and 4318 (HTTP)
- Stores traces locally in `/tmp/tempo/blocks`
- 24-hour retention period
- Accessible on port 3200

## Trace Context Propagation

### W3C Trace Context Format

Both services use the W3C Trace Context standard for propagation:

```
traceparent: 00-{trace-id}-{span-id}-{trace-flags}
```

Example:
```
traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01
```

### Flow

1. **Client → API Gateway**:
   - Client sends HTTP request (may include `traceparent` header)
   - Gateway extracts or creates trace context
   - Gateway creates root span

2. **API Gateway → User Auth Service**:
   - Gateway injects trace context into gRPC metadata
   - Uses `inject_trace_context()` function
   - Propagates via gRPC metadata headers

3. **User Auth Service**:
   - gRPC interceptor extracts trace context from metadata
   - Creates child spans for service operations
   - Exports spans to Tempo

4. **Tempo**:
   - Receives spans from both services
   - Links spans using trace ID
   - Stores complete trace tree

## Resource Attributes

### API Gateway
```
resource.service.name = "api-gateway"
```

### User Auth Service
```
resource.service.name = "user-auth-service"
resource.service.version = "1.0.0"
```

## Span Attributes

### HTTP Spans (API Gateway)
- `http.method`: HTTP method (GET, POST, etc.)
- `http.target`: Request path
- `http.route`: Route pattern
- `http.status_code`: Response status
- `user.id`: Authenticated user ID

### gRPC Spans (Both Services)
- `rpc.system`: "grpc"
- `rpc.service`: Service name (e.g., "user.UserService")
- `rpc.method`: Method name (e.g., "Register")
- `rpc.grpc.status_code`: gRPC status code

## Testing

### 1. Build and Start Services

```bash
docker-compose up --build
```

### 2. Generate Traces

Run the test script:
```bash
./test_distributed_tracing.sh
```

Or manually:
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "TestPassword123!",
    "first_name": "Test",
    "last_name": "User",
    "phone": "+1234567890"
  }'
```

### 3. View Traces in Grafana

1. Open Grafana: http://localhost:3000
2. Navigate to Explore (compass icon)
3. Select "Tempo" as data source
4. Use TraceQL queries:

**Find all traces**:
```traceql
{ }
```

**Find traces from API Gateway**:
```traceql
{ resource.service.name = "api-gateway" }
```

**Find traces from User Auth Service**:
```traceql
{ resource.service.name = "user-auth-service" }
```

**Find traces for specific method**:
```traceql
{ span.name = "user.UserService/Register" }
```

**Find traces with errors**:
```traceql
{ status = error }
```

### 4. Analyze Trace

When you click on a trace, you'll see:
- **Trace Timeline**: Visual representation of span durations
- **Span Tree**: Hierarchical view of parent-child relationships
- **Span Details**: Attributes, events, and metadata
- **Service Graph**: Visual map of service dependencies

Example trace structure:
```
api-gateway: gateway_request (200ms)
  └─ api-gateway: grpc.user.Register (180ms)
      └─ user-auth-service: user.UserService/Register (170ms)
          ├─ user-auth-service: db.query (50ms)
          └─ user-auth-service: password.hash (100ms)
```

## Troubleshooting

### No traces appearing in Tempo

1. **Check service logs**:
   ```bash
   docker-compose logs api-gateway | grep -i "tracing\|tempo"
   docker-compose logs user-auth-service | grep -i "tracing\|otel"
   ```

2. **Verify Tempo is receiving traces**:
   ```bash
   docker-compose logs tempo
   ```

3. **Check connectivity**:
   ```bash
   docker-compose exec api-gateway ping tempo
   docker-compose exec user-auth-service ping tempo
   ```

### Traces not connected between services

1. **Verify trace context propagation**:
   - Check API Gateway logs for "Injected trace header"
   - Check User Auth Service logs for trace IDs

2. **Verify gRPC interceptors**:
   - Ensure `otelgrpc.NewServerHandler()` is configured
   - Check that `inject_trace_context()` is called

### Query returns no results

1. **Wait for trace ingestion** (5-10 seconds)
2. **Expand time range** in Grafana
3. **Verify service names** match exactly:
   - `api-gateway` (not `api_gateway`)
   - `user-auth-service` (not `user_auth_service`)

## Performance Considerations

### Sampling

Currently set to 100% sampling (all traces):
- **API Gateway**: `SamplingRate: 1.0`
- **User Auth Service**: `SamplingRate: 1.0`

For production, reduce sampling:
```go
// Go service
SamplingRate: 0.1  // 10% of traces
```

```rust
// Rust service - modify tracer provider builder
.with_sampler(opentelemetry_sdk::trace::Sampler::TraceIdRatioBased(0.1))
```

### Batch Export

Both services use batch span processors:
- Reduces network overhead
- Configurable batch size and timeout
- Automatic retry on failure

### Resource Usage

Typical overhead per service:
- CPU: < 1%
- Memory: ~10-20 MB
- Network: ~1-5 KB per trace

## Dependencies

### API Gateway (Rust)
```toml
opentelemetry = { version = "0.29.0", features = ["trace"] }
opentelemetry-otlp = { version = "0.29.0", features = ["grpc-tonic"] }
opentelemetry_sdk = { version = "0.29.0" }
tracing-opentelemetry = "0.30.0"
```

### User Auth Service (Go)
```go
go.opentelemetry.io/otel v1.21.0
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.21.0
go.opentelemetry.io/otel/sdk v1.21.0
go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc v0.46.1
```

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [Grafana Tempo Documentation](https://grafana.com/docs/tempo/latest/)
- [TraceQL Query Language](https://grafana.com/docs/tempo/latest/traceql/)
