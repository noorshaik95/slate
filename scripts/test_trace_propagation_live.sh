#!/bin/bash

echo "=========================================="
echo "Live Trace Propagation Test"
echo "=========================================="
echo ""

# Make a request
echo "1. Making test request..."
RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"trace-test@example.com","password":"TestPassword123!"}')

if echo "$RESPONSE" | jq -e '.access_token' > /dev/null 2>&1; then
    echo "✓ Request successful"
    TRACE_TOKEN=$(echo "$RESPONSE" | jq -r '.access_token' | head -c 30)
    echo "  Token: ${TRACE_TOKEN}..."
else
    echo "✗ Request failed"
    echo "$RESPONSE"
    exit 1
fi

echo ""
echo "2. Waiting 5 seconds for trace export..."
sleep 5

echo ""
echo "3. Querying Tempo for recent traces..."
TRACES=$(curl -s "http://localhost:3200/api/search?limit=1")

if echo "$TRACES" | jq -e '.traces[0]' > /dev/null 2>&1; then
    TRACE_ID=$(echo "$TRACES" | jq -r '.traces[0].traceID')
    echo "✓ Found recent trace: $TRACE_ID"
    
    echo ""
    echo "4. Fetching full trace details..."
    TRACE_DATA=$(curl -s "http://localhost:3200/api/traces/$TRACE_ID")
    
    # Count services
    SERVICES=$(echo "$TRACE_DATA" | jq -r '.batches[].resource.attributes[] | select(.key == "service.name") | .value.stringValue' | sort -u)
    SERVICE_COUNT=$(echo "$SERVICES" | wc -l | tr -d ' ')
    
    echo ""
    echo "5. Services in trace:"
    echo "$SERVICES" | while read service; do
        echo "  - $service"
    done
    
    echo ""
    echo "=========================================="
    if [ "$SERVICE_COUNT" -ge 2 ]; then
        echo "✅ SUCCESS: Trace contains $SERVICE_COUNT services"
        echo "   Trace propagation is WORKING!"
    else
        echo "❌ FAILURE: Trace only contains $SERVICE_COUNT service(s)"
        echo "   Expected: 2 (api-gateway + user-auth-service)"
        echo ""
        echo "   This means trace context is NOT propagating"
        echo "   from API Gateway to User Auth Service"
    fi
    echo "=========================================="
    
    echo ""
    echo "6. Check logs manually:"
    echo "   docker-compose logs api-gateway | grep -i inject"
    echo "   docker-compose logs user-auth-service | grep TRACE-DEBUG"
    
else
    echo "✗ No traces found in Tempo"
fi
