# Deployment Architecture

## Deployment Strategy

**Unified Application Deployment:**
- **Platform:** Single Docker container with embedded templates and static asset serving
- **Build Command:** `npm run build && cargo build --release`
- **Binary:** Single executable with embedded Askama templates
- **Static Assets:** Served directly by axum with aggressive caching headers
- **Deployment Method:** Docker image with health checks and graceful shutdown

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
        image: postgres:17
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      redis:
        image: redis:8.2
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Run tests
        run: |
          cargo test --workspace
          
      - name: Run frontend tests
        run: |
          npm ci
          npm run test
          
      - name: Run E2E tests
        run: |
          npm run test:e2e

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Docker image
        run: |
          docker build -t imkitchen:latest .
          
      - name: Deploy to production
        run: |
          # Deploy Docker image to production environment
          # Implementation depends on chosen hosting platform
```

## Environments

| Environment | Application URL | Purpose |
|-------------|----------------|---------|
| Development | http://localhost:3000 | Local development |
| Staging | https://staging.imkitchen.app | Pre-production testing |
| Production | https://imkitchen.app | Live environment |
