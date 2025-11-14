#!/bin/bash

# Load test script to create users and test rate limiting
# Requires: grpcurl (install with: brew install grpcurl)

set -e

GRPC_HOST="localhost:50051"
NUM_USERS=10000
RATE_LIMIT_TEST_ATTEMPTS=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Statistics
REGISTER_SUCCESS=0
REGISTER_FAILED=0
REGISTER_RATE_LIMITED=0
LOGIN_SUCCESS=0
LOGIN_FAILED=0
LOGIN_RATE_LIMITED=0

echo "============================================================"
echo "USER LOAD TEST & RATE LIMIT VERIFICATION"
echo "============================================================"
echo "Start time: $(date '+%Y-%m-%d %H:%M:%S')"
echo "Target: $NUM_USERS users"
echo "gRPC Server: $GRPC_HOST"
echo "============================================================"
echo ""

# Check if grpcurl is installed
if ! command -v grpcurl &> /dev/null; then
    echo -e "${RED}Error: grpcurl is not installed${NC}"
    echo "Install with: brew install grpcurl (macOS) or go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest"
    exit 1
fi

# Check if server is reachable
echo "Checking gRPC server connectivity..."
if grpcurl -plaintext $GRPC_HOST list > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Connected to gRPC server${NC}"
else
    echo -e "${RED}✗ Cannot connect to gRPC server at $GRPC_HOST${NC}"
    exit 1
fi

echo ""

# Function to generate random string
random_string() {
    local length=$1
    LC_ALL=C tr -dc 'a-z0-9' < /dev/urandom | head -c $length
}

# Function to register a user
register_user() {
    local index=$1
    local email="user${index}_$(random_string 6)@loadtest.com"
    local password="Pass$(random_string 8)!@"
    local first_name="User"
    local last_name="LoadTest${index}"
    
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
        echo "failed|$email|$password|$response"
        return 1
    fi
}

# Function to login a user
login_user() {
    local email=$1
    local password=$2
    
    local response=$(grpcurl -plaintext -d "{
        \"email\": \"$email\",
        \"password\": \"$password\"
    }" $GRPC_HOST user.UserService/Login 2>&1)
    
    if echo "$response" | grep -q "RESOURCE_EXHAUSTED"; then
        return 2  # Rate limited
    elif echo "$response" | grep -q "accessToken"; then
        return 0  # Success
    else
        return 1  # Failed
    fi
}

# Test rate limiting on registration
test_register_rate_limit() {
    echo "============================================================"
    echo "Phase 1: Testing Registration Rate Limiting"
    echo "============================================================"
    echo "Attempting $RATE_LIMIT_TEST_ATTEMPTS rapid registrations..."
    echo "Expected limit: 3 per hour"
    echo ""
    
    local success=0
    local rate_limited=0
    local failed=0
    
    for i in $(seq 1 $RATE_LIMIT_TEST_ATTEMPTS); do
        result=$(register_user "ratelimit_test_$i")
        status=$(echo "$result" | cut -d'|' -f1)
        
        case $status in
            success)
                success=$((success + 1))
                echo -e "  [$i] ${GREEN}✓ Success${NC}"
                ;;
            rate_limited)
                rate_limited=$((rate_limited + 1))
                echo -e "  [$i] ${RED}❌ Rate limited${NC} (expected after 3 attempts)"
                ;;
            failed)
                failed=$((failed + 1))
                echo -e "  [$i] ${YELLOW}✗ Failed${NC}"
                ;;
        esac
        
        sleep 0.1
    done
    
    echo ""
    echo "Rate Limiting Test Results:"
    echo "  Successful: $success"
    echo "  Rate Limited: $rate_limited"
    echo "  Failed: $failed"
    echo "  Expected: First 3 succeed, rest rate limited"
    
    if [ $rate_limited -gt 0 ]; then
        echo -e "  ${GREEN}✓ Rate limiting is WORKING${NC}"
    else
        echo -e "  ${YELLOW}⚠ Rate limiting may not be working${NC}"
    fi
    echo "============================================================"
    echo ""
}

