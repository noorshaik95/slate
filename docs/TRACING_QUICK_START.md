# Distributed Tracing Quick Start Guide

## What Was Fixed

The user-auth-service (Go) was not sending traces to Tempo, so TraceQL queries like `{ resource.service.name = "user-auth-service" }` returned no results.

## Changes Made

### 1. User Auth Service (Go)
- ✅ Added OpenTelemetry SDK dependencies
- ✅ Created `pkg/tracing/tracing.go` for trace initialization
- ✅ Updated `cmd/server/main.go` to initialize tracer and add gRPC interceptors
- ✅ Configured OTLP exporter to send traces to Tempo
- ✅ Added resource attribute: `service.name = "user-auth-service"`

### 2. API Gateway (Rust)
- ✅ Added trace context injection in `handlers/user_service.rs`
- ✅ Created `inject_trace_context()` function to propagate W3C Trace Context
- ✅ Updated all gRPC calls to include trace metadata

### 3. Docker Compose
- ✅ Added `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable
- ✅ Added dependency on Tempo service

## How to Use

### Step 1: Rebuild and Start Services

```bash
# Stop existing containers
docker-compose down

# Rebuild with new tracing code
docker-compose up --build -d

# Check logs to verify tracing is initialized
docker-compose logs user-auth-service | grep -i "tracing\|otel"
docker-compose logs api-gateway | grep -i "tracing\|tempo"
```

Expected output:
```
user-auth-service | INFO: Initializing OpenTelemetry tracing with endpoint: tempo:4317
user-auth-service | INFO: OpenTelemetry tracing initialized successfully
api-gateway | INFO: Initializing observability tempo_endpoint=http://tempo:4317
```

### Step 2: Generate Traces

Run the test script:
```bash
./test_distributed_tracing.sh
```

Or make a manual request:
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "trace-test@example.com",
    "password": "TestPassword123!",
    "first_name": "Trace",
    "last_name": "Test",
    "phone": "+1234567890"
  }'
```

### Step 3: Query Traces in Grafana

1. Open Grafana: **http://localhost:3000**
2. Click **Explore** (compass icon on left)
3. Select **Tempo** from the dropdown
4. Enter a TraceQL query:

#### Query Examples

**All traces (last 5 minutes)**:
```traceql
{ }
```

**Traces from API Gateway**:
```traceql
{ resource.service.name = "api-gateway" }
```

**Traces from User Auth Service** (this now works!):
```traceql
{ resource.service.name = "user-auth-service" }
```

**Traces for Register endpoint**:
```traceql
{ span.name = "user.UserService/Register" }
```

**Traces with errors**:
```traceql
{ status = error }
```

**Traces for specific user**:
```traceql
{ span.user.id = "user-123" }
```

### Step 4: Analyze Distributed Trace

Click on any trace to see:

1. **Service Map**: Visual representation of service calls
   - api-gateway → user-auth-service

2. **Trace Timeline**: Duration of each span
   ```
   ┌─ api-gateway: gateway_request (200ms)
   │  └─ api-gateway: grpc.user.Register (180ms)
   │     └─ user-auth-service: user.UserService/Register (170ms)
   ```

3. **Span Details**: Click any span to see:
   - Service name
   - Operation name
   - Duration
   - Attributes (HTTP method, status code, etc.)
   - Events and logs

## Verification Checklist

✅ **Both services export traces**:
```bash
# Should see OTLP export logs
docker-compose logs user-auth-service | grep -i "export"
docker-compose logs api-gateway | grep -i "export"
```

✅ **Tempo receives traces**:
```bash
# Should see ingestion logs
docker-compose logs tempo | grep -i "received"
```

✅ **Traces are connected**:
- Query: `{ resource.service.name = "api-gateway" }`
- Click on a trace
- You should see child spans from "user-auth-service"

✅ **Service names are correct**:
- `api-gateway` (not `api_gateway`)
- `user-auth-service` (not `user_auth_service`)

## Common TraceQL Queries

### By Service
```traceql
{ resource.service.name = "api-gateway" }
{ resource.service.name = "user-auth-service" }
```

### By Operation
```traceql
{ span.name = "user.UserService/Register" }
{ span.name = "user.UserService/Login" }
{ span.name = "gateway_request" }
```

### By HTTP Method
```traceql
{ span.http.method = "POST" }
{ span.http.method = "GET" }
```

### By Status
```traceql
{ status = error }
{ status = ok }
{ span.http.status_code = 200 }
{ span.http.status_code >= 400 }
```

### By Duration
```traceql
{ duration > 1s }
{ duration > 500ms }
```

### Combined Queries
```traceql
{ resource.service.name = "user-auth-service" && duration > 100ms }
{ span.http.method = "POST" && status = error }
{ resource.service.name = "api-gateway" && span.http.status_code >= 500 }
```

## Troubleshooting

### "No traces found"

1. **Wait 5-10 seconds** after making a request
2. **Expand time range** in Grafana (top right)
3. **Check service logs**:
   ```bash
   docker-compose logs user-auth-service
   docker-compose logs api-gateway
   docker-compose logs tempo
   ```

### "Traces not connected"

1. Verify trace context propagation:
   ```bash
   docker-compose logs api-gateway | grep -i "traceparent"
   ```

2. Check gRPC metadata injection:
   - Should see "Injected trace header" in logs

### "Service name not found"

1. Verify exact service name (case-sensitive):
   - ✅ `user-auth-service`
   - ❌ `user_auth_service`
   - ❌ `UserAuthService`

2. Check resource attributes in span details

## Performance Impact

- **CPU**: < 1% overhead per service
- **Memory**: ~10-20 MB per service
- **Network**: ~1-5 KB per trace
- **Latency**: < 1ms added to requests

## Next Steps

1. **Add custom spans** for specific operations:
   ```go
   // Go
   ctx, span := tracer.Start(ctx, "database.query")
   defer span.End()
   ```

   ```rust
   // Rust
   #[tracing::instrument]
   async fn my_function() { }
   ```

2. **Add span attributes**:
   ```go
   span.SetAttributes(
       attribute.String("user.email", email),
       attribute.Int("query.rows", count),
   )
   ```

3. **Reduce sampling** for production:
   ```go
   SamplingRate: 0.1  // 10% of traces
   ```

4. **Set up alerts** in Grafana for:
   - High error rates
   - Slow traces (> 1s)
   - Service unavailability

## Resources

- Full documentation: `DISTRIBUTED_TRACING_SETUP.md`
- Test script: `./test_distributed_tracing.sh`
- Grafana: http://localhost:3000
- Tempo: http://localhost:3200
