# Deployment Architecture

## Deployment Strategy

**Frontend Deployment:**
- **Platform:** Integrated with backend binary
- **Build Command:** `cargo build --release`
- **Output Directory:** Binary with embedded templates
- **CDN/Edge:** Static assets served via CDN, HTML via server

**Backend Deployment:**
- **Platform:** Docker containers with Kubernetes orchestration
- **Build Command:** `cargo build --release --bin imkitchen`
- **Deployment Method:** Blue-green deployment with health checks

## CI/CD Pipeline
```yaml
name: IMKitchen CI/CD
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Run tests
        run: |
          cargo test --workspace
          cargo clippy --workspace -- -D warnings
          cargo fmt --all -- --check
          
      - name: Check TDD coverage
        run: cargo tarpaulin --workspace --out xml
        
  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: docker/build-push-action@v4
        with:
          push: true
          tags: imkitchen:${{ github.sha }}
          
  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/imkitchen \
            imkitchen=imkitchen:${{ github.sha }}
          kubectl rollout status deployment/imkitchen
```

## Environments

| Environment | Frontend URL | Backend URL | Purpose |
|-------------|--------------|-------------|---------|
| Development | http://localhost:3000 | http://localhost:3000 | Local development |
| Staging | https://staging.imkitchen.com | https://staging.imkitchen.com | Pre-production testing |
| Production | https://imkitchen.com | https://imkitchen.com | Live environment |
