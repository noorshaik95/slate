#!/bin/bash

echo "=========================================="
echo "Distributed Tracing Verification"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check services
echo -e "${YELLOW}1. Checking services...${NC}"
if curl -s http://localhost:8080/health > /dev/null; then
    echo -e "${GREEN}✓ API Gateway is running${NC}"
else
    echo -e "${RED}✗ API Gateway is not running${NC}"
    exit 1
fi

if curl -s http://localhost:3000/api/health > /dev/null; then
    echo -e "${GREEN}✓ Grafana is running${NC}"
else
    echo -e "${RED}✗ Grafana is not running${NC}"
    exit 1
fi

if curl -s http://localhost:3200/ready > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Tempo is running${NC}"
else
    echo -e "${YELLOW}⚠ Tempo might not be ready${NC}"
fi

echo ""

# Check tracing initialization
echo -e "${YELLOW}2. Checking tracing initialization...${NC}"

if docker-compose logs user-auth-service 2>&1 | grep -q "OpenTelemetry tracing initialized successfully"; then
    echo -e "${GREEN}✓ User Auth Service tracing initialized${NC}"
else
    echo -e "${RED}✗ User Auth Service tracing NOT initialized${NC}"
fi

if docker-compose logs api-gateway 2>&1 | grep -q "Initializing observability"; then
    echo -e "${GREEN}✓ API Gateway tracing initialized${NC}"
else
    echo -e "${RED}✗ API Gateway tracing NOT initialized${NC}"
fi

echo ""

# Generate a test trace
echo -e "${YELLOW}3. Generating test trace...${NC}"
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "verify-trace@example.com",
    "password": "TestPassword123!",
    "first_name": "Verify",
    "last_name": "Trace",
    "phone": "+1234567890"
  }')

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)

if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 409 ]; then
    echo -e "${GREEN}✓ Test request successful (HTTP $HTTP_CODE)${NC}"
else
    echo -e "${RED}✗ Test request failed (HTTP $HTTP_CODE)${NC}"
fi

echo ""
echo -e "${YELLOW}Waiting 10 seconds for trace export...${NC}"
sleep 10

echo ""
echo "=========================================="
echo -e "${GREEN}Verification Complete!${NC}"
echo "=========================================="
echo ""
echo "Next steps:"
echo ""
echo "1. Open Grafana: http://localhost:3000"
echo "2. Go to Explore → Select 'Tempo'"
echo "3. Try these queries:"
echo ""
echo -e "${YELLOW}   Query 1: All traces${NC}"
echo "   { }"
echo ""
echo -e "${YELLOW}   Query 2: API Gateway traces${NC}"
echo "   { resource.service.name = \"api-gateway\" }"
echo ""
echo -e "${YELLOW}   Query 3: User Auth Service traces (THE FIX!)${NC}"
echo "   { resource.service.name = \"user-auth-service\" }"
echo ""
echo -e "${YELLOW}   Query 4: Register operations${NC}"
echo "   { span.name = \"user.UserService/Register\" }"
echo ""
echo "4. Click on a trace to see spans from BOTH services!"
echo ""
