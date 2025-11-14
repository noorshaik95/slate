# How to Verify Trace Propagation

This guide shows you how to verify that trace context is properly propagating from the API Gateway to the User Auth Service.

## Quick Verification

Run the automated check:
```bash
./check_trace_propagation.sh
```

## Method 1: Visual Inspection in Grafana (RECOMMENDED)

This is the most reliable way to verify trace propagation.

### Steps:

1. **Generate a trace**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"trace-test@example.com","password":"TestPassword123!"}'
   ```

2. **Open Grafana**: http://localhost:3000

3. **Navigate to Explore**:
   - Click the compass icon (🧭) on the left sidebar
   - Select "Tempo" from the data source dropdown

4. **Query for traces**:
   ```traceql
   { resource.service.name = "api-gateway" }
   ```

5. **Click on any trace** in the results

### What to Look For:

#### ✅ SUCCESSFUL Propagation:

```
Trace Timeline View:
┌─────────────────────────────────────────────────────────┐
│ api-gateway                                             │
│ ├─ gateway_request                    [████████████]   │ 100ms
│ │  └─ grpc.user.Login                 [██████████]     │ 90ms
│ │                                                       │
│ user-auth-service                                       │
│ └─ user.UserService/Login             [████████]       │ 80ms
└─────────────────────────────────────────────────────────┘

Trace ID: 0af7651916cd43dd8448eb211c80319c  ← Same for all spans!
```

**Key Indicators**:
- ✓ Both services appear in the SAME trace
- ✓ Spans are nested (parent-child relationship)
- ✓ Same trace ID across all spans
- ✓ Continuous timeline showing request flow
- ✓ User Auth Service span is a child of API Gateway span

#### ❌ FAILED Propagation:

```
Trace 1:
┌─────────────────────────────────────────────────────────┐
│ api-gateway                                             │
│ ├─ gateway_request                    [████████████]   │
│ │  └─ grpc.user.Login                 [██████████]     │
└─────────────────────────────────────────────────────────┘
Trace ID: 0af7651916cd43dd8448eb211c80319c

