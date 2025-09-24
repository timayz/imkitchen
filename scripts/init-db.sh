#!/bin/bash
# Database initialization script for imkitchen

set -e

DATABASE_URL="${DATABASE_URL:-sqlite:imkitchen.db}"

echo "Initializing database: $DATABASE_URL"

# Create database if it doesn't exist
echo "Creating database..."
/home/snapiz/.cargo/bin/sqlx database create

# Run migrations
echo "Running migrations..."
/home/snapiz/.cargo/bin/sqlx migrate run

echo "Database initialization complete!"