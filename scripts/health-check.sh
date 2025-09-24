#!/bin/bash
# Comprehensive health check script for imkitchen

set -e

HOST="${1:-127.0.0.1}"
PORT="${2:-3000}"
URL="http://${HOST}:${PORT}/health"

echo "Health check for imkitchen server"
echo "URL: $URL"
echo ""

# Test if server is responding
if ! curl -f -s --max-time 5 "$URL" > /dev/null; then
    echo "❌ Server is not responding at $URL"
    exit 1
fi

# Get health response
RESPONSE=$(curl -s "$URL")

# Parse JSON and check status
STATUS=$(echo "$RESPONSE" | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
DATABASE_STATUS=$(echo "$RESPONSE" | grep -o '"database_status":"[^"]*"' | cut -d'"' -f4 2>/dev/null || echo "unknown")
VERSION=$(echo "$RESPONSE" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
UPTIME=$(echo "$RESPONSE" | grep -o '"uptime_seconds":[^,}]*' | cut -d':' -f2)

echo "✅ Server is responding"
echo "📊 Status: $STATUS"
echo "💾 Database: $DATABASE_STATUS"  
echo "📦 Version: $VERSION"
echo "⏱️  Uptime: ${UPTIME}s"
echo ""

if [ "$STATUS" = "healthy" ]; then
    echo "🎉 Health check PASSED"
    exit 0
else
    echo "⚠️  Health check FAILED - Status: $STATUS"
    exit 1
fi