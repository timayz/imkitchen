#!/bin/bash
# Database reset script for imkitchen

set -e

DATABASE_URL="${DATABASE_URL:-sqlite:imkitchen.db}"

echo "Resetting database: $DATABASE_URL"

# Remove existing database file if it exists
if [ -f "imkitchen.db" ]; then
    echo "Removing existing database file..."
    rm -f imkitchen.db
fi

# Recreate database and run migrations
echo "Creating fresh database..."
/home/snapiz/.cargo/bin/sqlx database create

echo "Running migrations..."
/home/snapiz/.cargo/bin/sqlx migrate run

echo "Database reset complete!"