Trace 2 (SEPARATE!):
┌─────────────────────────────────────────────────────────┐
│ user-auth-service                                       │
│ └─ user.UserService/Login             [████████]       │
└─────────────────────────────────────────────────────────┘
Trace ID: 9bc8762825de54ee9559fc322d91430d  ← Different!
```

**Problems**:
- ✗ Services appear in SEPARATE traces
- ✗ Different trace IDs
- ✗ No parent-child relationship
- ✗ Disconnected timelines

## Method 2: Inspect Span Details

### Steps:

1. Open a trace in Grafana (as above)
2. Click on the **API Gateway span**
3. Look at the "Tags" section

### API Gateway Span Should Have:

```yaml
service.name: "api-gateway"
http.method: "POST"
http.target: "/api/auth/login"
trace_id: "0af7651916cd43dd8448eb211c80319c"
span_id: "b7ad6b7169203331"
```

4. Click on the **User Auth Service span**
5. Look at the "Tags" section

### User Auth Service Span Should Have:

```yaml
service.name: "user-auth-service"
rpc.service: "user.UserService"
rpc.method: "Login"
trace_id: "0af7651916cd43dd8448eb211c80319c"  ← SAME as API Gateway!
span_id: "c8be7c8270304442"
parent_span_id: "b7ad6b7169203331"  ← Points to API Gateway span!
```

**Key Check**: The `trace_id` must be identical, and `parent_span_id` should match the API Gateway's `span_id`.

## Method 3: Check Service Logs

### API Gateway Logs:

```bash
docker-compose logs api-gateway | grep -i "login" | tail -10
```

Look for:
```json
{
  "message": "Calling user service with typed client",
  "method": "Login",
  "span": {
    "trace_id": "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### User Auth Service Logs:

```bash
docker-compose logs user-auth-service | grep -i "login" | tail -10
```

Look for logs with gRPC method calls. The trace context is automatically extracted by the `otelgrpc` interceptor.

## Method 4: Query Tempo API

### Search for Traces:

```bash
curl -s "http://localhost:3200/api/search?limit=10" | jq
```

### Get Specific Trace:

```bash
# Replace TRACE_ID with actual trace ID from search
curl -s "http://localhost:3200/api/traces/TRACE_ID" | jq
```

Look for multiple spans with the same `traceId` but different `spanId` values.

## Method 5: TraceQL Queries

### Query 1: Find traces spanning both services

```traceql
{ } | select(resource.service.name = "api-gateway") 
    | select(resource.service.name = "user-auth-service")
```

**Expected**: Returns traces that contain spans from both services

### Query 2: Find specific operations

```traceql
{ span.name = "user.UserService/Login" }
```

**Expected**: Shows traces with the Login operation, should include parent spans from API Gateway

### Query 3: Check span relationships

```traceql
{ resource.service.name = "user-auth-service" && span.parent_span_id != "" }
```

**Expected**: User Auth Service spans should have parent span IDs (from API Gateway)

## Method 6: Check gRPC Metadata

To verify that trace context is being injected into gRPC metadata, you can add debug logging:

### Temporary Debug Code (Optional):

Add to `services/api-gateway/src/handlers/user_service.rs`:

```rust
fn inject_trace_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    // ... existing code ...
    
    // Debug: Print metadata
    debug!("gRPC metadata: {:?}", request.metadata());
    
    request
}
```

Then check logs:
```bash
docker-compose logs api-gateway | grep "gRPC metadata"
```

You should see `traceparent` in the metadata.

## Common Issues and Solutions

### Issue 1: No User Auth Service spans in traces

**Symptoms**:
- Only API Gateway spans appear
- User Auth Service has separate traces

**Check**:
```bash
docker-compose logs user-auth-service | grep -i "tracing\|otel"
```

**Expected**:
```
INFO: OpenTelemetry tracing initialized successfully
```

**Solution**: Ensure `otelgrpc.NewServerHandler()` is configured in the gRPC server.

### Issue 2: Different trace IDs

**Symptoms**:
- Both services have traces but with different IDs
- No parent-child relationship

**Check**:
```bash
docker-compose logs api-gateway | grep -i "inject\|propagat"
```

**Solution**: Verify `inject_trace_context()` is called before gRPC requests.

### Issue 3: Traces not appearing in Tempo

**Symptoms**:
- Services log activity but no traces in Grafana

**Check**:
```bash
# Check Tempo connectivity
docker-compose exec api-gateway ping -c 3 tempo
docker-compose exec user-auth-service ping -c 3 tempo

# Check Tempo logs
docker-compose logs tempo | tail -20
```

**Solution**: Verify OTLP endpoint configuration in docker-compose.yml.

## Verification Checklist

Use this checklist to confirm trace propagation:

- [ ] Both services export traces to Tempo
- [ ] Traces appear in Grafana Tempo UI
- [ ] Single trace contains spans from both services
- [ ] Spans have parent-child relationship
- [ ] Same trace ID across all spans
- [ ] User Auth Service span has `parent_span_id`
- [ ] Timeline shows continuous request flow
- [ ] Span attributes are correct for both services

## Example: Complete Verification Flow

```bash
# 1. Generate trace
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "verify@example.com",
    "password": "Test123!",
    "first_name": "Verify",
    "last_name": "User",
    "phone": "+1234567890"
  }'

# 2. Wait for export
sleep 5

# 3. Check logs
echo "=== API Gateway ==="
docker-compose logs api-gateway | grep -i "register" | tail -3

echo "=== User Auth Service ==="
docker-compose logs user-auth-service | grep -i "register" | tail -3

# 4. Query Tempo
echo "=== Tempo Traces ==="
curl -s "http://localhost:3200/api/search?limit=1" | jq '.traces[0]'

# 5. Open Grafana
echo "Open: http://localhost:3000"
echo "Query: { span.name = \"user.UserService/Register\" }"
```

## Success Criteria

✅ **Trace propagation is working correctly when**:

1. A single trace contains spans from both services
2. The trace timeline shows a connected flow
3. User Auth Service spans are children of API Gateway spans
4. All spans share the same trace ID
5. Parent-child relationships are visible in the UI
6. Span attributes correctly identify each service

## Additional Resources

- **Automated Check**: `./check_trace_propagation.sh`
- **Test Script**: `./test_distributed_tracing.sh`
- **Full Documentation**: `DISTRIBUTED_TRACING_SETUP.md`
- **Quick Start**: `TRACING_QUICK_START.md`

## Need Help?

If trace propagation isn't working:

1. Run: `./check_trace_propagation.sh`
2. Check: `TRACING_DEPLOYMENT_CHECKLIST.md`
3. Review: Service logs for errors
4. Verify: OpenTelemetry initialization in both services
