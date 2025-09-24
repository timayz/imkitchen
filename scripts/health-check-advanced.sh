#!/bin/bash
set -euo pipefail

# Advanced health check script for imkitchen deployment verification
# Usage: ./health-check-advanced.sh [environment] [timeout]

ENVIRONMENT=${1:-staging}
TIMEOUT=${2:-60}
RETRY_INTERVAL=2

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(staging|production|local)$ ]]; then
    echo "Error: Environment must be 'staging', 'production', or 'local'"
    exit 1
fi

# Set environment-specific URLs and ports
case "$ENVIRONMENT" in
    local)
        BASE_URL="http://localhost:3000"
        HEALTH_URL="$BASE_URL/health"
        ;;
    staging)
        BASE_URL="https://staging.imkitchen.app"
        HEALTH_URL="$BASE_URL/health"
        # Fallback to localhost if external URL not available
        FALLBACK_URL="http://localhost:3001/health"
        ;;
    production)
        BASE_URL="https://imkitchen.app"
        HEALTH_URL="$BASE_URL/health"
        # Fallback to localhost if external URL not available
        FALLBACK_URL="http://localhost:3000/health"
        ;;
esac

echo "🏥 Advanced health check for $ENVIRONMENT environment"
echo "   URL: $HEALTH_URL"
echo "   Timeout: ${TIMEOUT}s"
echo "   Retry interval: ${RETRY_INTERVAL}s"
echo ""

# Function to perform health check
perform_health_check() {
    local url=$1
    local response
    local http_code
    
    response=$(curl -s -w "HTTPSTATUS:%{http_code}" "$url" 2>/dev/null || echo "HTTPSTATUS:000")
    http_code=$(echo "$response" | grep -o "HTTPSTATUS:[0-9]*" | cut -d: -f2)
    body=$(echo "$response" | sed 's/HTTPSTATUS:[0-9]*$//')
    
    if [[ "$http_code" == "200" ]]; then
        echo "✅ Health check passed (HTTP $http_code)"
        echo "📋 Response:"
        echo "$body" | jq . 2>/dev/null || echo "$body"
        return 0
    else
        echo "❌ Health check failed (HTTP $http_code)"
        if [[ -n "$body" ]]; then
            echo "📋 Response:"
            echo "$body"
        fi
        return 1
    fi
}

# Function to check if service is responsive
check_service_responsive() {
    local url=$1
    echo "🔍 Checking if service is responsive..."
    
    for i in $(seq 1 $((TIMEOUT/RETRY_INTERVAL))); do
        echo "⏳ Attempt $i/$(($TIMEOUT/RETRY_INTERVAL)): Checking $url"
        
        if perform_health_check "$url"; then
            return 0
        fi
        
        if [[ $i -lt $(($TIMEOUT/RETRY_INTERVAL)) ]]; then
            echo "⏰ Waiting ${RETRY_INTERVAL}s before retry..."
            sleep $RETRY_INTERVAL
        fi
    done
    
    return 1
}

# Perform health check
if check_service_responsive "$HEALTH_URL"; then
    echo ""
    echo "🎉 Health check completed successfully!"
    
    # Additional checks for production
    if [[ "$ENVIRONMENT" == "production" ]]; then
        echo ""
        echo "🔍 Running additional production health checks..."
        
        # Check response time
        echo "⏱️  Measuring response time..."
        response_time=$(curl -o /dev/null -s -w "%{time_total}" "$HEALTH_URL")
        echo "   Response time: ${response_time}s"
        
        if (( $(echo "$response_time > 3.0" | bc -l 2>/dev/null || echo "0") )); then
            echo "⚠️  Warning: Response time exceeds 3 seconds"
        else
            echo "✅ Response time within acceptable limits"
        fi
        
        # Check SSL certificate (for HTTPS URLs)
        if [[ "$BASE_URL" == https://* ]]; then
            echo "🔒 Checking SSL certificate..."
            if curl -s --connect-timeout 10 "$BASE_URL" > /dev/null; then
                echo "✅ SSL certificate is valid"
            else
                echo "⚠️  SSL certificate check failed"
            fi
        fi
    fi
    
    exit 0
else
    # Try fallback URL for staging/production
    if [[ "$ENVIRONMENT" != "local" && -n "${FALLBACK_URL:-}" ]]; then
        echo ""
        echo "🔄 Primary health check failed, trying fallback URL..."
        echo "   Fallback URL: $FALLBACK_URL"
        
        if check_service_responsive "$FALLBACK_URL"; then
            echo ""
            echo "⚠️  Health check passed on fallback URL"
            echo "   This indicates the service is running locally but external access may be unavailable"
            exit 0
        fi
    fi
    
    echo ""
    echo "❌ Health check failed after ${TIMEOUT} seconds"
    
    # Try to get container logs if running locally
    if [[ "$ENVIRONMENT" != "production" ]]; then
        echo ""
        echo "📋 Attempting to get container logs..."
        CONTAINER_NAME="imkitchen-$ENVIRONMENT"
        
        if docker ps -q -f name="$CONTAINER_NAME" | grep -q .; then
            echo "🐳 Container logs (last 20 lines):"
            docker logs "$CONTAINER_NAME" --tail 20
        else
            echo "🐳 Container '$CONTAINER_NAME' is not running"
            echo "📋 Available containers:"
            docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
        fi
    fi
    
    exit 1
fi