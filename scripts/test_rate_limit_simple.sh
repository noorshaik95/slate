#!/bin/bash

# Simple rate limit test - just 6 registrations to test the limit of 3

GRPC_HOST="localhost:50051"

echo "Testing Registration Rate Limiting (limit: 3 per hour)"
echo "Attempting 6 rapid registrations..."
echo ""

for i in {1..6}; do
    email="test${i}_$(date +%s)@test.com"
    result=$(grpcurl -plaintext -d "{
        \"email\": \"$email\",
        \"password\": \"TestPass123!\",
        \"first_name\": \"Test\",
        \"last_name\": \"User$i\"
    }" $GRPC_HOST user.UserService/Register 2>&1)
    
    if echo "$result" | grep -q "ResourceExhausted"; then
        echo "[$i] ❌ RATE LIMITED - $(echo "$result" | grep "Message:" | sed 's/.*Message: //')"
    elif echo "$result" | grep -q "\"id\""; then
        echo "[$i] ✓ Success"
    else
        echo "[$i] ✗ Failed: $(echo "$result" | head -1)"
    fi
done

echo ""
echo "Expected: First 3 succeed, last 3 rate limited"
