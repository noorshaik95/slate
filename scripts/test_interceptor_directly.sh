#!/bin/bash

echo "Testing if gRPC interceptor is being called..."
echo ""

# Make a request
echo "1. Making gRPC request via API Gateway..."
curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"trace-test@example.com","password":"TestPassword123!"}' > /dev/null

echo "✓ Request sent"
echo ""

# Check if debug file was created
echo "2. Checking for debug file in container..."
if docker-compose exec -T user-auth-service test -f /tmp/grpc-trace-debug.log 2>/dev/null; then
    echo "✓ Debug file EXISTS!"
    echo ""
    echo "3. File contents:"
    docker-compose exec -T user-auth-service cat /tmp/grpc-trace-debug.log
else
    echo "✗ Debug file DOES NOT EXIST"
    echo ""
    echo "This means the interceptor is NOT being called!"
    echo ""
    echo "Possible reasons:"
    echo "  1. Interceptor not registered correctly"
    echo "  2. Wrong interceptor type (Unary vs Stream)"
    echo "  3. gRPC server configuration issue"
    echo "  4. Requests not reaching the Go service"
fi

echo ""
echo "4. Checking stderr logs for [INTERCEPTOR] marker..."
if docker-compose logs --since=30s user-auth-service 2>&1 | grep "\[INTERCEPTOR\]"; then
    echo "✓ Found interceptor logs in stderr"
else
    echo "✗ No interceptor logs in stderr"
fi
