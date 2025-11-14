# How to Trace Requests

## Overview

The system has distributed tracing configured with:
- **Tempo**: Trace storage backend (port 4317 for OTLP)
- **Grafana**: Visualization dashboard (port 3000)
- **OpenTelemetry**: Instrumentation in API Gateway

## Accessing Traces

### 1. Open Grafana
```
URL: http://localhost:3000
Username: admin
Password: admin
```

### 2. Navigate to Explore
1. Click on the **Explore** icon (compass) in the left sidebar
2. Select **Tempo** as the data source from the dropdown at the top

### 3. Search for Traces

**Option A: Search by Service**
- In the "Query type" dropdown, select **Search**
- Set filters:
  - Service Name: `api-gateway`
  - Span Name: `request` (for HTTP requests)
  - Status: `ok` or `error`

**Option B: Search by Trace ID**
- If you have a trace ID from logs, select **TraceQL** query type
- Enter the trace ID directly

**Option C: Use TraceQL Query**
```traceql
# Find all requests to auth endpoints
{ span.http.route = "/api/auth/register" }

# Find all requests with errors
{ status = error }

# Find slow requests (>1s)
{ duration > 1s }

# Find requests to user-auth-service
{ resource.service.name = "user-auth-service" }
```

## Understanding Trace Data

### Trace Structure
Each trace shows:
- **Spans**: Individual operations (HTTP request, gRPC call, database query)
- **Duration**: How long each operation took
- **Attributes**: Metadata (HTTP method, status code, error messages)
- **Events**: Log events within spans

### Example Trace Flow
```
HTTP POST /api/auth/register
├─ Auth Middleware (check if public route)
├─ Route Matching (find gRPC method)
├─ gRPC Call to user-auth-service
│  ├─ user.UserService/Register
│  ├─ Database: INSERT user
│  └─ Database: INSERT user_role
└─ Response Serialization
```

## Viewing Traces in Logs

### API Gateway Logs
```bash
# View recent gateway logs with trace info
docker-compose logs --tail=50 api-gateway | grep -E "trace|span"

# Follow logs in real-time
docker-compose logs -f api-gateway
```

### Extract Trace ID from Logs
The gateway logs include trace context. Look for fields like:
```json
{
  "timestamp": "...",
  "level": "INFO",
  "span": {
    "method": "POST",
    "uri": "/api/auth/register",
    "name": "request"
  },
  "trace_id": "abc123...",
  "span_id": "def456..."
}
```

## Making Traced Requests

### Add Trace Headers (Optional)
You can propagate trace context by adding headers:

```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -H "traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01" \
  -d '{
    "email": "test@example.com",
    "password": "Pass123!",
    "first_name": "Test",
    "last_name": "User",
    "phone": "+1234567890"
  }'
```

### Automatic Tracing
All requests are automatically traced. The gateway:
1. Creates a root span for each HTTP request
2. Propagates trace context to gRPC calls
3. Exports traces to Tempo every 5 seconds

## Useful Grafana Queries

### 1. Find All Auth Requests
```traceql
{ span.http.route =~ "/api/auth/.*" }
```

### 2. Find Failed Requests
```traceql
{ status = error && resource.service.name = "api-gateway" }
```

### 3. Find Slow Requests
```traceql
{ duration > 500ms && span.http.method = "POST" }
```

### 4. Find Requests by User Email
```traceql
{ span.http.route = "/api/auth/login" && span.email =~ ".*@example.com" }
```

## Trace Metrics

### View in Prometheus
```
URL: http://localhost:9090
```

Available metrics:
- `gateway_requests_total` - Total requests
- `gateway_request_duration_seconds` - Request latency
- `gateway_auth_failures_total` - Auth failures
- `gateway_rate_limit_exceeded_total` - Rate limit hits

### Example Prometheus Queries
```promql
# Request rate
rate(gateway_requests_total[5m])

# 95th percentile latency
histogram_quantile(0.95, rate(gateway_request_duration_seconds_bucket[5m]))

# Error rate
rate(gateway_requests_total{status=~"5.."}[5m])
```

## Debugging with Traces

### 1. Identify Slow Operations
- Look at span durations in the waterfall view
- Identify which service/operation is taking the most time
- Check for sequential operations that could be parallelized

### 2. Find Error Sources
- Filter traces by `status = error`
- Look at span events for error messages
- Check span attributes for error details

### 3. Verify Request Flow
- Ensure all expected spans are present
- Check that trace context is propagated correctly
- Verify gRPC calls reach the backend service

## Example: Tracing a Register Request

### 1. Make a Request
```bash
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "trace-test@example.com",
    "password": "Pass123!",
    "first_name": "Trace",
    "last_name": "Test",
    "phone": "+1234567890"
  }'
```

### 2. Find in Grafana
1. Go to Grafana Explore → Tempo
2. Search for traces in the last 5 minutes
3. Filter by: `span.http.route = "/api/auth/register"`
4. Click on a trace to see the full waterfall

### 3. Analyze the Trace
You should see:
- **HTTP Request Span** (~100-500ms total)
  - Auth middleware check (~1-5ms)
  - Route matching (~1-2ms)
  - **gRPC Call Span** (~50-200ms)
    - Register method execution
    - Database operations
  - Response serialization (~1-5ms)

## Trace Retention

- **Tempo**: Traces are stored in memory (ephemeral)
- **Retention**: Traces are kept for the lifetime of the container
- **For Production**: Configure persistent storage in `config/tempo.yaml`

## Troubleshooting

### No Traces Appearing
1. Check Tempo is running: `docker-compose ps tempo`
2. Verify gateway can reach Tempo: `docker-compose logs api-gateway | grep tempo`
3. Check Tempo logs: `docker-compose logs tempo`

### Traces Not Linked
- Ensure trace context propagation is working
- Check that gRPC metadata includes trace headers
- Verify OpenTelemetry configuration

### Grafana Can't Connect to Tempo
1. Check datasource configuration: `config/grafana-datasources.yaml`
2. Verify Tempo endpoint: `http://tempo:3200`
3. Restart Grafana: `docker-compose restart grafana`

## Advanced: Custom Spans

To add custom spans in the gateway code:

```rust
use tracing::{info_span, instrument};

#[instrument(skip(channel))]
async fn my_function(channel: Channel) {
    let span = info_span!("custom_operation", operation = "my_op");
    let _enter = span.enter();
    
    // Your code here
    
    // Span automatically ends when _enter is dropped
}
```

## Quick Reference

| Component | URL | Purpose |
|-----------|-----|---------|
| Grafana | http://localhost:3000 | View traces and metrics |
| Tempo | http://localhost:3200 | Trace storage (internal) |
| Prometheus | http://localhost:9090 | Metrics storage |
| API Gateway | http://localhost:8080 | Application entry point |

## Example Workflow

1. **Make a request** to the API gateway
2. **Check logs** for trace ID: `docker-compose logs api-gateway | tail -20`
3. **Open Grafana** at http://localhost:3000
4. **Go to Explore** → Select Tempo
5. **Search** for recent traces or use the trace ID
6. **Analyze** the waterfall view to see timing and errors
7. **Drill down** into specific spans for details

---

**Pro Tip**: Keep Grafana open in a browser tab while testing. Refresh the trace search after each request to see the distributed trace in real-time!
