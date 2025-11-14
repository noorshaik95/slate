#!/bin/bash

# Quick rate limit test script
# Tests rate limiting without creating 10k users

set -e

GRPC_HOST="localhost:50051"
RATE_LIMIT_TEST_ATTEMPTS=10

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "============================================================"
echo "QUICK RATE LIMIT TEST"
echo "============================================================"
echo "gRPC Server: $GRPC_HOST"
echo "============================================================"
echo ""

# Check grpcurl
if ! command -v grpcurl &> /dev/null; then
    echo -e "${RED}Error: grpcurl not installed${NC}"
    echo "Install: brew install grpcurl"
    exit 1
fi

# Check connectivity
echo "Checking server..."
if ! grpcurl -plaintext $GRPC_HOST list > /dev/null 2>&1; then
    echo -e "${RED}✗ Cannot connect to $GRPC_HOST${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Connected${NC}"
echo ""

# Random string generator
random_string() {
    LC_ALL=C tr -dc 'a-z0-9' < /dev/urandom | head -c $1
}

# Register user
register_user() {
    local index=$1
    local email="user${index}_$(random_string 6)@test.com"
    local password="Pass$(random_string 8)!@"
    local first_name="User"
    local last_name="Test${index}"
    
    local response=$(grpcurl -plaintext -d "{
        \"email\": \"$email\",
        \"password\": \"$password\",
        \"first_name\": \"$first_name\",
        \"last_name\": \"$last_name\"
    }" $GRPC_HOST user.UserService/Register 2>&1)
    
    if echo "$response" | grep -q "RESOURCE_EXHAUSTED"; then
        echo "rate_limited|$email|$password"
        return 2
    elif echo "$response" | grep -q "\"id\""; then
        echo "success|$email|$password"
        return 0
    else
        echo "failed|$email|$password"
        return 1
    fi
}

# Login user
login_user() {
    local email=$1
    local password=$2
    
    local response=$(grpcurl -plaintext -d "{
        \"email\": \"$email\",
        \"password\": \"$password\"
    }" $GRPC_HOST user.UserService/Login 2>&1)
    
    if echo "$response" | grep -q "RESOURCE_EXHAUSTED"; then
        return 2
    elif echo "$response" | grep -q "accessToken"; then
        return 0
    else
        return 1
    fi
}

# Test registration rate limit
echo "============================================================"
echo "TEST 1: Registration Rate Limiting"
echo "============================================================"
echo "Limit: 3 per hour"
echo "Attempting $RATE_LIMIT_TEST_ATTEMPTS registrations..."
echo ""

success=0
rate_limited=0
failed=0

for i in $(seq 1 $RATE_LIMIT_TEST_ATTEMPTS); do
    result=$(register_user "test_$i")
    status=$(echo "$result" | cut -d'|' -f1)
    
    case $status in
        success)
            success=$((success + 1))
            echo -e "  [$i] ${GREEN}✓ Success${NC}"
            ;;
        rate_limited)
            rate_limited=$((rate_limited + 1))
            echo -e "  [$i] ${RED}❌ Rate limited${NC}"
            ;;
        failed)
            failed=$((failed + 1))
            echo -e "  [$i] ${YELLOW}✗ Failed${NC}"
            ;;
    esac
    
    sleep 0.2
done

echo ""
echo "Results:"
echo "  Success: $success"
echo "  Rate Limited: $rate_limited"
echo "  Failed: $failed"

if [ $rate_limited -gt 0 ]; then
    echo -e "  ${GREEN}✓ Registration rate limiting WORKS${NC}"
else
    echo -e "  ${YELLOW}⚠ Rate limiting may not be enabled${NC}"
fi
echo ""

# Test login rate limit
echo "============================================================"
echo "TEST 2: Login Rate Limiting"
echo "============================================================"
echo "Limit: 5 per 15 minutes"
echo "Creating test user..."

result=$(register_user "login_test")
status=$(echo "$result" | cut -d'|' -f1)

if [ "$status" != "success" ]; then
    echo -e "${YELLOW}⚠ Could not create test user${NC}"
    exit 0
fi

email=$(echo "$result" | cut -d'|' -f2)
password=$(echo "$result" | cut -d'|' -f3)

echo "Test user: $email"
echo "Attempting $RATE_LIMIT_TEST_ATTEMPTS logins..."
echo ""

success=0
rate_limited=0
failed=0

for i in $(seq 1 $RATE_LIMIT_TEST_ATTEMPTS); do
    login_user "$email" "$password"
    result=$?
    
    case $result in
        0)
            success=$((success + 1))
            echo -e "  [$i] ${GREEN}✓ Success${NC}"
            ;;
        2)
            rate_limited=$((rate_limited + 1))
            echo -e "  [$i] ${RED}❌ Rate limited${NC}"
            ;;
        1)
            failed=$((failed + 1))
            echo -e "  [$i] ${YELLOW}✗ Failed${NC}"
            ;;
    esac
    
    sleep 0.2
done

echo ""
echo "Results:"
echo "  Success: $success"
echo "  Rate Limited: $rate_limited"
echo "  Failed: $failed"

if [ $rate_limited -gt 0 ]; then
    echo -e "  ${GREEN}✓ Login rate limiting WORKS${NC}"
else
    echo -e "  ${YELLOW}⚠ Rate limiting may not be enabled${NC}"
fi

echo ""
echo "============================================================"
echo "SUMMARY"
echo "============================================================"
echo "Registration rate limiting: $([ $rate_limited -gt 0 ] && echo -e "${GREEN}WORKING${NC}" || echo -e "${YELLOW}NOT DETECTED${NC}")"
echo "Login rate limiting: $([ $rate_limited -gt 0 ] && echo -e "${GREEN}WORKING${NC}" || echo -e "${YELLOW}NOT DETECTED${NC}")"
echo "============================================================"
