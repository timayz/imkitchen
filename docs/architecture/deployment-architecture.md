# Deployment Architecture

## Deployment Strategy

**Frontend Deployment:**
- **Platform:** Docker containers with platform-agnostic deployment
- **Build Command:** `pnpm build`
- **Output Directory:** `.next/`
- **CDN/Edge:** CloudFlare for global content delivery and edge caching

**Backend Deployment:**
- **Platform:** Same Docker container (Next.js full-stack)
- **Build Command:** `pnpm build`
- **Deployment Method:** Blue-green deployment with health checks

## CI/CD Pipeline

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: imkitchen_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
        with:
          version: 8
      - uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: pnpm install --frozen-lockfile
      
      - name: Type check
        run: pnpm type-check
      
      - name: Lint
        run: pnpm lint
      
      - name: Run unit tests
        run: pnpm test
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test
          REDIS_URL: redis://localhost:6379
      
      - name: Run integration tests
        run: pnpm test:integration
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test
          REDIS_URL: redis://localhost:6379
      
      - name: Build application
        run: pnpm build
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/imkitchen_test

  deploy-staging:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/develop'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Docker image
        run: docker build -t imkitchen:staging .
      
      - name: Deploy to staging
        run: |
          # Platform-specific deployment commands
          # Could be AWS ECS, GCP Cloud Run, Azure Container Instances, etc.
          echo "Deploying to staging environment"

  deploy-production:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Docker image
        run: docker build -t imkitchen:production .
      
      - name: Deploy to production
        run: |
          # Production deployment with blue-green strategy
          echo "Deploying to production environment"
```

## Environments

| Environment | Frontend URL | Backend URL | Purpose |
|-------------|--------------|-------------|---------|
| Development | http://localhost:3000 | http://localhost:3000/api | Local development |
| Staging | https://staging.imkitchen.com | https://staging.imkitchen.com/api | Pre-production testing |
| Production | https://app.imkitchen.com | https://app.imkitchen.com/api | Live environment |
