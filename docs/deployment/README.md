# Deployment Guide

Comprehensive deployment guide for IMKitchen, covering Docker containerization, CI/CD pipelines, and production deployment procedures.

## Table of Contents

- [Overview](#overview)
- [Docker Deployment](#docker-deployment)
- [CI/CD Pipeline](#cicd-pipeline)
- [Environment Setup](#environment-setup)
- [Production Deployment](#production-deployment)
- [Monitoring and Observability](#monitoring-and-observability)
- [Backup and Recovery](#backup-and-recovery)
- [Troubleshooting](#troubleshooting)

## Overview

### Deployment Strategy

IMKitchen uses a **containerized deployment strategy** with the following characteristics:

- **Docker Containers**: Application packaged as lightweight, portable containers
- **Multi-Stage Builds**: Optimized container images with minimal attack surface
- **Environment Parity**: Consistent deployment across development, staging, and production
- **Zero-Downtime Deployments**: Rolling updates with health checks
- **Infrastructure as Code**: Reproducible deployments with version control

### Deployment Environments

| Environment | Purpose | URL Pattern | Database |
|-------------|---------|-------------|----------|
| **Development** | Local development | `localhost:3000` | SQLite file |
| **Staging** | Pre-production testing | `staging.imkitchen.com` | SQLite/PostgreSQL |
| **Production** | Live application | `imkitchen.com` | PostgreSQL/SQLite |

## Docker Deployment

### Dockerfile

```dockerfile
# Multi-stage build for optimized production image
FROM rust:1.90-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first (for better layer caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/

# Create dummy source files to build dependencies
RUN mkdir -p src crates/imkitchen-shared/src crates/imkitchen-user/src \
    crates/imkitchen-recipe/src crates/imkitchen-meal-planning/src \
    crates/imkitchen-shopping/src crates/imkitchen-notification/src \
    crates/imkitchen-web/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy" > crates/imkitchen-shared/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-user/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-recipe/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-meal-planning/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-shopping/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-notification/src/lib.rs && \
    echo "// dummy" > crates/imkitchen-web/src/lib.rs

# Build dependencies (will be cached if Cargo.toml doesn't change)
RUN cargo build --release --workspace
RUN rm -rf src crates/*/src

# Copy actual source code
COPY src ./src
COPY crates ./crates

# Build application
RUN cargo build --release --workspace

# Install SQLx CLI for migrations
RUN cargo install sqlx-cli --no-default-features --features sqlite

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1001 imkitchen

# Copy binary and SQLx CLI
COPY --from=builder /app/target/release/imkitchen /usr/local/bin/
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

# Copy migrations
COPY migrations /app/migrations

# Copy static assets
COPY crates/imkitchen-web/static /app/static
COPY crates/imkitchen-web/templates /app/templates

# Create data directory
RUN mkdir -p /app/data && chown imkitchen:imkitchen /app/data

# Switch to non-root user
USER imkitchen
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Expose port
EXPOSE 3000

# Default command
CMD ["imkitchen", "web", "start", "--host", "0.0.0.0", "--port", "3000"]
```

### Docker Compose

#### Development Compose

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  imkitchen-dev:
    build:
      context: .
      dockerfile: Dockerfile.dev
    ports:
      - "3000:3000"
    volumes:
      - .:/app
      - target:/app/target
      - ~/.cargo/registry:/usr/local/cargo/registry
    environment:
      - DATABASE_URL=sqlite:/app/data/imkitchen.db
      - RUST_LOG=debug
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=3000
      - SESSION_SECRET=dev-secret-key-32-characters-long
    working_dir: /app
    command: cargo watch -x "run -- web start --host 0.0.0.0 --port 3000"

volumes:
  target:
```

#### Production Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  imkitchen:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - imkitchen-data:/app/data
    environment:
      - DATABASE_URL=sqlite:/app/data/imkitchen.db
      - RUST_LOG=info
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=3000
      - SESSION_SECRET=${SESSION_SECRET}
      - SMTP_HOST=${SMTP_HOST}
      - SMTP_PORT=${SMTP_PORT}
      - SMTP_USERNAME=${SMTP_USERNAME}
      - SMTP_PASSWORD=${SMTP_PASSWORD}
      - SMTP_FROM_EMAIL=${SMTP_FROM_EMAIL}
      - SMTP_FROM_NAME=${SMTP_FROM_NAME}
    restart: unless-stopped
    depends_on:
      - nginx
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
      - imkitchen-static:/var/www/static:ro
    restart: unless-stopped
    depends_on:
      - imkitchen

volumes:
  imkitchen-data:
  imkitchen-static:
```

### Build Commands

```bash
# Development build
docker build -f Dockerfile.dev -t imkitchen:dev .

# Production build
docker build -t imkitchen:latest .

# Multi-platform build (for ARM and x86)
docker buildx build --platform linux/amd64,linux/arm64 -t imkitchen:latest .

# Build with specific tag
docker build -t imkitchen:v1.0.0 .
```

### Running Containers

```bash
# Development with compose
docker-compose -f docker-compose.dev.yml up

# Production with compose
docker-compose up -d

# Single container
docker run -d \
  --name imkitchen \
  -p 3000:3000 \
  -v imkitchen-data:/app/data \
  -e DATABASE_URL=sqlite:/app/data/imkitchen.db \
  -e SESSION_SECRET=your-production-secret \
  imkitchen:latest

# With environment file
docker run -d \
  --name imkitchen \
  -p 3000:3000 \
  -v imkitchen-data:/app/data \
  --env-file .env.production \
  imkitchen:latest
```

## CI/CD Pipeline

### GitHub Actions Workflow

```yaml
# .github/workflows/deploy.yml
name: Build and Deploy

on:
  push:
    branches: [main]
    tags: ['v*']
  pull_request:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true
          components: rustfmt, clippy
          
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Check formatting
        run: cargo fmt --all -- --check
        
      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings
        
      - name: Run tests
        run: cargo test --workspace
        
      - name: Run integration tests
        run: cargo test --workspace --test integration

  build:
    needs: test
    runs-on: ubuntu-latest
    outputs:
      image: ${{ steps.image.outputs.image }}
      digest: ${{ steps.build.outputs.digest }}
      
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
        
      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
          
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha
            
      - name: Build and push Docker image
        id: build
        uses: docker/build-push-action@v5
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          
      - name: Output image
        id: image
        run: |
          echo "image=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ steps.meta.outputs.version }}" >> $GITHUB_OUTPUT

  deploy-staging:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    environment: staging
    
    steps:
      - name: Deploy to staging
        run: |
          echo "Deploying ${{ needs.build.outputs.image }} to staging"
          # Add staging deployment commands here
          
  deploy-production:
    needs: build
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    environment: production
    
    steps:
      - name: Deploy to production
        run: |
          echo "Deploying ${{ needs.build.outputs.image }} to production"
          # Add production deployment commands here
```

### Deployment Scripts

#### Staging Deployment

```bash
#!/bin/bash
# scripts/deploy-staging.sh

set -e

echo "Starting staging deployment..."

# Configuration
IMAGE_TAG=${1:-latest}
STAGING_HOST="staging.imkitchen.com"
STAGING_USER="deploy"

# Pre-deployment checks
echo "Running pre-deployment checks..."
curl -f "https://${STAGING_HOST}/health" || echo "Service currently down"

# Pull latest image
echo "Pulling image: imkitchen:${IMAGE_TAG}"
ssh ${STAGING_USER}@${STAGING_HOST} "docker pull ghcr.io/yourorg/imkitchen:${IMAGE_TAG}"

# Backup database
echo "Creating database backup..."
ssh ${STAGING_USER}@${STAGING_HOST} "
  docker exec imkitchen-staging cp /app/data/imkitchen.db /app/data/imkitchen.db.backup.\$(date +%Y%m%d_%H%M%S)
"

# Run database migrations
echo "Running database migrations..."
ssh ${STAGING_USER}@${STAGING_HOST} "
  docker run --rm \
    -v imkitchen-staging-data:/app/data \
    ghcr.io/yourorg/imkitchen:${IMAGE_TAG} \
    sqlx migrate run --database-url sqlite:/app/data/imkitchen.db
"

# Deploy new version
echo "Deploying new version..."
ssh ${STAGING_USER}@${STAGING_HOST} "
  cd /opt/imkitchen-staging
  export IMAGE_TAG=${IMAGE_TAG}
  docker-compose pull
  docker-compose up -d
"

# Health check
echo "Performing health check..."
for i in {1..30}; do
  if curl -f "https://${STAGING_HOST}/health"; then
    echo "Deployment successful!"
    exit 0
  fi
  echo "Waiting for service to be ready... ($i/30)"
  sleep 10
done

echo "Health check failed!"
exit 1
```

#### Production Deployment

```bash
#!/bin/bash
# scripts/deploy-production.sh

set -e

echo "Starting production deployment..."

# Configuration
IMAGE_TAG=${1:-latest}
PRODUCTION_HOST="imkitchen.com"
PRODUCTION_USER="deploy"

# Confirmation
read -p "Deploy ${IMAGE_TAG} to production? (yes/no): " confirm
if [[ $confirm != "yes" ]]; then
  echo "Deployment cancelled"
  exit 1
fi

# Pre-deployment backup
echo "Creating production backup..."
ssh ${PRODUCTION_USER}@${PRODUCTION_HOST} "
  # Backup database
  docker exec imkitchen-prod cp /app/data/imkitchen.db /app/data/imkitchen.db.backup.\$(date +%Y%m%d_%H%M%S)
  
  # Backup static files
  tar czf /opt/backups/static-\$(date +%Y%m%d_%H%M%S).tar.gz /opt/imkitchen/static/
"

# Pull and validate image
echo "Pulling production image..."
ssh ${PRODUCTION_USER}@${PRODUCTION_HOST} "
  docker pull ghcr.io/yourorg/imkitchen:${IMAGE_TAG}
  docker image inspect ghcr.io/yourorg/imkitchen:${IMAGE_TAG}
"

# Run migrations
echo "Running database migrations..."
ssh ${PRODUCTION_USER}@${PRODUCTION_HOST} "
  docker run --rm \
    -v imkitchen-prod-data:/app/data \
    ghcr.io/yourorg/imkitchen:${IMAGE_TAG} \
    sqlx migrate run --database-url sqlite:/app/data/imkitchen.db
"

# Rolling deployment
echo "Performing rolling deployment..."
ssh ${PRODUCTION_USER}@${PRODUCTION_HOST} "
  cd /opt/imkitchen-production
  export IMAGE_TAG=${IMAGE_TAG}
  
  # Update with zero-downtime deployment
  docker-compose pull
  docker-compose up -d --no-deps imkitchen
  
  # Wait for health check
  sleep 30
  
  # Verify deployment
  curl -f http://localhost:3000/health
"

echo "Production deployment completed successfully!"
```

## Environment Setup

### Environment Configuration

#### Development (.env.development)

```bash
# Application
DATABASE_URL=sqlite:imkitchen.db
RUST_LOG=debug
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Security
SESSION_SECRET=dev-secret-key-32-characters-long-replace-in-production
PASSWORD_SALT_ROUNDS=10

# SMTP (disabled for development)
# SMTP_HOST=localhost
# SMTP_PORT=1025

# Features
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=false
ENABLE_COMMUNITY_FEATURES=true

# Development
RUST_BACKTRACE=1
CARGO_WATCH_IGNORE="*.db,*.log"
```

#### Staging (.env.staging)

```bash
# Application
DATABASE_URL=sqlite:/app/data/imkitchen.db
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
SESSION_SECRET=${STAGING_SESSION_SECRET}
PASSWORD_SALT_ROUNDS=12

# SMTP
SMTP_HOST=smtp.mailtrap.io
SMTP_PORT=2525
SMTP_USERNAME=${STAGING_SMTP_USERNAME}
SMTP_PASSWORD=${STAGING_SMTP_PASSWORD}
SMTP_FROM_EMAIL=noreply@staging.imkitchen.com
SMTP_FROM_NAME=IMKitchen Staging
SMTP_SECURITY=starttls

# Features
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
ENABLE_COMMUNITY_FEATURES=true
```

#### Production (.env.production)

```bash
# Application
DATABASE_URL=sqlite:/app/data/imkitchen.db
RUST_LOG=warn
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Security
SESSION_SECRET=${PRODUCTION_SESSION_SECRET}
PASSWORD_SALT_ROUNDS=12

# SMTP
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=${PRODUCTION_SENDGRID_API_KEY}
SMTP_FROM_EMAIL=noreply@imkitchen.com
SMTP_FROM_NAME=IMKitchen
SMTP_SECURITY=starttls
SMTP_TIMEOUT=30

# Features
ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
ENABLE_COMMUNITY_FEATURES=true
```

### Secret Management

#### Using Docker Secrets

```yaml
# docker-compose.yml with secrets
version: '3.8'

services:
  imkitchen:
    image: ghcr.io/yourorg/imkitchen:latest
    environment:
      - DATABASE_URL=sqlite:/app/data/imkitchen.db
      - SESSION_SECRET_FILE=/run/secrets/session_secret
      - SMTP_PASSWORD_FILE=/run/secrets/smtp_password
    secrets:
      - session_secret
      - smtp_password

secrets:
  session_secret:
    external: true
  smtp_password:
    external: true
```

#### Using Environment Variable Files

```bash
# Create secrets
echo "your-super-secure-session-secret-32-chars" | docker secret create session_secret -
echo "your-smtp-password" | docker secret create smtp_password -

# List secrets
docker secret ls
```

## Production Deployment

### Server Requirements

#### Minimum Requirements

- **CPU**: 1 vCPU (2+ recommended)
- **Memory**: 1GB RAM (2GB+ recommended)
- **Storage**: 10GB SSD (20GB+ recommended)
- **Network**: 1Gbps connection
- **OS**: Ubuntu 22.04 LTS or similar

#### Recommended Setup

- **CPU**: 2+ vCPUs
- **Memory**: 4GB+ RAM
- **Storage**: 50GB+ SSD with backup
- **Network**: Load balancer with SSL termination
- **OS**: Latest LTS Linux distribution

### SSL/TLS Configuration

#### Let's Encrypt with Nginx

```nginx
# /etc/nginx/sites-available/imkitchen
server {
    listen 80;
    server_name imkitchen.com www.imkitchen.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name imkitchen.com www.imkitchen.com;

    # SSL configuration
    ssl_certificate /etc/letsencrypt/live/imkitchen.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/imkitchen.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512:ECDHE-RSA-AES256-GCM-SHA384;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header Strict-Transport-Security "max-age=63072000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header Referrer-Policy "strict-origin-when-cross-origin";

    # Static files
    location /static/ {
        alias /var/www/imkitchen/static/;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # Application
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Health check
        proxy_set_header Connection "";
        proxy_http_version 1.1;
        proxy_connect_timeout 30s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;
    }
}
```

#### SSL Certificate Setup

```bash
# Install Certbot
sudo apt update
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d imkitchen.com -d www.imkitchen.com

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

### Load Balancing and High Availability

#### Docker Swarm Setup

```yaml
# docker-stack.yml
version: '3.8'

services:
  imkitchen:
    image: ghcr.io/yourorg/imkitchen:latest
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
        order: start-first
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
      placement:
        constraints:
          - node.role == worker
    networks:
      - imkitchen-network
    volumes:
      - imkitchen-data:/app/data
    environment:
      - DATABASE_URL=sqlite:/app/data/imkitchen.db
      - SESSION_SECRET_FILE=/run/secrets/session_secret
    secrets:
      - session_secret

  nginx:
    image: nginx:alpine
    deploy:
      replicas: 2
      placement:
        constraints:
          - node.role == manager
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/nginx/ssl:ro
    networks:
      - imkitchen-network

networks:
  imkitchen-network:
    driver: overlay

volumes:
  imkitchen-data:

secrets:
  session_secret:
    external: true
```

#### Deploy Stack

```bash
# Initialize swarm
docker swarm init

# Deploy stack
docker stack deploy -c docker-stack.yml imkitchen

# Scale services
docker service scale imkitchen_imkitchen=5

# Monitor services
docker service ls
docker service logs imkitchen_imkitchen
```

## Monitoring and Observability

### Health Checks

#### Application Health Check

```rust
// src/handlers/health.rs
use axum::{extract::State, response::Json};
use serde_json::{json, Value};

pub async fn health_check_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let mut health = json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    // Database connectivity check
    match sqlx::query("SELECT 1").fetch_one(&app_state.db_pool).await {
        Ok(_) => {
            health["database"] = json!("connected");
        }
        Err(e) => {
            health["status"] = json!("unhealthy");
            health["database"] = json!(format!("error: {}", e));
        }
    }

    // SMTP connectivity check (if configured)
    if let Some(smtp_config) = &app_state.smtp_config {
        match test_smtp_connection(smtp_config).await {
            Ok(_) => health["smtp"] = json!("connected"),
            Err(e) => {
                health["smtp"] = json!(format!("error: {}", e));
                // Don't mark as unhealthy for SMTP issues
            }
        }
    }

    Ok(Json(health))
}
```

#### Docker Health Check

```dockerfile
# In Dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1
```

### Logging Configuration

#### Structured Logging

```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "imkitchen=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

#### Log Aggregation

```yaml
# docker-compose.yml with logging
version: '3.8'

services:
  imkitchen:
    # ... other config
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
        labels: "service=imkitchen"

  # Optional: Log aggregation with Loki
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki-config.yml:/etc/loki/local-config.yaml

  promtail:
    image: grafana/promtail:latest
    volumes:
      - /var/log:/var/log:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
      - ./promtail-config.yml:/etc/promtail/config.yml
```

### Metrics Collection

#### Prometheus Metrics

```rust
// Cargo.toml
[dependencies]
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

// src/metrics.rs
use metrics::{counter, histogram, increment_counter, register_counter, register_histogram};

pub fn init_metrics() {
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install Prometheus recorder");

    // Register metrics
    register_counter!("http_requests_total", "Total HTTP requests");
    register_histogram!("http_request_duration_seconds", "HTTP request duration");
}

// In handlers
pub async fn recipe_handler(/* ... */) -> Result<Html<String>, AppError> {
    increment_counter!("http_requests_total", "endpoint" => "recipes");
    
    let start = std::time::Instant::now();
    let result = handle_recipe_request().await;
    let duration = start.elapsed();
    
    histogram!("http_request_duration_seconds", duration.as_secs_f64(), "endpoint" => "recipes");
    
    result
}
```

For more deployment information:
- [Environment Setup Guide](environment.md)
- [Monitoring Setup](monitoring.md)
- [Security Configuration](security.md)
- [Backup Procedures](backup.md)