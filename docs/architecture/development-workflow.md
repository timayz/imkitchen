# Development Workflow

## Local Development Setup

### Prerequisites

```bash
# Install Node.js 18+ and pnpm
curl -fsSL https://fnm.vercel.app/install | bash
fnm install 18
fnm use 18
npm install -g pnpm

# Install Docker and Docker Compose
# Follow official Docker installation for your OS

# Install PostgreSQL and Redis (via Docker)
docker --version
docker-compose --version
```

### Initial Setup

```bash
# 1. REPOSITORY & DEPENDENCIES (Epic 1, Story 1.1)
git clone <repository-url>
cd imkitchen
pnpm install

# 2. ENVIRONMENT CONFIGURATION (Epic 1, Story 1.1 - BEFORE any services)
cp .env.example .env.local
# CRITICAL: Edit .env.local with your configuration including:
# - Database connection strings
# - External API keys (Spoonacular, OpenFoodFacts)
# - Authentication secrets
# - Email service configuration

# 3. INFRASTRUCTURE SERVICES (Epic 1, Story 1.2 - BEFORE database operations)
docker-compose up -d postgres redis

# 4. DATABASE SETUP (Epic 1, Story 1.2 - AFTER services running)
pnpm db:migrate
pnpm db:generate
pnpm db:seed

# 5. DEVELOPMENT SERVER (Epic 1, Story 1.6 - FINAL step)
pnpm dev
```

**⚠️ CRITICAL TIMING:** Environment variables must be configured before starting any services. Database must be running before migrations. All setup must complete before development server start.

### Development Commands

```bash
# Start all services
pnpm dev

# Start frontend only (assumes API running elsewhere)
pnpm dev:frontend

# Start backend only (API routes and services)
pnpm dev:backend

# Run tests
pnpm test              # Unit tests
pnpm test:integration  # Integration tests
pnpm test:e2e          # End-to-end tests
pnpm test:watch        # Watch mode

# Database operations
pnpm db:migrate        # Run migrations
pnpm db:reset          # Reset database
pnpm db:seed           # Seed with test data
pnpm db:studio         # Open Prisma Studio

# Code quality
pnpm lint              # ESLint
pnpm type-check        # TypeScript check
pnpm format            # Prettier
```

## Environment Configuration

### Required Environment Variables

```bash
# Frontend (.env.local)
NEXT_PUBLIC_APP_URL=http://localhost:3000
NEXT_PUBLIC_API_URL=http://localhost:3000/api
NEXT_PUBLIC_VOICE_API_KEY=your_voice_api_key
NEXT_PUBLIC_SENTRY_DSN=your_sentry_dsn

# Backend (.env)
DATABASE_URL=postgresql://user:password@localhost:5432/imkitchen
REDIS_URL=redis://localhost:6379
NEXTAUTH_SECRET=your_nextauth_secret
NEXTAUTH_URL=http://localhost:3000

# External APIs
SPOONACULAR_API_KEY=your_spoonacular_key
OPENAI_API_KEY=your_openai_key_for_voice
SENDGRID_API_KEY=your_sendgrid_key

# Storage
S3_BUCKET_NAME=imkitchen-uploads
S3_ACCESS_KEY_ID=your_s3_access_key
S3_SECRET_ACCESS_KEY=your_s3_secret_key
S3_REGION=us-east-1

# Shared
NODE_ENV=development
LOG_LEVEL=debug
```
