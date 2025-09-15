#!/bin/bash

# Deployment script for imkitchen application
set -e

ENVIRONMENT=${1:-staging}
CONTAINER_NAME="imkitchen-${ENVIRONMENT}"
IMAGE_TAG="imkitchen:${ENVIRONMENT}-latest"

echo "=€ Deploying imkitchen to ${ENVIRONMENT} environment..."

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
    echo "L Invalid environment: $ENVIRONMENT. Must be 'staging' or 'production'"
    exit 1
fi

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "L Docker is not running. Please start Docker and try again."
    exit 1
fi

# Pre-deployment checks
echo "= Running pre-deployment checks..."

# Check if required environment variables are set
REQUIRED_VARS=(
    "DATABASE_URL"
    "NEXTAUTH_SECRET"
    "NEXTAUTH_URL"
)

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        echo "L Required environment variable $var is not set"
        exit 1
    fi
done

echo " Environment variables validated"

# Build the application
echo "<× Building application..."
./scripts/build.sh

# Build Docker image
echo "=3 Building Docker image..."
docker build -f docker/Dockerfile -t $IMAGE_TAG .

# Run database migrations
echo "=Ã Running database migrations..."
docker run --rm \
    -e DATABASE_URL="$DATABASE_URL" \
    $IMAGE_TAG \
    npx prisma migrate deploy

# Health check function
health_check() {
    local max_attempts=30
    local attempt=1
    
    echo "<å Performing health check..."
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f http://localhost:3000/api/health > /dev/null 2>&1; then
            echo " Health check passed!"
            return 0
        fi
        
        echo "ó Attempt $attempt/$max_attempts - waiting for application to start..."
        sleep 2
        ((attempt++))
    done
    
    echo "L Health check failed after $max_attempts attempts"
    return 1
}

# Stop existing container
echo "=Ñ Stopping existing container..."
docker stop $CONTAINER_NAME || true
docker rm $CONTAINER_NAME || true

# Start new container
echo "¶ Starting new container..."
docker run -d \
    --name $CONTAINER_NAME \
    --restart unless-stopped \
    -p 3000:3000 \
    -e NODE_ENV=production \
    -e DATABASE_URL="$DATABASE_URL" \
    -e NEXTAUTH_SECRET="$NEXTAUTH_SECRET" \
    -e NEXTAUTH_URL="$NEXTAUTH_URL" \
    -e SENTRY_DSN="${SENTRY_DSN:-}" \
    -e VERCEL_ANALYTICS_ID="${VERCEL_ANALYTICS_ID:-}" \
    $IMAGE_TAG

# Wait for container to start
sleep 10

# Perform health check
if health_check; then
    echo "<‰ Deployment to $ENVIRONMENT completed successfully!"
    
    # Show container status
    echo "=Ê Container status:"
    docker ps --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
    
    # Show recent logs
    echo "=Ý Recent logs:"
    docker logs --tail 20 $CONTAINER_NAME
else
    echo "L Deployment failed - health check unsuccessful"
    echo "=Ý Container logs:"
    docker logs --tail 50 $CONTAINER_NAME
    exit 1
fi

# Optional: Clean up old images
if [ "$CLEANUP_OLD_IMAGES" = "true" ]; then
    echo ">ù Cleaning up old Docker images..."
    docker image prune -f
fi

echo " Deployment completed!"