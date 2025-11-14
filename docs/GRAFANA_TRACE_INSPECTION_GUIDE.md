# Grafana Trace Inspection Guide

## Step-by-Step Visual Guide

### Step 1: Open Grafana and Navigate to Tempo

1. Open your browser: **http://localhost:3000**
2. Click the **Explore** icon (🧭 compass) in the left sidebar
3. Select **"Tempo"** from the data source dropdown at the top

### Step 2: Query for Traces

In the query builder, enter:
```traceql
{ resource.service.name = "api-gateway" }
```

Click **"Run query"** button (or press Shift+Enter)

### Step 3: Identify a Trace

You'll see a list of traces. Each trace shows:
- **Trace ID**: Unique identifier (e.g., `8d1caca48426bea339f474660942760`)
- **Duration**: How long the request took (e.g., `91ms`)
- **Spans**: Number of operations (e.g., `3 spans`)
- **Services**: Which services were involved (e.g., `api-gateway, user-auth-service`)

**✅ GOOD SIGN**: If you see **2 services** listed, propagation is working!

### Step 4: Open the Trace Details

Click on any trace row to open the detailed view.

## What You Should See (Successful Propagation)

### Trace Timeline View

```
┌─────────────────────────────────────────────────────────────────┐
│ Service: api-gateway                                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ gateway_request                    [████████████████████]   │ │ 91ms
│ │   Span ID: b7ad6b7169203331                                 │ │
│ │   ┌───────────────────────────────────────────────────────┐ │ │
│ │   │ grpc.user.Login                [████████████████]     │ │ │ 87ms
│ │   │   Span ID: c8be7c8270304442                           │ │ │
│ │   └───────────────────────────────────────────────────────┘ │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                                 │
│ Service: user-auth-service                                      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │   user.UserService/Login           [██████████████]        │ │ 80ms
│ │     Span ID: d9cf8d9381405553                               │ │
│ │     Parent: c8be7c8270304442  ← Points to API Gateway!     │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘

Trace ID: 8d1caca48426bea339f474660942760  ← Same for ALL spans!
```

### Key Indicators of Successful Propagation:

1. **✅ Multiple Services**: Both `api-gateway` and `user-auth-service` appear
2. **✅ Nested Structure**: User Auth Service span is indented (child of API Gateway)
3. **✅ Same Trace ID**: All spans share the same trace ID
4. **✅ Parent-Child Links**: User Auth span has a parent span ID pointing to API Gateway
5. **✅ Continuous Timeline**: No gaps, shows complete request flow

## Inspecting Individual Spans

### Click on the API Gateway Span

You'll see a panel with tabs. Click on **"Tags"** or **"Attributes"**:

```yaml
# Resource Attributes
resource.service.name: "api-gateway"

# Span Attributes
http.method: "POST"
http.target: "/api/auth/login"
http.status_code: 200
rpc.service: "user.UserService"
rpc.method: "Login"

# Trace Context
trace_id: "8d1caca48426bea339f474660942760"
span_id: "c8be7c8270304442"
parent_span_id: "b7ad6b7169203331"
```

### Click on the User Auth Service Span

```yaml
# Resource Attributes
resource.service.name: "user-auth-service"
resource.service.version: "1.0.0"

# Span Attributes
rpc.system: "grpc"
rpc.service: "user.UserService"
rpc.method: "Login"
rpc.grpc.status_code: 0

# Trace Context (IMPORTANT!)
trace_id: "8d1caca48426bea339f474660942760"  ← SAME as API Gateway!
span_id: "d9cf8d9381405553"
parent_span_id: "c8be7c8270304442"  ← Points to API Gateway span!
```

**🔍 Critical Check**: 
- `trace_id` must be **identical** in both spans
- `parent_span_id` in User Auth span must match `span_id` from API Gateway

## What Failed Propagation Looks Like

### Symptom 1: Separate Traces

If you query `{ resource.service.name = "api-gateway" }` and see:

```
Trace 1: 8d1caca48426bea339f474660942760
  Services: api-gateway only
  Spans: 2
  Duration: 91ms
```

