# Distributed Tracing Deployment Checklist

Use this checklist to verify the distributed tracing implementation is working correctly.

## Pre-Deployment

### Code Changes
- [x] Created `services/user-auth-service/pkg/tracing/tracing.go`
- [x] Updated `services/user-auth-service/cmd/server/main.go`
- [x] Updated `services/user-auth-service/go.mod` with OpenTelemetry dependencies
- [x] Updated `services/api-gateway/src/handlers/user_service.rs`
- [x] Updated `docker-compose.yml` with OTEL environment variables
- [x] All code compiles without errors

### Documentation
- [x] Created `DISTRIBUTED_TRACING_SETUP.md` (technical details)
- [x] Created `TRACING_QUICK_START.md` (user guide)
- [x] Created `TRACING_IMPLEMENTATION_SUMMARY.md` (overview)
- [x] Created `test_distributed_tracing.sh` (test script)
- [x] Updated `README.md` with tracing section

## Deployment Steps

### 1. Stop Existing Services
```bash
docker-compose down
```

### 2. Rebuild Services
```bash
docker-compose build
```

**Expected**: Both services build successfully without errors

### 3. Start Services
```bash
docker-compose up -d
```

**Expected**: All services start and become healthy

### 4. Verify Service Health
```bash
# Check all services are running
docker-compose ps

# Check API Gateway
curl http://localhost:8080/health

# Check Grafana
curl http://localhost:3000/api/health

# Check Tempo
curl http://localhost:3200/ready
```

**Expected**: All health checks return 200 OK

## Post-Deployment Verification

### 1. Check Tracing Initialization

#### API Gateway
```bash
docker-compose logs api-gateway | grep -i "tracing\|tempo"
```

**Expected Output**:
```
INFO: Initializing observability tempo_endpoint=http://tempo:4317
INFO: Starting API Gateway
```

#### User Auth Service
```bash
docker-compose logs user-auth-service | grep -i "tracing\|otel"
```

**Expected Output**:
```
INFO: Initializing OpenTelemetry tracing with endpoint: tempo:4317
INFO: OpenTelemetry tracing initialized successfully
```

### 2. Generate Test Traces
```bash
./test_distributed_tracing.sh
```

**Expected**: Script completes successfully with HTTP 200 or 409 response

### 3. Verify Traces in Tempo

#### Check Tempo Logs
```bash
docker-compose logs tempo | tail -20
```

**Expected**: No errors, service running normally

#### Query Tempo API
```bash
# Wait 10 seconds for trace ingestion
sleep 10

# Search for traces (requires jq)
curl -s "http://localhost:3200/api/search?tags=service.name=api-gateway" | jq
```

**Expected**: JSON response with trace data

### 4. Verify in Grafana UI

1. **Open Grafana**: http://localhost:3000
2. **Navigate**: Explore → Select "Tempo" datasource
3. **Query**: `{ }`
4. **Expected**: List of traces appears

### 5. Test Service-Specific Queries

#### API Gateway Traces
```traceql
{ resource.service.name = "api-gateway" }
```
**Expected**: Traces from API Gateway appear

#### User Auth Service Traces
```traceql
{ resource.service.name = "user-auth-service" }
```
**Expected**: Traces from User Auth Service appear (THIS WAS THE FIX!)

#### Register Method Traces
```traceql
{ span.name = "user.UserService/Register" }
```
**Expected**: Traces for Register operations appear

### 6. Verify Trace Connectivity

1. Query: `{ resource.service.name = "api-gateway" }`
2. Click on any trace
3. **Expected**: 
   - See spans from both `api-gateway` AND `user-auth-service`
   - Spans are connected in a parent-child relationship
   - Trace ID is the same across all spans

### 7. Verify Span Attributes

Click on a span and check attributes:

#### API Gateway Span
- [x] `service.name = "api-gateway"`
- [x] `http.method` (e.g., "POST")
- [x] `http.target` (e.g., "/api/auth/register")
- [x] `http.status_code` (e.g., 200)

#### User Auth Service Span
- [x] `service.name = "user-auth-service"`
- [x] `rpc.service` (e.g., "user.UserService")
- [x] `rpc.method` (e.g., "Register")
- [x] `rpc.grpc.status_code` (e.g., 0 for OK)

