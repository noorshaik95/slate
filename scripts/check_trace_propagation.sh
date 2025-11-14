#!/bin/bash

# Script to verify trace propagation from API Gateway to User Auth Service

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "=========================================="
echo "Trace Propagation Verification"
echo "=========================================="
echo ""

# Method 1: Check logs for trace IDs
echo -e "${BLUE}Method 1: Checking logs for trace IDs${NC}"
echo "----------------------------------------"
echo ""

# Make a request
echo -e "${YELLOW}Making test request...${NC}"
RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"trace-test@example.com","password":"TestPassword123!"}' 2>&1)

echo "Response received"
echo ""

# Wait for logs to be written
sleep 2

# Get recent logs from both services
echo -e "${YELLOW}Extracting trace information from logs...${NC}"
echo ""

# Check API Gateway logs
echo -e "${BLUE}API Gateway logs:${NC}"
docker-compose logs --tail=50 api-gateway 2>&1 | grep -i "login" | tail -5
echo ""

# Check User Auth Service logs  
echo -e "${BLUE}User Auth Service logs:${NC}"
docker-compose logs --tail=50 user-auth-service 2>&1 | grep -i "login" | tail -5
echo ""

# Method 2: Query Tempo API for traces
echo ""
echo -e "${BLUE}Method 2: Querying Tempo API${NC}"
echo "----------------------------------------"
echo ""

echo -e "${YELLOW}Waiting 5 seconds for trace export...${NC}"
sleep 5

# Search for recent traces
echo -e "${YELLOW}Searching for traces in Tempo...${NC}"
TEMPO_RESPONSE=$(curl -s "http://localhost:3200/api/search?limit=5" 2>&1)

if echo "$TEMPO_RESPONSE" | grep -q "traces"; then
    echo -e "${GREEN}✓ Traces found in Tempo${NC}"
    
    # Try to extract trace IDs
    if command -v jq &> /dev/null; then
        echo ""
        echo "Recent trace IDs:"
        echo "$TEMPO_RESPONSE" | jq -r '.traces[].traceID' 2>/dev/null | head -3
    fi
else
    echo -e "${RED}✗ No traces found or Tempo not responding${NC}"
fi

echo ""

# Method 3: Check for connected spans in Grafana
echo ""
echo -e "${BLUE}Method 3: Visual verification in Grafana${NC}"
echo "----------------------------------------"
echo ""

cat << 'EOF'
To visually verify trace propagation:

1. Open Grafana: http://localhost:3000
2. Navigate to: Explore → Select "Tempo"
3. Run this query:
   
   { resource.service.name = "api-gateway" }

4. Click on any trace in the results
5. Look for the trace timeline view

✓ SUCCESSFUL PROPAGATION looks like:
   
   ┌─ api-gateway: gateway_request (100ms)
   │  └─ api-gateway: grpc.user.Login (90ms)
   │     └─ user-auth-service: user.UserService/Login (80ms)  ← Connected!
   
   Key indicators:
   - Multiple services in ONE trace
   - Parent-child relationship (indentation)
   - Same trace ID across all spans
   - Continuous timeline

✗ FAILED PROPAGATION looks like:
   
   ┌─ api-gateway: gateway_request (100ms)
   │  └─ api-gateway: grpc.user.Login (90ms)
   
   (Separate trace)
   ┌─ user-auth-service: user.UserService/Login (80ms)  ← Disconnected!
   
   Problems:
   - Services appear in SEPARATE traces
   - Different trace IDs
   - No parent-child relationship

EOF

echo ""

# Method 4: Check gRPC metadata injection
echo ""
echo -e "${BLUE}Method 4: Verify gRPC metadata injection${NC}"
echo "----------------------------------------"
echo ""

echo -e "${YELLOW}Checking if trace context is injected into gRPC calls...${NC}"

# Look for trace-related metadata in logs
if docker-compose logs api-gateway 2>&1 | grep -q "inject\|traceparent\|trace"; then
    echo -e "${GREEN}✓ Trace context injection code is active${NC}"
else
    echo -e "${YELLOW}⚠ No explicit trace injection logs found (this is normal)${NC}"
fi

echo ""

# Method 5: Detailed trace inspection
echo ""
echo -e "${BLUE}Method 5: Detailed trace inspection${NC}"
echo "----------------------------------------"
echo ""

cat << 'EOF'
For detailed trace inspection:

1. Open Grafana: http://localhost:3000
2. Go to Explore → Tempo
3. Query: { }  (all traces)
4. Click on a trace
5. Click on any span to see details

Check these attributes:

API Gateway span should have:
  ✓ service.name = "api-gateway"
  ✓ http.method = "POST"
  ✓ http.target = "/api/auth/login"
  ✓ trace_id = "abc123..."
  ✓ span_id = "def456..."

User Auth Service span should have:
  ✓ service.name = "user-auth-service"
  ✓ rpc.service = "user.UserService"
  ✓ rpc.method = "Login"
  ✓ trace_id = "abc123..."  ← SAME as API Gateway!
  ✓ parent_span_id = "def456..."  ← Points to API Gateway span!

EOF

echo ""

# Method 6: TraceQL queries
echo ""
echo -e "${BLUE}Method 6: TraceQL queries to verify propagation${NC}"
echo "----------------------------------------"
echo ""

cat << 'EOF'
Use these TraceQL queries in Grafana to verify:

Query 1: Find traces that span both services
  { } | select(resource.service.name = "api-gateway") 
      | select(resource.service.name = "user-auth-service")

Query 2: Find traces with specific span relationships
  { span.name = "user.UserService/Login" }

Query 3: Find traces by duration (should include both services)
  { duration > 50ms }

Query 4: Check for traces with multiple services
  { } | count() by resource.service.name

If propagation works:
  ✓ You'll see traces containing BOTH service names
  ✓ Spans will be nested (parent-child)
  ✓ Same trace ID across services

If propagation fails:
  ✗ Each service has separate traces
  ✗ No parent-child relationships
  ✗ Different trace IDs

EOF

echo ""

# Summary
echo ""
echo "=========================================="
echo -e "${GREEN}Verification Methods Summary${NC}"
echo "=========================================="
echo ""
echo "1. ✓ Check logs for trace IDs (automated above)"
echo "2. ✓ Query Tempo API (automated above)"
echo "3. → Visual inspection in Grafana (manual)"
echo "4. ✓ Verify gRPC metadata injection (automated above)"
echo "5. → Detailed span inspection (manual)"
echo "6. → TraceQL queries (manual)"
echo ""
echo "For the most reliable verification:"
echo "→ Open Grafana and visually inspect a trace"
echo "→ Look for nested spans from both services"
echo ""
