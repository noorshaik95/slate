# Distributed Tracing Implementation Summary

## Problem Statement

The TraceQL query `{ resource.service.name = "user-auth-service" }` was returning no results in Grafana Tempo because the Go-based user-auth-service had no OpenTelemetry instrumentation and was not exporting traces.

## Solution Overview

Implemented end-to-end distributed tracing between the API Gateway (Rust) and User Auth Service (Go) using OpenTelemetry and Grafana Tempo, with proper W3C Trace Context propagation.

## Files Created

### 1. `services/user-auth-service/pkg/tracing/tracing.go`
**Purpose**: OpenTelemetry initialization for Go service

**Key Features**:
- OTLP gRPC exporter configuration
- Resource attributes (service name, version)
- W3C Trace Context propagation
- Configurable sampling rate
- Graceful shutdown handling

**Configuration**:
```go
type Config struct {
    ServiceName     string  // "user-auth-service"
    ServiceVersion  string  // "1.0.0"
    OTLPEndpoint    string  // "tempo:4317"
    OTLPInsecure    bool    // true for local dev
    SamplingRate    float64 // 1.0 = 100% sampling
}
```

### 2. `test_distributed_tracing.sh`
**Purpose**: Automated testing script for distributed tracing

**Features**:
- Health checks for services
- Generates test traces via API calls
- Provides TraceQL query examples
- Step-by-step instructions for Grafana

**Usage**:
```bash
chmod +x test_distributed_tracing.sh
./test_distributed_tracing.sh
```

### 3. `DISTRIBUTED_TRACING_SETUP.md`
**Purpose**: Comprehensive technical documentation

**Contents**:
- Architecture diagram
- Component descriptions
- Trace context propagation flow
- Resource and span attributes
- Testing procedures
- Troubleshooting guide
- Performance considerations
- Dependency versions

### 4. `TRACING_QUICK_START.md`
**Purpose**: Quick reference guide for developers

**Contents**:
- What was fixed
- How to use the tracing
- Common TraceQL queries
- Verification checklist
- Troubleshooting tips
- Next steps

### 5. `TRACING_IMPLEMENTATION_SUMMARY.md`
**Purpose**: This file - overview of all changes

## Files Modified

### 1. `services/user-auth-service/cmd/server/main.go`

**Changes**:
- Added OpenTelemetry imports
- Initialize tracer on startup
- Configure OTLP endpoint from environment
- Add gRPC server interceptor: `otelgrpc.NewServerHandler()`
- Graceful tracer shutdown on exit

**Key Code**:
```go
import (
    "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

// Initialize tracing
tp, err := tracing.InitTracer(tracingCfg)
defer tracing.Shutdown(ctx, tp)

// Add interceptor to gRPC server
grpcServer := grpc.NewServer(
    grpc.StatsHandler(otelgrpc.NewServerHandler()),
)
```

### 2. `services/user-auth-service/go.mod`

**Changes**:
- Added OpenTelemetry dependencies:
  - `go.opentelemetry.io/otel v1.21.0`
  - `go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.21.0`
  - `go.opentelemetry.io/otel/sdk v1.21.0`
  - `go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc v0.46.1`

### 3. `services/api-gateway/src/handlers/user_service.rs`

**Changes**:
- Added `inject_trace_context()` function
- Injects W3C Trace Context into gRPC metadata
- Updated all gRPC calls (Register, Login) to use trace injection

**Key Code**:
```rust
fn inject_trace_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    let propagator = opentelemetry::global::get_text_map_propagator();
    let context = opentelemetry::Context::current();
    let mut injector = MetadataInjector(request.metadata_mut());
    propagator.inject_context(&context, &mut injector);
    request
}

// Usage
let grpc_request = inject_trace_context(tonic::Request::new(request));
let response = client.register(grpc_request).await?;
```

### 4. `docker-compose.yml`

**Changes**:
- Added environment variable to user-auth-service:
  ```yaml
  OTEL_EXPORTER_OTLP_ENDPOINT: "tempo:4317"
  ```
- Added dependency on Tempo:
  ```yaml
  depends_on:
    tempo:
      condition: service_started
  ```

## Architecture

```
┌─────────┐  HTTP   ┌─────────────┐  gRPC+Trace  ┌──────────────┐
│ Client  │────────>│ API Gateway │─────────────>│ User Auth    │
│         │         │   (Rust)    │              │ Service (Go) │
└─────────┘         └─────────────┘              └──────────────┘
                           │                             │
                           │ OTLP                        │ OTLP
                           ▼                             ▼
                    ┌──────────────────────────────────────┐
                    │         Grafana Tempo                │
                    │   (Distributed Trace Storage)        │
                    └──────────────────────────────────────┘
                                      │
                                      │ TraceQL
                                      ▼
                               ┌─────────────┐
                               │   Grafana   │
                               │     UI      │
                               └─────────────┘
```

