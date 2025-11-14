# Trace Propagation Issue Analysis

## Problem

Trace ID `18a2005378209ebdba590df2e93014a8` from a login request shows spans ONLY from `api-gateway`, with NO spans from `user-auth-service`.

## Investigation Results

### 1. API Gateway (Rust) ✅
- **Tracing initialized**: ✓
- **Spans created**: ✓  
- **Spans exported to Tempo**: ✓
- **Trace injection code added**: ✓ (`inject_trace_context()` function)
- **Instrumentation added**: ✓ (`#[tracing::instrument]` on `call_user_service`)

### 2. User Auth Service (Go) ⚠️
- **Tracing initialized**: ✓
- **gRPC interceptor configured**: ✓ (`otelgrpc.NewServerHandler()`)
- **Requests being processed**: ✓ (returns valid responses)
- **Spans created**: ❌ **NOT VISIBLE IN TEMPO**
- **Debug logs**: ❌ **NOT APPEARING**

### 3. Tempo Query Results
```json
{
  "batches": [
    {
      "resource": {
        "attributes": [
          {"key": "service.name", "value": {"stringValue": "api-gateway"}}
        ]
      },
      "scopeSpans": [...]
    }
  ]
}
```

**Only `api-gateway` spans present. No `user-auth-service` spans.**

## Root Cause Analysis

The issue is that while both services are configured for tracing:

1. **API Gateway** creates and exports spans successfully
2. **User Auth Service** receives requests but:
   - Either NOT creating spans
   - OR creating spans but NOT exporting them
   - OR creating spans with wrong trace context (disconnected)

### Possible Causes:

#### A. Trace Context Not Being Injected
- The `inject_trace_context()` function might not be working
- Metadata might not be properly set in gRPC requests

#### B. Trace Context Not Being Extracted  
- The `otelgrpc.NewServerHandler()` might not be extracting metadata
- W3C Trace Context propagation might not be configured

#### C. Spans Not Being Created
- The Go service might not be in an active span context
- The `otelgrpc` interceptor might not be creating spans

#### D. Spans Not Being Exported
- The Go service might be creating spans but not exporting them
- OTLP exporter might be failing silently

## Evidence

### From Tempo API:
- Trace contains 2 spans, both from `api-gateway`
- No spans with `service.name = "user-auth-service"`
- Trace ID is valid and consistent

### From Logs:
- API Gateway logs show successful gRPC calls
- User Auth Service logs show service is running
- **NO gRPC request logs from User Auth Service** (suspicious!)
- **NO debug logs from interceptor** (very suspicious!)

## Next Steps to Debug

### Step 1: Verify Trace Injection (API Gateway)

Add explicit logging in `inject_trace_context()`:

```rust
fn inject_trace_context<T>(mut request: tonic::Request<T>) -> tonic::Request<T> {
    // ... existing code ...
    
    // After injection, log the metadata
    debug!("gRPC metadata after injection: {:?}", request.metadata());
    
    request
}
```

**Expected**: Should see `traceparent` in metadata

### Step 2: Verify Trace Extraction (Go Service)

The debug interceptor should log:
- All incoming metadata keys
- Specifically the `traceparent` header

**Current Status**: NO logs appearing (indicates interceptor might not be running)

### Step 3: Check gRPC Interceptor Order

The interceptor chain might be wrong:

```go
grpcServer := grpc.NewServer(
    grpc.ChainUnaryInterceptor(
        tracing.LoggingUnaryInterceptor(), // Debug first
    ),
    grpc.StatsHandler(otelgrpc.NewServerHandler()), // Then OTel
)
```

**Issue**: `StatsHandler` and `UnaryInterceptor` are different mechanisms!

### Step 4: Verify OTel Configuration

Check if `otelgrpc.NewServerHandler()` needs additional configuration:

```go
handler := otelgrpc.NewServerHandler(
    otelgrpc.WithTracerProvider(tp),
    otelgrpc.WithPropagators(propagation.TraceContext{}),
)
```

## Recommended Fix

### Option 1: Use Interceptor Instead of StatsHandler

```go
import (
    "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

grpcServer := grpc.NewServer(
    grpc.UnaryInterceptor(
        otelgrpc.UnaryServerInterceptor(),
    ),
)
```

### Option 2: Configure StatsHandler Properly

```go
handler := otelgrpc.NewServerHandler(
    otelgrpc.WithTracerProvider(tp),
)

grpcServer := grpc.NewServer(
    grpc.StatsHandler(handler),
)
```

### Option 3: Verify Propagator Configuration

Ensure the tracer provider uses W3C Trace Context:

```go
tp := sdktrace.NewTracerProvider(
    sdktrace.WithBatcher(exporter),
    sdktrace.WithResource(res),
)

otel.SetTracerProvider(tp)
otel.SetTextMapPropagator(propagation.TraceContext{}) // ← Important!
```

## Testing Plan

1. **Add debug logging** to both services
2. **Make a test request**
3. **Check logs** for:
   - API Gateway: trace context injection
   - User Auth Service: trace context extraction
4. **Query Tempo** for the trace
5. **Verify** both services appear

## Success Criteria

✅ Trace contains spans from BOTH services  
✅ User Auth Service span has `parent_span_id` pointing to API Gateway  
✅ Same `trace_id` across all spans  
✅ Debug logs show trace context being propagated  

## Current Status

🔴 **FAILING**: Trace propagation not working  
⚠️ **INVESTIGATING**: Debug logs not appearing from Go service  
🔍 **NEXT**: Try Option 1 (use UnaryInterceptor instead of StatsHandler)  

## Files to Modify

1. `services/user-auth-service/cmd/server/main.go` - Change interceptor configuration
2. `services/user-auth-service/pkg/tracing/tracing.go` - Add propagator configuration
3. `services/api-gateway/src/handlers/user_service.rs` - Add debug logging

## References

- [OpenTelemetry Go gRPC](https://pkg.go.dev/go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc)
- [W3C Trace Context](https://www.w3.org/TR/trace-context/)
- [gRPC Metadata](https://grpc.io/docs/guides/metadata/)
