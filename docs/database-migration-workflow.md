# Database Migration Workflow

## Overview

This document outlines the database migration workflow for the imkitchen project using Prisma ORM.

## Prerequisites

- PostgreSQL 15+ running (via Docker Compose)
- Node.js 18+ with npm/pnpm
- Prisma CLI installed (`npm install prisma`)

## Available Scripts

| Script                      | Command                 | Description                               |
| --------------------------- | ----------------------- | ----------------------------------------- |
| `npm run db:generate`       | `prisma generate`       | Generate Prisma Client from schema        |
| `npm run db:migrate`        | `prisma migrate dev`    | Create and apply migration in development |
| `npm run db:migrate:deploy` | `prisma migrate deploy` | Apply migrations in production            |
| `npm run db:seed`           | `prisma db seed`        | Seed database with initial data           |
| `npm run db:studio`         | `prisma studio`         | Open Prisma Studio GUI                    |
| `npm run db:reset`          | `prisma migrate reset`  | Reset database (development only)         |

## Development Workflow

### 1. Making Schema Changes

1. Edit `prisma/schema.prisma` to add/modify models
2. Run `npm run db:migrate` to create and apply migration
3. Provide a descriptive migration name when prompted
4. Commit both schema changes and migration files

### 2. Applying Existing Migrations

```bash
# Start database services
docker compose up -d postgres redis

# Apply all pending migrations
npm run db:migrate:deploy

# Generate Prisma Client
npm run db:generate

# (Optional) Seed database with test data
npm run db:seed
```

### 3. Database Reset (Development Only)

**⚠️ WARNING: This will delete all data!**

```bash
npm run db:reset
```

This command:

- Drops the database
- Creates a new database
- Applies all migrations
- Runs seed script (if configured)

## Production Deployment

### 1. Migration Deployment

```bash
# Set production environment
export NODE_ENV=production
export DATABASE_URL="postgresql://user:password@host:port/database"

# Apply migrations
npm run db:migrate:deploy

# Generate client
npm run db:generate
```

### 2. Migration Safety Checks

- **Always backup** the production database before migrations
- **Test migrations** on a staging environment first
- **Review migration SQL** in `prisma/migrations/` before deployment
- **Never run** `prisma migrate reset` in production

## Migration Files

### Location

- All migrations stored in `prisma/migrations/`
- Each migration in timestamped folder: `YYYYMMDDHHMMSS_description/`
- Contains `migration.sql` with exact SQL commands

### Example Structure

```
prisma/migrations/
├── 20250914231026_init/
│   └── migration.sql
├── 20250915120000_add_user_preferences/
│   └── migration.sql
└── migration_lock.toml
```

## Troubleshooting

### Common Issues

**Migration conflicts:**

```bash
# Reset migration state (development only)
npm run db:reset

# Or manually resolve conflicts
npx prisma migrate resolve --applied "migration_name"
```

**Schema drift:**

```bash
# Check if database is in sync
npx prisma migrate status

# Reset to current schema state
npx prisma db push
```

**Connection issues:**

```bash
# Check database connection
npx prisma db ping

# Verify environment variables
echo $DATABASE_URL
```

### Database Connection Verification

```bash
# Test connection health
node -e "
import { db, checkDatabaseHealth } from './src/lib/db.js';
checkDatabaseHealth().then(console.log);
"
```

## Best Practices

### Schema Design

- Use UUIDs for primary keys (security)
- Add proper indexes for query performance
- Use enums for controlled vocabularies
- Include timestamps (`createdAt`, `updatedAt`)

### Migration Naming

- Use descriptive names: `add_user_preferences`, `update_recipe_schema`
- Avoid generic names: `migration`, `update`, `changes`

### Data Safety

- **Never** edit existing migration files
- **Always** create new migrations for changes
- **Test** migrations on realistic data volumes
- **Document** breaking changes in migration comments

## Environment Configuration

### Development

```env
DATABASE_URL="postgresql://postgres:postgres@localhost:5432/imkitchen"
```

### Production

```env
DATABASE_URL="postgresql://user:password@prod-host:5432/imkitchen"
NODE_ENV="production"
```

## Team Collaboration

### Pull Request Workflow

1. Create migration in feature branch
2. Test migration locally
3. Include migration files in PR
4. Review migration SQL before merge
5. Apply to staging after merge
6. Deploy to production after staging validation

### Migration Conflicts

- If multiple developers create migrations, use `prisma migrate resolve`
- Coordinate schema changes through team communication
- Consider using feature flags for gradual schema rollouts

## Monitoring

### Migration Status

```bash
# Check applied migrations
npx prisma migrate status

# View migration history
npx prisma migrate diff
```

### Performance Monitoring

- Monitor migration execution time
- Check for blocking operations
- Validate index usage after schema changes
- Monitor query performance post-migration
