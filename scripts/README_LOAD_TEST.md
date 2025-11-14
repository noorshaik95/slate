# Load Testing Scripts

This directory contains scripts to create 10,000 users and test rate limiting functionality.

## Available Scripts

### 1. Bash Script (Recommended for Quick Testing)

**File:** `load_test_users.sh`

**Requirements:**
- `grpcurl` - Install with: `brew install grpcurl` (macOS)

**Usage:**
```bash
./scripts/load_test_users.sh
```

**Features:**
- Tests registration rate limiting (3 per hour)
- Tests login rate limiting (5 per 15 minutes)
- Creates 10,000 users with progress tracking
- Color-coded output for easy reading
- Interactive confirmation before bulk creation

### 2. Python Script (Advanced Testing)

**File:** `load_test_users.py`

**Requirements:**
- Python 3.7+
- gRPC Python libraries
- Generated protobuf files

**Setup:**
```bash
# Install dependencies
pip install grpcio grpcio-tools

# Generate protobuf files
cd proto
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. user.proto
cd ..
```

**Usage:**
```bash
python3 scripts/load_test_users.py
```

**Features:**
- Concurrent user creation with thread pool
- Detailed statistics and performance metrics
- Rate limiting verification
- Configurable batch size and worker count
- Progress tracking with throughput metrics

## Rate Limiting Configuration

Current rate limits (configured in `docker-compose.yml`):

- **Registration:** 3 attempts per hour (3600 seconds)
- **Login:** 5 attempts per 15 minutes (900 seconds)

## Test Phases

Both scripts execute in three phases:

### Phase 1: Registration Rate Limiting Test
- Attempts 10 rapid registrations
- Verifies that only first 3 succeed
- Remaining 7 should be rate limited

### Phase 2: Login Rate Limiting Test
- Creates a test user
- Attempts 10 rapid logins
- Verifies that only first 5 succeed
- Remaining 5 should be rate limited

### Phase 3: Bulk User Creation
- Creates 10,000 users
- Tracks success/failure/rate-limited counts
- Reports performance metrics

## Expected Output

### Successful Rate Limiting
```
Rate Limiting Test Results:
  Successful: 3
  Rate Limited: 7
  Expected: First 3 succeed, rest rate limited
  ✓ Rate limiting is WORKING
```

### Performance Metrics
```
User Creation Complete!
Total time: 120s
Average rate: 83.3 req/s
Successful: 9997
Failed: 0
Rate Limited: 3
```

## Monitoring During Tests

### Check Prometheus Metrics
```bash
# View rate limit metrics
curl -s 'http://localhost:9090/api/v1/query?query=rate_limit_exceeded_total' | jq

# View registration metrics
curl -s 'http://localhost:9090/api/v1/query?query=user_registrations_total' | jq

# View login metrics
curl -s 'http://localhost:9090/api/v1/query?query=user_logins_total' | jq
```

### Check Redis Rate Limit Keys
```bash
# Connect to Redis
docker-compose exec redis redis-cli

# List rate limit keys
KEYS ratelimit:*

# Check specific key TTL
TTL ratelimit:register:127.0.0.1
```

### View Service Logs
```bash
# User auth service logs
docker-compose logs -f user-auth-service

# API gateway logs
docker-compose logs -f api-gateway
```

## Troubleshooting

### grpcurl not found
```bash
# macOS
brew install grpcurl

# Linux
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest

# Or download binary from: https://github.com/fullstorydev/grpcurl/releases
```

### Connection refused
```bash
# Check if services are running
docker-compose ps

# Check if port 50051 is accessible
nc -zv localhost 50051

# Restart services
docker-compose restart user-auth-service
```

### Rate limiting not working
```bash
# Check Redis is running
docker-compose ps redis

# Check rate limit configuration
docker-compose exec user-auth-service env | grep RATE_LIMIT

# Verify Redis connection
docker-compose exec redis redis-cli ping
```

### Python script import errors
```bash
# Ensure protobuf files are generated
cd proto
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. user.proto

# Install required packages
pip install grpcio grpcio-tools
```

## Performance Tuning

### Increase Concurrency (Python Script)
Edit `load_test_users.py`:
```python
MAX_WORKERS = 50  # Increase from 20
BATCH_SIZE = 200  # Increase from 100
```

### Adjust Rate Limits
Edit `docker-compose.yml`:
```yaml
RATE_LIMIT_REGISTER_MAX: "10"      # Increase from 3
RATE_LIMIT_REGISTER_WINDOW: "60"   # Decrease from 3600
```

Then restart:
```bash
docker-compose restart user-auth-service
```

## Database Verification

After running the load test, verify users in the database:

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

# Check users by email pattern
SELECT COUNT(*) FROM users WHERE email LIKE '%loadtest.com';
```

## Cleanup

To remove test users from the database:

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U postgres -d userauth

# Delete load test users
DELETE FROM users WHERE email LIKE '%loadtest.com';

# Verify deletion
SELECT COUNT(*) FROM users;
```

To clear Redis rate limit data:

```bash
# Connect to Redis
docker-compose exec redis redis-cli

# Clear all rate limit keys
KEYS ratelimit:* | xargs redis-cli DEL

# Or flush all Redis data (use with caution)
FLUSHALL
```