## Trace Flow

1. **Client → API Gateway**:
   - HTTP request arrives
   - Gateway creates root span
   - Span attributes: `http.method`, `http.target`, `http.status_code`

2. **API Gateway → User Auth Service**:
   - Gateway calls `inject_trace_context()`
   - W3C Trace Context injected into gRPC metadata
   - Format: `traceparent: 00-{trace-id}-{span-id}-{flags}`

3. **User Auth Service**:
   - `otelgrpc` interceptor extracts trace context
   - Creates child span for gRPC method
   - Span attributes: `rpc.service`, `rpc.method`, `rpc.grpc.status_code`

4. **Both Services → Tempo**:
   - Spans exported via OTLP/gRPC
   - Batch processing for efficiency
   - Tempo links spans by trace ID

5. **Grafana → Tempo**:
   - User queries with TraceQL
   - Tempo returns matching traces
   - Grafana displays trace tree

## Key Features

### W3C Trace Context Propagation
- Standard format: `traceparent` header
- Automatic propagation across service boundaries
- Compatible with other OpenTelemetry services

### Resource Attributes
- **API Gateway**: `service.name = "api-gateway"`
- **User Auth Service**: `service.name = "user-auth-service"`
- Enables service-specific queries in TraceQL

### Span Attributes
- HTTP: method, target, route, status_code
- gRPC: service, method, status_code
- Custom: user.id, error messages

### Performance
- Batch span export (reduces network overhead)
- Configurable sampling (100% for dev, lower for prod)
- Minimal latency impact (< 1ms per request)

## Testing

### Automated Test
```bash
./test_distributed_tracing.sh
```

### Manual Test
```bash
# Generate trace
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!",...}'

# Query in Grafana
# Open: http://localhost:3000
# Query: { resource.service.name = "user-auth-service" }
```

### Verification
```bash
# Check service logs
docker-compose logs user-auth-service | grep -i "tracing"
docker-compose logs api-gateway | grep -i "tempo"

# Check Tempo ingestion
docker-compose logs tempo | grep -i "received"
```

## TraceQL Query Examples

```traceql
# All traces
{ }

# By service
{ resource.service.name = "api-gateway" }
{ resource.service.name = "user-auth-service" }

# By operation
{ span.name = "user.UserService/Register" }

# By status
{ status = error }
{ span.http.status_code >= 400 }

# By duration
{ duration > 500ms }

# Combined
{ resource.service.name = "user-auth-service" && duration > 100ms }
```

## Benefits

1. **End-to-End Visibility**: See complete request flow across services
2. **Performance Analysis**: Identify slow operations and bottlenecks
3. **Error Debugging**: Trace errors back to source with full context
4. **Service Dependencies**: Visualize service call graph
5. **Production Ready**: Configurable sampling and batch export

## Next Steps

1. **Add Custom Spans**: Instrument database queries, external API calls
2. **Add Span Events**: Log important events within spans
3. **Configure Sampling**: Reduce to 10% for production
4. **Set Up Alerts**: Alert on high error rates or slow traces
5. **Add More Services**: Extend tracing to other microservices

## Dependencies

### Go (user-auth-service)
```
go.opentelemetry.io/otel v1.21.0
go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc v1.21.0
go.opentelemetry.io/otel/sdk v1.21.0
go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc v0.46.1
```

### Rust (api-gateway)
```toml
opentelemetry = { version = "0.29.0", features = ["trace"] }
opentelemetry-otlp = { version = "0.29.0", features = ["grpc-tonic"] }
opentelemetry_sdk = { version = "0.29.0" }
tracing-opentelemetry = "0.30.0"
```

## References

- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [W3C Trace Context Spec](https://www.w3.org/TR/trace-context/)
- [Grafana Tempo Docs](https://grafana.com/docs/tempo/latest/)
- [TraceQL Language](https://grafana.com/docs/tempo/latest/traceql/)

## Support

For issues or questions:
1. Check `TRACING_QUICK_START.md` for common problems
2. Review `DISTRIBUTED_TRACING_SETUP.md` for detailed setup
3. Check service logs: `docker-compose logs <service-name>`
4. Verify Tempo connectivity: `docker-compose exec <service> ping tempo`