Then query `{ resource.service.name = "user-auth-service" }` and see:

```
Trace 2: a808d713008ae1176e82d2deccd243f7  ← Different trace ID!
  Services: user-auth-service only
  Spans: 1
  Duration: 80ms
```

**❌ Problem**: Different trace IDs = No propagation

### Symptom 2: Missing Parent-Child Relationship

If both services appear in one trace but:

```
┌─ api-gateway: gateway_request
│  └─ api-gateway: grpc.user.Login
│
└─ user-auth-service: user.UserService/Login  ← Not nested!
```

**❌ Problem**: Flat structure = Context not propagated correctly

## Advanced Queries

### Find Traces Spanning Both Services

```traceql
{ } | select(resource.service.name = "api-gateway") 
    | select(resource.service.name = "user-auth-service")
```

**Expected**: Only returns traces that contain spans from BOTH services

### Find Slow Traces

```traceql
{ duration > 100ms }
```

**Use**: Identify performance bottlenecks across services

### Find Errors

```traceql
{ status = error }
```

**Use**: Debug failed requests with full context

### Find Specific Operations

```traceql
{ span.name = "user.UserService/Register" }
```

**Use**: Track specific endpoints

## Service Graph View

Some Grafana versions show a **Service Graph**:

1. In the trace details, look for a **"Service Graph"** or **"Node Graph"** tab
2. You should see:

```
┌─────────────┐         ┌──────────────────┐
│ api-gateway │ ──────> │ user-auth-service│
└─────────────┘         └──────────────────┘
     91ms                      80ms
```

This visualizes the service dependency and latency.

## Troubleshooting in Grafana

### No Traces Appear

**Check**:
1. Time range (top right) - expand to "Last 15 minutes"
2. Query syntax - try `{ }` to see all traces
3. Tempo data source - ensure it's selected

**Fix**:
```bash
# Verify Tempo is receiving data
docker-compose logs tempo | grep -i "received\|ingested"
```

### Only API Gateway Traces

**Check**:
```bash
# Verify User Auth Service is exporting
docker-compose logs user-auth-service | grep -i "tracing\|otel"
```

**Expected**:
```
INFO: OpenTelemetry tracing initialized successfully
```

### Traces Not Connected

**Check**:
```bash
# Verify trace injection
docker-compose logs api-gateway | grep -i "inject"
```

**Fix**: Ensure `inject_trace_context()` is called in `user_service.rs`

## Quick Verification Checklist

Open a trace in Grafana and verify:

- [ ] Trace contains spans from both services
- [ ] User Auth Service span is nested under API Gateway span
- [ ] All spans have the same `trace_id`
- [ ] User Auth span has `parent_span_id` pointing to API Gateway
- [ ] Timeline shows continuous flow (no gaps)
- [ ] Span attributes correctly identify each service
- [ ] Total duration makes sense (child ≤ parent)

## Example: Complete Inspection

1. **Generate trace**:
   ```bash
   curl -X POST http://localhost:8080/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"trace-test@example.com","password":"TestPassword123!"}'
   ```

2. **Wait 5 seconds** for export

3. **Open Grafana**: http://localhost:3000

4. **Query**: `{ span.name = "user.UserService/Login" }`

5. **Click on trace**

6. **Verify**:
   - ✅ See both `api-gateway` and `user-auth-service`
   - ✅ Nested structure
   - ✅ Same trace ID
   - ✅ Parent-child relationship

7. **Click on User Auth span**

8. **Check Tags**:
   - ✅ `trace_id` matches API Gateway
   - ✅ `parent_span_id` points to API Gateway span

## Success!

If you see all the indicators above, **trace propagation is working correctly**! 🎉

You now have full distributed tracing from the API Gateway through to the User Auth Service.

## Next Steps

- Add custom spans for specific operations
- Set up alerts for slow traces
- Create dashboards for service dependencies
- Reduce sampling rate for production (10-20%)

## Resources

- **Automated Check**: `./check_trace_propagation.sh`
- **Full Guide**: `VERIFY_TRACE_PROPAGATION.md`
- **Documentation**: `DISTRIBUTED_TRACING_SETUP.md`
