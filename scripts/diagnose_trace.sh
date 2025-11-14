#!/bin/bash

# Diagnostic script to check if a specific trace propagated correctly

TRACE_ID="$1"

if [ -z "$TRACE_ID" ]; then
    echo "Usage: ./diagnose_trace.sh <trace_id>"
    echo "Example: ./diagnose_trace.sh 18a2005378209ebdba590df2e93014a8"
    exit 1
fi

echo "=========================================="
echo "Diagnosing Trace: $TRACE_ID"
echo "=========================================="
echo ""

# Query Tempo for the specific trace
echo "Querying Tempo for trace details..."
TRACE_DATA=$(curl -s "http://localhost:3200/api/traces/$TRACE_ID")

if echo "$TRACE_DATA" | grep -q "error\|not found"; then
    echo "❌ Trace not found in Tempo"
    echo "$TRACE_DATA"
    exit 1
fi

echo "✓ Trace found in Tempo"
echo ""

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "⚠ jq not installed, showing raw JSON"
    echo "$TRACE_DATA"
    exit 0
fi

# Extract services
echo "Services in this trace:"
echo "$TRACE_DATA" | jq -r '.batches[].resource.attributes[] | select(.key == "service.name") | .value.stringValue' | sort -u | while read service; do
    echo "  - $service"
done
echo ""

# Count spans per service
echo "Span count by service:"
for service in $(echo "$TRACE_DATA" | jq -r '.batches[].resource.attributes[] | select(.key == "service.name") | .value.stringValue' | sort -u); do
    count=$(echo "$TRACE_DATA" | jq -r ".batches[] | select(.resource.attributes[] | select(.key == \"service.name\" and .value.stringValue == \"$service\")) | .scopeSpans[].spans[]" | jq -s 'length')
    echo "  $service: $count spans"
done
echo ""

# Check for parent-child relationships
echo "Checking span relationships..."
echo "$TRACE_DATA" | jq -r '.batches[].scopeSpans[].spans[] | "\(.name) (span_id: \(.spanId | @base64d | .[0:8])) parent: \(.parentSpanId | @base64d | .[0:8])"' | head -10
echo ""

# Verdict
SERVICE_COUNT=$(echo "$TRACE_DATA" | jq -r '.batches[].resource.attributes[] | select(.key == "service.name") | .value.stringValue' | sort -u | wc -l)

echo "=========================================="
if [ "$SERVICE_COUNT" -ge 2 ]; then
    echo "✅ PROPAGATION SUCCESSFUL"
    echo "   Trace contains spans from $SERVICE_COUNT services"
else
    echo "❌ PROPAGATION FAILED"
    echo "   Trace only contains spans from $SERVICE_COUNT service(s)"
    echo "   Expected: 2 (api-gateway + user-auth-service)"
fi
echo "=========================================="
