#!/bin/bash

echo "=== Testing Periodic Refresh with Route Overrides ==="
echo ""

echo "Step 1: Verify gateway is running..."
if ! docker-compose ps | grep -q "api-gateway.*Up"; then
    echo "ERROR: API Gateway is not running"
    exit 1
fi
echo "✓ Gateway is running"
echo ""

echo "Step 2: Test override route at startup (should work)..."
response=$(curl -s -X POST http://localhost:8080/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test"}')

if echo "$response" | grep -q "BACKEND_ERROR\|Unauthenticated"; then
    echo "✓ Override route /api/auth/login is working (reached backend)"
else
    echo "✗ Override route /api/auth/login failed"
    echo "Response: $response"
fi
echo ""

echo "Step 3: Check startup logs for override application..."
startup_overrides=$(docker-compose logs api-gateway | grep "Applying route overrides" | tail -1)
if [ -n "$startup_overrides" ]; then
    echo "✓ Found override application in startup logs:"
    echo "  $startup_overrides"
else
    echo "✗ No override application found in startup logs"
fi
echo ""

echo "Step 4: Waiting for periodic refresh (checking every 30 seconds)..."
echo "Note: Refresh interval is 300 seconds (5 minutes)"
echo "You can also manually restart the gateway to see a fresh cycle"
echo ""

# Get the current timestamp from logs
last_refresh=$(docker-compose logs api-gateway | grep "Route refresh cycle completed" | tail -1 | grep -oE '[0-9]+\.[0-9]+s' | head -1)
echo "Last refresh was at: ${last_refresh:-startup}"
echo ""

# Monitor for next refresh
echo "Monitoring logs for next refresh cycle..."
echo "Press Ctrl+C to stop monitoring"
echo ""

docker-compose logs -f api-gateway 2>&1 | grep --line-buffered -E "(Starting periodic route refresh cycle|Route refresh cycle completed|Applying route overrides|Added new route from override)"
