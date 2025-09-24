#!/bin/bash
set -euo pipefail

# Database migration script for imkitchen
# Usage: ./migrate.sh [up|down] [environment]

ACTION=${1:-up}
ENVIRONMENT=${2:-development}

# Validate action
if [[ ! "$ACTION" =~ ^(up|down)$ ]]; then
    echo "Error: Action must be 'up' or 'down'"
    exit 1
fi

# Set environment-specific database URL
case "$ENVIRONMENT" in
    development)
        DATABASE_URL="sqlite:imkitchen.db"
        ;;
    staging)
        DATABASE_URL="sqlite:/data/staging/imkitchen.db"
        ;;
    production)
        DATABASE_URL="sqlite:/data/production/imkitchen.db"
        ;;
    *)
        echo "Error: Environment must be 'development', 'staging', or 'production'"
        exit 1
        ;;
esac

echo "🔄 Running database migration..."
echo "   Action: $ACTION"
echo "   Environment: $ENVIRONMENT"
echo "   Database URL: $DATABASE_URL"

# Ensure SQLx CLI is available
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features sqlite
fi

# Create backup for production
if [[ "$ENVIRONMENT" == "production" && "$ACTION" == "up" ]]; then
    DB_PATH="/data/production/imkitchen.db"
    if [[ -f "$DB_PATH" ]]; then
        echo "💾 Creating database backup..."
        BACKUP_PATH="/data/backups/migration-backup-$(date +%Y%m%d_%H%M%S).db"
        mkdir -p "/data/backups"
        cp "$DB_PATH" "$BACKUP_PATH"
        echo "✅ Backup created: $BACKUP_PATH"
    fi
fi

# Export DATABASE_URL for sqlx
export DATABASE_URL

# Run migration
if [[ "$ACTION" == "up" ]]; then
    echo "⬆️  Running migrations..."
    sqlx migrate run
    
    # Verify migration status
    echo "📊 Migration status:"
    sqlx migrate info
    
    echo "✅ Migrations completed successfully"
    
elif [[ "$ACTION" == "down" ]]; then
    echo "⬇️  Reverting last migration..."
    sqlx migrate revert
    
    # Verify migration status
    echo "📊 Migration status:"
    sqlx migrate info
    
    echo "✅ Migration reverted successfully"
fi

# Health check after migration (if app is running)
if [[ "$ENVIRONMENT" != "development" ]]; then
    HEALTH_PORT=$([ "$ENVIRONMENT" == "production" ] && echo "3000" || echo "3001")
    HEALTH_URL="http://localhost:$HEALTH_PORT/health"
    
    echo "🏥 Checking application health..."
    
    if curl -f -s "$HEALTH_URL" > /dev/null 2>&1; then
        echo "✅ Application is healthy after migration"
        
        # Display health info
        echo "📋 Health check response:"
        curl -s "$HEALTH_URL" | jq . || curl -s "$HEALTH_URL"
    else
        echo "⚠️  Application health check failed (this is normal if app is not running)"
    fi
fi

echo "🎉 Database migration completed successfully!"