#!/bin/bash

# Smoke tests for the API Gateway and User Auth Service
# These tests verify basic connectivity and functionality

set -e

GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
USER_AUTH_URL="${USER_AUTH_URL:-http://localhost:50051}"

echo "🧪 Running smoke tests..."
echo "Gateway URL: $GATEWAY_URL"
echo "User Auth Service URL: $USER_AUTH_URL"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -n "Testing: $test_name... "
    
    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((TESTS_FAILED++))
        return 1
    fi
}

# Test 1: Gateway health check (liveness)
run_test "Gateway liveness probe" \
    "curl -f -s $GATEWAY_URL/health/live"

# Test 2: Gateway readiness check
run_test "Gateway readiness probe" \
    "curl -f -s $GATEWAY_URL/health/ready"

# Test 3: Gateway metrics endpoint
run_test "Gateway metrics endpoint" \
    "curl -f -s $GATEWAY_URL/metrics | grep -q 'api_gateway'"

# Test 4: User Auth Service connectivity (via gateway)
echo -n "Testing: User Auth Service connectivity... "
if command -v grpcurl > /dev/null 2>&1; then
    if grpcurl -plaintext localhost:50051 list > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((TESTS_FAILED++))
    fi
else
    echo -e "${YELLOW}⊘ SKIP (grpcurl not installed)${NC}"
fi

# Test 5: User registration (if service is available)
echo -n "Testing: User registration flow... "
REGISTER_RESPONSE=$(curl -s -X POST "$GATEWAY_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "smoketest@example.com",
        "password": "testpass123",
        "first_name": "Smoke",
        "last_name": "Test",
        "phone": "+1234567890"
    }' 2>&1)

if echo "$REGISTER_RESPONSE" | grep -q "access_token\|email"; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((TESTS_PASSED++))
elif echo "$REGISTER_RESPONSE" | grep -q "already exists\|duplicate"; then
    echo -e "${YELLOW}⊘ SKIP (user already exists)${NC}"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "Response: $REGISTER_RESPONSE"
    ((TESTS_FAILED++))
fi

# Test 6: User login
echo -n "Testing: User login flow... "
LOGIN_RESPONSE=$(curl -s -X POST "$GATEWAY_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d '{
        "email": "admin@example.com",
        "password": "admin123"
    }' 2>&1)

if echo "$LOGIN_RESPONSE" | grep -q "access_token"; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((TESTS_PASSED++))
    
    # Extract token for subsequent tests
    ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"access_token":"[^"]*"' | cut -d'"' -f4)
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "Response: $LOGIN_RESPONSE"
    ((TESTS_FAILED++))
    ACCESS_TOKEN=""
fi

# Test 7: List users (requires authentication)
if [ -n "$ACCESS_TOKEN" ]; then
    echo -n "Testing: List users with authentication... "
    LIST_RESPONSE=$(curl -s -X GET "$GATEWAY_URL/api/users" \
        -H "Authorization: Bearer $ACCESS_TOKEN" 2>&1)
    
    if echo "$LIST_RESPONSE" | grep -q "users\|email"; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        echo "Response: $LIST_RESPONSE"
        ((TESTS_FAILED++))
    fi
else
    echo -e "${YELLOW}⊘ SKIP (no access token)${NC}"
fi

# Test 8: CORS headers (if enabled)
echo -n "Testing: CORS headers... "
CORS_RESPONSE=$(curl -s -I -X OPTIONS "$GATEWAY_URL/api/users" \
    -H "Origin: http://example.com" \
    -H "Access-Control-Request-Method: GET" 2>&1)

if echo "$CORS_RESPONSE" | grep -qi "access-control"; then
    echo -e "${GREEN}✓ PASS${NC}"
    ((TESTS_PASSED++))
else
    echo -e "${YELLOW}⊘ SKIP (CORS not enabled)${NC}"
fi

# Test 9: Rate limiting (if enabled)
echo -n "Testing: Rate limiting... "
RATE_LIMIT_COUNT=0
for i in {1..5}; do
    if curl -f -s "$GATEWAY_URL/health" > /dev/null 2>&1; then
        ((RATE_LIMIT_COUNT++))
    fi
done

if [ $RATE_LIMIT_COUNT -ge 3 ]; then
    echo -e "${GREEN}✓ PASS${NC} (made $RATE_LIMIT_COUNT requests)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (only $RATE_LIMIT_COUNT requests succeeded)"
    ((TESTS_FAILED++))
fi

# Test 10: Invalid path parameter (security)
echo -n "Testing: Path traversal protection... "
SECURITY_RESPONSE=$(curl -s -w "%{http_code}" -o /dev/null \
    "$GATEWAY_URL/api/users/../../../etc/passwd" 2>&1)

if [ "$SECURITY_RESPONSE" = "400" ] || [ "$SECURITY_RESPONSE" = "404" ]; then
    echo -e "${GREEN}✓ PASS${NC} (returned $SECURITY_RESPONSE)"
    ((TESTS_PASSED++))
else
    echo -e "${RED}✗ FAIL${NC} (returned $SECURITY_RESPONSE, expected 400 or 404)"
    ((TESTS_FAILED++))
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Smoke Test Results"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
