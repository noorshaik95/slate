# Load Testing & Rate Limit Verification Guide

This guide explains how to test the user authentication service with 10,000 users and verify rate limiting functionality.

## Quick Start

### Option 1: Quick Rate Limit Test (Recommended First)

Test rate limiting without creating thousands of users:

```bash
./scripts/test_rate_limit_quick.sh
```

This will:
- Test registration rate limiting (3 per hour)
- Test login rate limiting (5 per 15 minutes)
- Complete in under 1 minute

### Option 2: Full Load Test with 10k Users

```bash
./scripts/load_test_users.sh
```

This will:
- Run rate limit tests first
- Ask for confirmation before creating 10k users
- Track performance metrics
- Take 2-5 minutes depending on system

### Option 3: Python Script (Advanced)

For concurrent testing with detailed metrics:

```bash
# First time setup
pip install grpcio grpcio-tools
cd proto && python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. user.proto && cd ..

# Run test
python3 scripts/load_test_users.py
```

## Prerequisites

### For Bash Scripts
- `grpcurl` - Install with: `brew install grpcurl`
- Services running: `docker-compose up -d`

### For Python Script
- Python 3.7+
- `pip install grpcio grpcio-tools`
- Generated protobuf files (see setup above)

## What Gets Tested

### 1. Registration Rate Limiting
- **Limit:** 3 registrations per hour per IP
- **Test:** Attempts 10 rapid registrations
- **Expected:** First 3 succeed, remaining 7 are rate limited

### 2. Login Rate Limiting
- **Limit:** 5 login attempts per 15 minutes per IP
- **Test:** Attempts 10 rapid logins with same credentials
- **Expected:** First 5 succeed, remaining 5 are rate limited

### 3. Bulk User Creation (Optional)
- **Target:** 10,000 users
- **Metrics:** Success rate, throughput, failures
- **Duration:** 2-5 minutes

## Expected Output

### Successful Rate Limiting

```
============================================================
TEST 1: Registration Rate Limiting
============================================================
Limit: 3 per hour
Attempting 10 registrations...

  [1] ✓ Success
  [2] ✓ Success
  [3] ✓ Success
  [4] ❌ Rate limited
  [5] ❌ Rate limited
  [6] ❌ Rate limited
  [7] ❌ Rate limited
  [8] ❌ Rate limited
  [9] ❌ Rate limited
  [10] ❌ Rate limited

Results:
  Success: 3
  Rate Limited: 7
  Failed: 0
  ✓ Registration rate limiting WORKS
```

### Performance Metrics (10k Users)

```
User Creation Complete!
Total time: 120s
Average rate: 83.3 req/s
Successful: 9997
Failed: 0
Rate Limited: 3
```

## Monitoring During Tests

### Real-time Metrics (Prometheus)

```bash
# Rate limit violations
curl -s 'http://localhost:9090/api/v1/query?query=rate_limit_exceeded_total' | jq

# User registrations
curl -s 'http://localhost:9090/api/v1/query?query=user_registrations_total' | jq

# User logins
curl -s 'http://localhost:9090/api/v1/query?query=user_logins_total' | jq

# Database connections
curl -s 'http://localhost:9090/api/v1/query?query=db_connections' | jq
```

### Service Logs

```bash
# Watch user-auth-service logs
docker-compose logs -f user-auth-service

# Watch API gateway logs
docker-compose logs -f api-gateway

# Check for rate limit messages
docker-compose logs user-auth-service | grep -i "rate limit"
```

### Redis Rate Limit Keys

```bash
# Connect to Redis
docker-compose exec redis redis-cli

# List all rate limit keys
KEYS ratelimit:*

# Check specific key
GET ratelimit:register:127.0.0.1

# Check TTL (time to live)
TTL ratelimit:register:127.0.0.1
```

### Database Verification

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U postgres -d userauth

# Count total users
SELECT COUNT(*) FROM users;

