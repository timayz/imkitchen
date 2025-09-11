# 12. Unified Project Structure

## 12.1 Monorepo Organization

```
imkitchen/
├── packages/
│   ├── api/                    # tRPC API server
│   │   ├── src/
│   │   │   ├── routers/       # API route definitions
│   │   │   ├── services/      # Business logic
│   │   │   ├── lib/           # Utilities and helpers
│   │   │   └── types/         # Shared TypeScript types
│   │   ├── prisma/            # Database schema and migrations
│   │   └── package.json
│   │
│   ├── web/                   # Next.js frontend application
│   │   ├── src/
│   │   │   ├── app/           # Next.js 15 App Router
│   │   │   ├── components/    # React components
│   │   │   ├── hooks/         # Custom React hooks
│   │   │   ├── lib/           # Client utilities
│   │   │   ├── styles/        # Global styles and Tailwind
│   │   │   └── types/         # Frontend types
│   │   ├── public/            # Static assets
│   │   └── package.json
│   │
│   ├── shared/                # Shared utilities and types
│   │   ├── src/
│   │   │   ├── types/         # Shared TypeScript definitions
│   │   │   ├── utils/         # Common utilities
│   │   │   ├── constants/     # Application constants
│   │   │   └── schemas/       # Zod validation schemas
│   │   └── package.json
│   │
│   └── worker/                # Background job processing
│       ├── src/
│       │   ├── jobs/          # Job definitions
│       │   ├── services/      # Worker services
│       │   └── lib/           # Worker utilities
│       └── package.json
│
├── apps/
│   └── docs/                  # Documentation site (optional)
│
├── tools/
│   ├── eslint-config/         # Shared ESLint configuration
│   ├── typescript-config/     # Shared TypeScript configuration
│   └── tailwind-config/       # Shared Tailwind configuration
│
├── docs/                      # Architecture and documentation
├── docker/                    # Docker configurations
├── k8s/                       # Kubernetes manifests
├── .github/                   # GitHub Actions workflows
├── turbo.json                 # Turborepo configuration
├── package.json               # Root package.json
└── pnpm-workspace.yaml        # PNPM workspace configuration
```

## 12.2 Package Dependencies

```json
// Root package.json
{
  "name": "imkitchen",
  "private": true,
  "scripts": {
    "build": "turbo run build",
    "dev": "turbo run dev --parallel",
    "lint": "turbo run lint",
    "test": "turbo run test",
    "db:push": "cd packages/api && pnpm db:push",
    "db:migrate": "cd packages/api && pnpm db:migrate",
    "docker:build": "docker-compose build",
    "docker:up": "docker-compose up -d"
  },
  "devDependencies": {
    "turbo": "^1.10.0",
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0",
    "prettier": "^3.0.0"
  },
  "packageManager": "pnpm@8.6.0"
}

// packages/web/package.json
{
  "name": "@imkitchen/web",
  "dependencies": {
    "next": "^15.0.0",
    "react": "^18.0.0",
    "react-dom": "^18.0.0",
    "@trpc/client": "^10.45.0",
    "@trpc/next": "^10.45.0",
    "@tanstack/react-query": "^4.0.0",
    "tailwindcss": "^4.1.0",
    "@headlessui/react": "^2.1.0",
    "next-auth": "^4.24.0",
    "react-hook-form": "^7.0.0",
    "zod": "^3.22.0",
    "@imkitchen/shared": "workspace:*"
  }
}

// packages/api/package.json  
{
  "name": "@imkitchen/api",
  "dependencies": {
    "@trpc/server": "^10.45.0",
    "prisma": "^5.0.0",
    "@prisma/client": "^5.0.0",
    "redis": "^4.6.0",
    "zod": "^3.22.0",
    "next-auth": "^4.24.0",
    "@imkitchen/shared": "workspace:*"
  }
}
```

## 12.3 Development Workflow

**Local Development Setup:**
```bash
# Clone repository
git clone https://github.com/your-org/imkitchen.git
cd imkitchen

# Install dependencies
pnpm install

# Setup environment variables
cp .env.example .env.local

# Start database with Docker
docker-compose up -d postgres redis

# Run database migrations
pnpm db:migrate

# Start development servers
pnpm dev
```

**Development Scripts:**
```json
{
  "scripts": {
    "dev": "turbo run dev --parallel",
    "build": "turbo run build",
    "test": "turbo run test",
    "test:e2e": "turbo run test:e2e",
    "lint": "turbo run lint",
    "lint:fix": "turbo run lint:fix",
    "format": "prettier --write \"**/*.{ts,tsx,js,jsx,json,md}\"",
    "type-check": "turbo run type-check",
    "db:studio": "cd packages/api && pnpm db:studio",
    "db:seed": "cd packages/api && pnpm db:seed"
  }
}
```

## 12.4 Environment Configuration

**.env.example:**
```bash
# Database
DATABASE_URL="postgresql://postgres:password@localhost:5432/imkitchen"
REDIS_URL="redis://localhost:6379"

# Authentication
NEXTAUTH_URL="http://localhost:3000"
NEXTAUTH_SECRET="your-secret-key"

# OAuth Providers
GOOGLE_CLIENT_ID="your-google-client-id"
GOOGLE_CLIENT_SECRET="your-google-client-secret"

# External APIs
SPOONACULAR_API_KEY="your-spoonacular-key"
EDAMAM_APP_ID="your-edamam-app-id"
EDAMAM_APP_KEY="your-edamam-key"

# Notifications
VAPID_PUBLIC_KEY="your-vapid-public-key"
VAPID_PRIVATE_KEY="your-vapid-private-key"
RESEND_API_KEY="your-resend-api-key"

# Monitoring
SENTRY_DSN="your-sentry-dsn"
DATADOG_API_KEY="your-datadog-key"

# Development
NODE_ENV="development"
LOG_LEVEL="debug"
```

## 12.5 Build and Deployment Configuration

**Turbo Configuration (turbo.json):**
```json
{
  "$schema": "https://turbo.build/schema.json",
  "globalDependencies": ["**/.env.*local"],
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],
      "outputs": [".next/**", "!.next/cache/**", "dist/**"]
    },
    "dev": {
      "cache": false,
      "persistent": true
    },
    "lint": {
      "outputs": []
    },
    "test": {
      "dependsOn": ["^build"],
      "outputs": ["coverage/**"]
    },
    "type-check": {
      "dependsOn": ["^build"],
      "outputs": []
    },
    "db:migrate": {
      "cache": false
    },
    "db:push": {
      "cache": false
    }
  }
}
```

**Docker Compose for Development:**
```yaml
version: '3.8'
services:
  postgres:
    image: postgres:17.6
    environment:
      POSTGRES_DB: imkitchen
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  web:
    build:
      context: .
      dockerfile: docker/web.Dockerfile
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/imkitchen
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    volumes:
      - .:/app
      - /app/node_modules

volumes:
  postgres_data:
  redis_data:
```