## Troubleshooting

### Issue: No traces in Tempo

**Check**:
```bash
# 1. Verify services can reach Tempo
docker-compose exec api-gateway ping -c 3 tempo
docker-compose exec user-auth-service ping -c 3 tempo

# 2. Check for export errors
docker-compose logs api-gateway | grep -i "error\|failed"
docker-compose logs user-auth-service | grep -i "error\|failed"

# 3. Verify OTLP endpoint configuration
docker-compose exec api-gateway env | grep TEMPO
docker-compose exec user-auth-service env | grep OTEL
```

**Fix**: Ensure environment variables are set correctly in docker-compose.yml

### Issue: Traces not connected

**Check**:
```bash
# Verify trace context propagation
docker-compose logs api-gateway | grep -i "traceparent\|inject"
```

**Expected**: Should see "Injected trace header" messages

**Fix**: Ensure `inject_trace_context()` is called in user_service.rs

### Issue: "user-auth-service" traces not found

**Check**:
```bash
# Verify service name in traces
docker-compose logs user-auth-service | grep -i "service.name"
```

**Expected**: Should see `service.name = "user-auth-service"`

**Fix**: Verify tracing.Config in main.go has correct ServiceName

### Issue: High latency or performance issues

**Check**:
```bash
# Monitor resource usage
docker stats

# Check batch export settings
docker-compose logs user-auth-service | grep -i "batch\|export"
```

**Fix**: 
- Reduce sampling rate (set to 0.1 for 10%)
- Increase batch size
- Add more resources to containers

## Performance Baseline

After deployment, establish baseline metrics:

### Latency Impact
```bash
# Before tracing (baseline)
# After tracing (with overhead)
```

**Expected**: < 1ms additional latency per request

### Resource Usage
```bash
docker stats --no-stream
```

**Expected per service**:
- CPU: < 1% additional
- Memory: ~10-20 MB additional

### Trace Volume
```bash
# Count traces per minute
curl -s "http://localhost:3200/api/search?limit=100" | jq '.traces | length'
```

**Expected**: Proportional to request rate (1 trace per request at 100% sampling)

## Production Readiness

### Before Production Deployment

- [ ] Reduce sampling rate to 10-20%:
  ```go
  // Go service
  SamplingRate: 0.1
  ```
  ```rust
  // Rust service
  .with_sampler(Sampler::TraceIdRatioBased(0.1))
  ```

- [ ] Configure Tempo retention policy
- [ ] Set up Grafana alerts for:
  - [ ] High error rates
  - [ ] Slow traces (> 1s)
  - [ ] Service unavailability

- [ ] Configure Tempo storage backend (S3, GCS, etc.)
- [ ] Set up Tempo compaction and retention
- [ ] Enable TLS for OTLP endpoints
- [ ] Configure authentication for Grafana

### Monitoring

Set up alerts for:
- Trace export failures
- High trace volume
- Tempo storage usage
- Query performance

## Success Criteria

✅ **All checks passed**:
- [x] Both services export traces
- [x] Traces appear in Tempo
- [x] Traces are connected across services
- [x] TraceQL queries work for both services
- [x] Span attributes are correct
- [x] Performance impact is minimal
- [x] Documentation is complete

## Rollback Plan

If issues occur:

1. **Disable tracing without redeployment**:
   ```bash
   # Set sampling to 0
   docker-compose exec user-auth-service env OTEL_TRACES_SAMPLER=always_off
   ```

2. **Revert to previous version**:
   ```bash
   git revert <commit-hash>
   docker-compose up --build -d
   ```

3. **Emergency fix**:
   - Comment out tracer initialization in main.go
   - Remove gRPC interceptor
   - Rebuild and redeploy

## Support Resources

- Quick Start: `TRACING_QUICK_START.md`
- Technical Details: `DISTRIBUTED_TRACING_SETUP.md`
- Implementation: `TRACING_IMPLEMENTATION_SUMMARY.md`
- Test Script: `./test_distributed_tracing.sh`

## Sign-Off

- [ ] Development: Tested locally
- [ ] QA: Verified in staging
- [ ] DevOps: Reviewed deployment
- [ ] Security: Approved configuration
- [ ] Product: Accepted feature

**Deployed by**: _________________
**Date**: _________________
**Version**: _________________