# View recent users
SELECT id, email, username, created_at 
FROM users 
ORDER BY created_at DESC 
LIMIT 10;

# Count load test users
SELECT COUNT(*) FROM users WHERE email LIKE '%loadtest.com%';
```

## Rate Limit Configuration

Current settings in `docker-compose.yml`:

```yaml
RATE_LIMIT_ENABLED: "true"
RATE_LIMIT_LOGIN_MAX: "5"           # 5 attempts
RATE_LIMIT_LOGIN_WINDOW: "900"      # per 15 minutes
RATE_LIMIT_REGISTER_MAX: "3"        # 3 attempts
RATE_LIMIT_REGISTER_WINDOW: "3600"  # per hour
```

### Adjusting Rate Limits

Edit `docker-compose.yml` and restart:

```bash
# Edit configuration
vim docker-compose.yml

# Restart service
docker-compose restart user-auth-service
```

## Troubleshooting

### grpcurl: command not found

```bash
# macOS
brew install grpcurl

# Linux
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

### Connection refused

```bash
# Check services
docker-compose ps

# Check port
nc -zv localhost 50051

# Restart
docker-compose restart user-auth-service
```

### Rate limiting not working

```bash
# Check Redis
docker-compose ps redis
docker-compose exec redis redis-cli ping

# Check configuration
docker-compose exec user-auth-service env | grep RATE_LIMIT

# Check logs
docker-compose logs user-auth-service | grep -i redis
```

### Database connection errors

```bash
# Check PostgreSQL
docker-compose ps postgres

# Check connection
docker-compose exec postgres pg_isready -U postgres

# View logs
docker-compose logs postgres
```

## Cleanup

### Remove Test Users

```bash
# Connect to database
docker-compose exec postgres psql -U postgres -d userauth

# Delete load test users
DELETE FROM users WHERE email LIKE '%loadtest.com%' OR email LIKE '%test.com%';

# Verify
SELECT COUNT(*) FROM users;
```

### Clear Redis Rate Limits

```bash
# Connect to Redis
docker-compose exec redis redis-cli

# Clear rate limit keys
EVAL "return redis.call('del', unpack(redis.call('keys', 'ratelimit:*')))" 0

# Or flush all (use with caution)
FLUSHALL
```

### Reset Everything

```bash
# Stop all services
docker-compose down

# Remove volumes (deletes all data)
docker-compose down -v

# Restart fresh
docker-compose up -d
```

## Performance Tips

### Increase Throughput

1. **Increase database connections** (docker-compose.yml):
   ```yaml
   DB_MAX_OPEN_CONNS: "50"  # Increase from 25
   ```

2. **Use Python script with more workers**:
   ```python
   MAX_WORKERS = 50  # Increase from 20
   ```

3. **Reduce delays in bash script**:
   ```bash
   sleep 0.05  # Reduce from 0.1
   ```

### Monitor System Resources

```bash
# Docker stats
docker stats

# System resources
htop

# Network connections
netstat -an | grep 50051 | wc -l
```

## Grafana Dashboards

View metrics in Grafana: http://localhost:3000

1. Navigate to Dashboards
2. Look for:
   - User Auth Service Metrics
   - Rate Limiting Dashboard
   - Database Performance

## Scripts Reference

| Script | Purpose | Duration | Users Created |
|--------|---------|----------|---------------|
| `test_rate_limit_quick.sh` | Quick rate limit test | <1 min | ~20 |
| `load_test_users.sh` | Full load test | 2-5 min | 10,000 |
| `load_test_users.py` | Advanced concurrent test | 1-3 min | 10,000 |

## Support

For issues or questions:
1. Check service logs: `docker-compose logs`
2. Verify configuration: `docker-compose config`
3. Check connectivity: `grpcurl -plaintext localhost:50051 list`
4. Review README: `scripts/README_LOAD_TEST.md`
