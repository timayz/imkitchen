#!/bin/bash
set -euo pipefail

# Deployment script for imkitchen application
# Usage: ./deploy.sh [staging|production] [image_tag]

ENVIRONMENT=${1:-staging}
IMAGE_TAG=${2:-latest}
REGISTRY=${REGISTRY:-"ghcr.io"}
IMAGE_NAME=${IMAGE_NAME:-"snapiz/imkitchen"}

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
    echo "Error: Environment must be 'staging' or 'production'"
    exit 1
fi

echo "🚀 Deploying imkitchen to $ENVIRONMENT environment"
echo "📦 Image: $REGISTRY/$IMAGE_NAME:$IMAGE_TAG"

# Set environment-specific variables
if [[ "$ENVIRONMENT" == "staging" ]]; then
    DEPLOY_URL="https://staging.imkitchen.app"
    CONFIG_FILE="config/staging.toml"
    DB_PATH="/data/staging/imkitchen.db"
elif [[ "$ENVIRONMENT" == "production" ]]; then
    DEPLOY_URL="https://imkitchen.app"
    CONFIG_FILE="config/production.toml"
    DB_PATH="/data/production/imkitchen.db"
fi

echo "🔧 Environment: $ENVIRONMENT"
echo "🌐 URL: $DEPLOY_URL"
echo "📋 Config: $CONFIG_FILE"

# Create backup of current database (production only)
if [[ "$ENVIRONMENT" == "production" && -f "$DB_PATH" ]]; then
    echo "💾 Creating database backup..."
    BACKUP_PATH="/data/backups/imkitchen-$(date +%Y%m%d_%H%M%S).db"
    mkdir -p "/data/backups"
    cp "$DB_PATH" "$BACKUP_PATH"
    echo "✅ Backup created: $BACKUP_PATH"
fi

# Run database migrations
echo "🔄 Running database migrations..."
docker run --rm \
    -v "/data/$ENVIRONMENT:/data" \
    -v "$(pwd)/migrations:/app/migrations" \
    -e DATABASE_URL="sqlite:/data/imkitchen.db" \
    "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG" \
    sh -c "cargo install sqlx-cli --no-default-features --features sqlite && sqlx migrate run"

# Deploy the application
echo "🚀 Deploying application..."

# Stop existing container if running
if docker ps -q -f name="imkitchen-$ENVIRONMENT" | grep -q .; then
    echo "🛑 Stopping existing container..."
    docker stop "imkitchen-$ENVIRONMENT"
    docker rm "imkitchen-$ENVIRONMENT"
fi

# Start new container
echo "▶️  Starting new container..."
docker run -d \
    --name "imkitchen-$ENVIRONMENT" \
    --restart unless-stopped \
    -p "$([ "$ENVIRONMENT" == "production" ] && echo "3000:3000" || echo "3001:3000")" \
    -v "/data/$ENVIRONMENT:/data" \
    -v "$(pwd)/$CONFIG_FILE:/app/config/local.toml" \
    -e DATABASE_URL="sqlite:/data/imkitchen.db" \
    -e RUST_LOG="info" \
    -e ENVIRONMENT="$ENVIRONMENT" \
    "$REGISTRY/$IMAGE_NAME:$IMAGE_TAG"

echo "⏳ Waiting for application to start..."
sleep 10

# Health check
echo "🏥 Running health check..."
HEALTH_URL="http://localhost:$([ "$ENVIRONMENT" == "production" ] && echo "3000" || echo "3001")/health"

for i in {1..30}; do
    if curl -f -s "$HEALTH_URL" > /dev/null; then
        echo "✅ Health check passed"
        echo "🎉 Deployment to $ENVIRONMENT completed successfully!"
        
        # Show deployment info
        echo ""
        echo "📊 Deployment Summary:"
        echo "   Environment: $ENVIRONMENT"
        echo "   Image: $REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
        echo "   URL: $DEPLOY_URL"
        echo "   Container: imkitchen-$ENVIRONMENT"
        
        # Test the health endpoint
        echo ""
        echo "🔍 Health Check Response:"
        curl -s "$HEALTH_URL" | jq . || curl -s "$HEALTH_URL"
        
        exit 0
    fi
    
    echo "⏳ Waiting for health check... ($i/30)"
    sleep 2
done

echo "❌ Health check failed after 60 seconds"
echo "📋 Container logs:"
docker logs "imkitchen-$ENVIRONMENT" --tail 50

# Rollback on failure (production only)
if [[ "$ENVIRONMENT" == "production" && -f "$BACKUP_PATH" ]]; then
    echo "🔄 Rolling back to previous version..."
    docker stop "imkitchen-$ENVIRONMENT" || true
    docker rm "imkitchen-$ENVIRONMENT" || true
    
    # Restore backup
    cp "$BACKUP_PATH" "$DB_PATH"
    echo "💾 Database restored from backup"
fi

exit 1