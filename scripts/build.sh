#!/bin/bash

# Build script for imkitchen application
set -e

echo "<× Building imkitchen application..."

# Environment check
if [ -z "$NODE_ENV" ]; then
    export NODE_ENV=production
fi

echo "=Ë Environment: $NODE_ENV"

# Clean previous builds
echo ">ù Cleaning previous builds..."
rm -rf .next
rm -rf dist

# Install dependencies
echo "=æ Installing dependencies..."
npm ci --only=production

# Generate Prisma client
echo "= Generating Prisma client..."
npx prisma generate

# Run database migrations (if applicable)
if [ "$SKIP_DB_MIGRATIONS" != "true" ]; then
    echo "=Ã Running database migrations..."
    npx prisma migrate deploy
fi

# Build the application
echo "¡ Building Next.js application..."
npm run build

# Verify build output
echo " Verifying build output..."
if [ -d ".next" ]; then
    echo " Build completed successfully!"
    echo "=Ê Build statistics:"
    du -sh .next
    ls -la .next/
else
    echo "L Build failed - .next directory not found"
    exit 1
fi

# Optional: Build Docker image
if [ "$BUILD_DOCKER" = "true" ]; then
    echo "=3 Building Docker image..."
    docker build -f docker/Dockerfile -t imkitchen:latest .
    echo " Docker image built successfully!"
fi

echo "<‰ Build completed successfully!"