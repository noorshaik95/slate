#!/bin/bash

# Test script for distributed tracing between API Gateway and User Auth Service

set -e

echo "=========================================="
echo "Distributed Tracing Test"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if services are running
echo -e "${YELLOW}Checking if services are running...${NC}"
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo -e "${RED}API Gateway is not running on port 8080${NC}"
    exit 1
fi
echo -e "${GREEN}✓ API Gateway is running${NC}"

if ! curl -s http://localhost:3200/ready > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠ Tempo might not be ready yet (this is okay)${NC}"
else
    echo -e "${GREEN}✓ Tempo is running${NC}"
fi
echo ""

# Make a test request to generate traces
echo -e "${YELLOW}Making test request to generate traces...${NC}"
echo "POST /api/auth/register"

RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "trace-test@example.com",
    "password": "TestPassword123!",
    "first_name": "Trace",
    "last_name": "Test",
    "phone": "+1234567890"
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
BODY=$(echo "$RESPONSE" | head -n-1)

if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 409 ]; then
    echo -e "${GREEN}✓ Request successful (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
else
    echo -e "${RED}✗ Request failed (HTTP $HTTP_CODE)${NC}"
    echo "Response: $BODY"
fi
echo ""

# Wait for traces to be exported
echo -e "${YELLOW}Waiting 5 seconds for traces to be exported to Tempo...${NC}"
sleep 5
echo ""

# Instructions for viewing traces
echo "=========================================="
echo -e "${GREEN}Trace Generation Complete!${NC}"
echo "=========================================="
echo ""
echo "To view the distributed traces:"
echo ""
echo "1. Open Grafana: http://localhost:3000"
echo "2. Go to Explore (compass icon on left sidebar)"
echo "3. Select 'Tempo' as the data source"
echo "4. Use one of these TraceQL queries:"
echo ""
echo -e "${YELLOW}   Query 1: Find traces from api-gateway${NC}"
echo '   { resource.service.name = "api-gateway" }'
echo ""
echo -e "${YELLOW}   Query 2: Find traces from user-auth-service${NC}"
echo '   { resource.service.name = "user-auth-service" }'
echo ""
echo -e "${YELLOW}   Query 3: Find traces for Register method${NC}"
echo '   { span.name = "user.UserService/Register" }'
echo ""
echo -e "${YELLOW}   Query 4: Find all traces (last 5 minutes)${NC}"
echo '   { }'
echo ""
echo "5. Click on a trace to see the full span tree"
echo "6. You should see spans from both services connected!"
echo ""
echo "=========================================="
echo ""