# Test rate limiting on login
test_login_rate_limit() {
    echo "============================================================"
    echo "Phase 2: Testing Login Rate Limiting"
    echo "============================================================"
    echo "Creating test user for login rate limit test..."
    
    # Create a test user
    result=$(register_user "login_test_user")
    status=$(echo "$result" | cut -d'|' -f1)
    
    if [ "$status" != "success" ]; then
        echo -e "${YELLOW}⚠ Could not create test user, skipping login rate limit test${NC}"
        echo "============================================================"
        echo ""
        return
    fi
    
    email=$(echo "$result" | cut -d'|' -f2)
    password=$(echo "$result" | cut -d'|' -f3)
    
    echo "Test user created: $email"
    echo "Attempting $RATE_LIMIT_TEST_ATTEMPTS rapid logins..."
    echo "Expected limit: 5 per 15 minutes"
    echo ""
    
    local success=0
    local rate_limited=0
    local failed=0
    
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
                echo -e "  [$i] ${RED}❌ Rate limited${NC} (expected after 5 attempts)"
                ;;
            1)
                failed=$((failed + 1))
                echo -e "  [$i] ${YELLOW}✗ Failed${NC}"
                ;;
        esac
        
        sleep 0.1
    done
    
    echo ""
    echo "Rate Limiting Test Results:"
    echo "  Successful: $success"
    echo "  Rate Limited: $rate_limited"
    echo "  Failed: $failed"
    echo "  Expected: First 5 succeed, rest rate limited"
    
    if [ $rate_limited -gt 0 ]; then
        echo -e "  ${GREEN}✓ Rate limiting is WORKING${NC}"
    else
        echo -e "  ${YELLOW}⚠ Rate limiting may not be working${NC}"
    fi
    echo "============================================================"
    echo ""
}

# Create bulk users
create_bulk_users() {
    echo "============================================================"
    echo "Phase 3: Bulk User Creation"
    echo "============================================================"
    echo "Creating $NUM_USERS users..."
    echo ""
    
    START_TIME=$(date +%s)
    
    for i in $(seq 1 $NUM_USERS); do
        result=$(register_user $i)
        status=$(echo "$result" | cut -d'|' -f1)
        
        case $status in
            success)
                REGISTER_SUCCESS=$((REGISTER_SUCCESS + 1))
                ;;
            rate_limited)
                REGISTER_RATE_LIMITED=$((REGISTER_RATE_LIMITED + 1))
                ;;
            failed)
                REGISTER_FAILED=$((REGISTER_FAILED + 1))
                ;;
        esac
        
        # Progress update every 100 users
        if [ $((i % 100)) -eq 0 ]; then
            ELAPSED=$(($(date +%s) - START_TIME))
            RATE=$(echo "scale=1; $i / $ELAPSED" | bc)
            echo "Progress: $i/$NUM_USERS users (${RATE} req/s, ${ELAPSED}s elapsed)"
        fi
    done
    
    END_TIME=$(date +%s)
    TOTAL_TIME=$((END_TIME - START_TIME))
    AVG_RATE=$(echo "scale=1; $NUM_USERS / $TOTAL_TIME" | bc)
    
    echo ""
    echo "User Creation Complete!"
    echo "Total time: ${TOTAL_TIME}s"
    echo "Average rate: ${AVG_RATE} req/s"
    echo "Successful: $REGISTER_SUCCESS"
    echo "Failed: $REGISTER_FAILED"
    echo "Rate Limited: $REGISTER_RATE_LIMITED"
    echo "============================================================"
    echo ""
}

# Print summary
print_summary() {
    echo "============================================================"
    echo "LOAD TEST SUMMARY"
    echo "============================================================"
    echo "Registration Stats:"
    echo "  Success: $REGISTER_SUCCESS"
    echo "  Failed: $REGISTER_FAILED"
    echo "  Rate Limited: $REGISTER_RATE_LIMITED"
    echo ""
    echo "Login Stats:"
    echo "  Success: $LOGIN_SUCCESS"
    echo "  Failed: $LOGIN_FAILED"
    echo "  Rate Limited: $LOGIN_RATE_LIMITED"
    echo "============================================================"
    echo "End time: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "============================================================"
}

# Main execution
main() {
    # Test rate limiting
    test_register_rate_limit
    
    echo "Waiting 2 seconds before login rate limit test..."
    sleep 2
    
    test_login_rate_limit
    
    # Ask user if they want to proceed with bulk creation
    echo "============================================================"
    read -p "Proceed with creating $NUM_USERS users? (yes/no): " answer
    echo "============================================================"
    echo ""
    
    if [[ "$answer" == "yes" || "$answer" == "y" ]]; then
        create_bulk_users
    else
        echo "Skipping bulk user creation."
    fi
    
    # Print summary
    print_summary
}

# Run main function
main
