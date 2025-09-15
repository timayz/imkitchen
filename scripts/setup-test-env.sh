#!/bin/bash

# Setup Test Environment Script
# This script sets up environment variables for testing in CI/CD environments

echo "Setting up test environment variables..."

# Export required environment variables for tests
export DATABASE_URL="postgresql://test:test@localhost:5432/imkitchen_test"
export NEXTAUTH_SECRET="test-secret-key-for-testing-minimum-32-characters-long-secure"
export NEXTAUTH_URL="http://localhost:3000"
export NEXT_PUBLIC_APP_URL="http://localhost:3000"
export NEXT_PUBLIC_API_URL="http://localhost:3000/api"
export NODE_ENV="development"
export LOG_LEVEL="error"
export NEXT_TELEMETRY_DISABLED="1"

echo "Test environment variables set successfully!"

# Run the command passed as arguments
exec "$@"