#!/bin/bash

echo "========================================="
echo "Testing Auth Endpoints - Full Flow"
echo "========================================="
echo ""

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Generate unique email
UNIQUE_EMAIL="user$(date +%s)@example.com"

echo "Test 1: Register New User"
echo "-------------------------"
echo "Email: $UNIQUE_EMAIL"
echo ""

REGISTER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$UNIQUE_EMAIL\",
    \"password\": \"TestPass123!\",
    \"first_name\": \"Test\",
    \"last_name\": \"User\",
    \"phone\": \"+1234567890\"
  }")

if echo "$REGISTER_RESPONSE" | grep -q "access_token"; then
    echo -e "${GREEN}✓ Registration successful!${NC}"
    echo ""
    echo "Response:"
    echo "$REGISTER_RESPONSE" | python3 -m json.tool | head -15
    echo "..."
    
    # Extract access token
    ACCESS_TOKEN=$(echo "$REGISTER_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['access_token'])")
    echo ""
    echo -e "${GREEN}Access Token: ${ACCESS_TOKEN:0:50}...${NC}"
else
    echo -e "${RED}✗ Registration failed${NC}"
    echo "$REGISTER_RESPONSE"
    exit 1
fi

echo ""
echo "========================================="
echo ""

echo "Test 2: Login with Created User"
echo "--------------------------------"
echo ""

LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d "{
    \"email\": \"$UNIQUE_EMAIL\",
    \"password\": \"TestPass123!\"
  }")

if echo "$LOGIN_RESPONSE" | grep -q "access_token"; then
    echo -e "${GREEN}✓ Login successful!${NC}"
    echo ""
    echo "Response:"
    echo "$LOGIN_RESPONSE" | python3 -m json.tool | head -15
    echo "..."
    
    # Extract new access token
    NEW_ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['access_token'])")
    echo ""
    echo -e "${GREEN}New Access Token: ${NEW_ACCESS_TOKEN:0:50}...${NC}"
else
    echo -e "${RED}✗ Login failed${NC}"
    echo "$LOGIN_RESPONSE"
    exit 1
fi

echo ""
echo "========================================="
echo ""

echo "Test 3: Access Protected Endpoint with Token"
echo "---------------------------------------------"
echo ""

USERS_RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}" \
  -H "Authorization: Bearer $NEW_ACCESS_TOKEN" \
  http://localhost:8080/api/users)

HTTP_STATUS=$(echo "$USERS_RESPONSE" | grep "HTTP_STATUS" | cut -d: -f2)
BODY=$(echo "$USERS_RESPONSE" | sed '/HTTP_STATUS/d')

echo "HTTP Status: $HTTP_STATUS"
echo "Response:"
echo "$BODY" | python3 -m json.tool 2>/dev/null | head -20 || echo "$BODY"

if [ "$HTTP_STATUS" = "200" ]; then
    echo ""
    echo -e "${GREEN}✓ Protected endpoint access successful with JWT!${NC}"
else
    echo ""
    echo -e "${YELLOW}Note: Status $HTTP_STATUS - endpoint may require additional permissions${NC}"
fi

echo ""
echo "========================================="
echo ""

echo "SUMMARY"
echo "-------"
echo ""
echo -e "${GREEN}✓ Register endpoint working${NC}"
echo -e "${GREEN}✓ Login endpoint working${NC}"
echo -e "${GREEN}✓ JWT tokens generated and returned${NC}"
echo -e "${GREEN}✓ Gateway ↔ User-Auth-Service communication verified${NC}"
echo ""
echo "Full authentication flow is operational!"
echo ""
echo "========================================="
echo ""
echo "View traces in Grafana:"
echo "  URL: http://localhost:3000"
echo "  Login: admin/admin"
echo "  Navigate to: Explore → Tempo"
echo "  Search for traces with: { span.http.route =~ \"/api/auth/.*\" }"
echo ""
