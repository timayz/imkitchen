# ImKitchen Deployment Guide

This guide covers deployment strategies and procedures for ImKitchen across different environments.

## Deployment Overview

ImKitchen supports multiple deployment strategies:
- **Docker containers** (recommended)
- **Binary deployment** on Linux/macOS
- **Cloud platforms** (AWS, GCP, Azure)
- **Container orchestration** (Kubernetes)

## Prerequisites

### System Requirements

- **CPU**: 2+ cores recommended
- **RAM**: 2GB minimum, 4GB recommended
- **Storage**: 20GB minimum
- **OS**: Linux (Ubuntu 20.04+), macOS, or Windows with WSL2

### Required Services

- **PostgreSQL 17+**: Primary database
- **Redis 8.2+**: Session storage and caching
- **Reverse Proxy**: nginx, Apache, or cloud load balancer (production)

## Docker Deployment (Recommended)

### Quick Start

```bash
# Production deployment
docker-compose -f docker-compose.prod.yml up -d

# Check service status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f app
```

### Environment Setup

1. **Create production environment file**
   ```bash
   cp .env.example .env.prod
   # Edit with production values
   ```

2. **Set required environment variables**
   ```bash
   export POSTGRES_PASSWORD=secure_production_password
   export JWT_SECRET=$(openssl rand -base64 48)
   export REDIS_PASSWORD=$(openssl rand -base64 24)
   ```

3. **Deploy with environment**
   ```bash
   docker-compose -f docker-compose.prod.yml --env-file .env.prod up -d
   ```

### SSL/TLS Configuration

For HTTPS support, configure SSL certificates:

```bash
# Create SSL certificate directory
mkdir -p infrastructure/nginx/ssl

# Add your certificates (replace with your actual certificates)
cp your_certificate.crt infrastructure/nginx/ssl/
cp your_private_key.key infrastructure/nginx/ssl/

# Update nginx configuration for SSL
# Edit infrastructure/nginx/nginx.conf
```

## Binary Deployment

### Build Process

```bash
# Install build dependencies
sudo apt-get install pkg-config libssl-dev postgresql-client

# Build optimized binary
cargo build --release

# Copy binary and assets
mkdir -p /opt/imkitchen
cp target/release/imkitchen /opt/imkitchen/
cp -r templates /opt/imkitchen/
cp -r static /opt/imkitchen/
cp -r migrations /opt/imkitchen/
```

### Systemd Service

Create `/etc/systemd/system/imkitchen.service`:

```ini
[Unit]
Description=ImKitchen Recipe Management Platform
After=postgresql.service redis.service
Wants=postgresql.service redis.service

[Service]
Type=simple
User=imkitchen
Group=imkitchen
WorkingDirectory=/opt/imkitchen
ExecStart=/opt/imkitchen/imkitchen
Restart=always
RestartSec=10

# Environment variables
Environment=DATABASE_URL=postgresql://imkitchen:password@localhost:5432/imkitchen
Environment=REDIS_URL=redis://localhost:6379
Environment=JWT_SECRET=your_jwt_secret_here
Environment=ENVIRONMENT=production
Environment=SERVER_HOST=127.0.0.1
Environment=SERVER_PORT=3000

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/imkitchen/uploads /var/log/imkitchen

[Install]
WantedBy=multi-user.target
```

Start the service:

```bash
# Create user
sudo useradd -r -s /bin/false imkitchen
sudo chown -R imkitchen:imkitchen /opt/imkitchen

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable imkitchen
sudo systemctl start imkitchen

# Check status
sudo systemctl status imkitchen
```

## Cloud Platform Deployment

### AWS ECS

1. **Build and push Docker image**
   ```bash
   # Build image
   docker build -t imkitchen .
   
   # Tag for ECR
   docker tag imkitchen:latest 123456789.dkr.ecr.us-west-2.amazonaws.com/imkitchen:latest
   
   # Push to ECR
   docker push 123456789.dkr.ecr.us-west-2.amazonaws.com/imkitchen:latest
   ```

2. **Create ECS task definition**
   ```json
   {
     "family": "imkitchen",
     "networkMode": "awsvpc",
     "requiresCompatibilities": ["FARGATE"],
     "cpu": "512",
     "memory": "1024",
     "executionRoleArn": "arn:aws:iam::123456789:role/ecsTaskExecutionRole",
     "containerDefinitions": [
       {
         "name": "imkitchen",
         "image": "123456789.dkr.ecr.us-west-2.amazonaws.com/imkitchen:latest",
         "portMappings": [
           {
             "containerPort": 3000,
             "protocol": "tcp"
           }
         ],
         "environment": [
           {
             "name": "ENVIRONMENT",
             "value": "production"
           }
         ],
         "secrets": [
           {
             "name": "DATABASE_URL",
             "valueFrom": "arn:aws:secretsmanager:us-west-2:123456789:secret:imkitchen/database-url"
           },
           {
             "name": "JWT_SECRET",
             "valueFrom": "arn:aws:secretsmanager:us-west-2:123456789:secret:imkitchen/jwt-secret"
           }
         ],
         "logConfiguration": {
           "logDriver": "awslogs",
           "options": {
             "awslogs-group": "/ecs/imkitchen",
             "awslogs-region": "us-west-2",
             "awslogs-stream-prefix": "ecs"
           }
         }
       }
     ]
   }
   ```

### Google Cloud Run

```bash
# Build and deploy to Cloud Run
gcloud run deploy imkitchen \
  --image gcr.io/PROJECT_ID/imkitchen \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars ENVIRONMENT=production \
  --set-secrets DATABASE_URL=database-url:latest,JWT_SECRET=jwt-secret:latest
```

### Azure Container Instances

```bash
# Deploy to Azure
az container create \
  --resource-group imkitchen-rg \
  --name imkitchen \
  --image imkitchen.azurecr.io/imkitchen:latest \
  --cpu 2 \
  --memory 4 \
  --ports 3000 \
  --environment-variables ENVIRONMENT=production \
  --secure-environment-variables \
    DATABASE_URL=postgresql://... \
    JWT_SECRET=your_jwt_secret
```

## Kubernetes Deployment

### Namespace and ConfigMap

```yaml
# kubernetes/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: imkitchen

---
# kubernetes/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: imkitchen-config
  namespace: imkitchen
data:
  ENVIRONMENT: "production"
  SERVER_HOST: "0.0.0.0"
  SERVER_PORT: "3000"
  REDIS_URL: "redis://redis:6379"
```

### Secret Management

```yaml
# kubernetes/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: imkitchen-secrets
  namespace: imkitchen
type: Opaque
stringData:
  DATABASE_URL: "postgresql://user:password@postgres:5432/imkitchen"
  JWT_SECRET: "your_jwt_secret_here"
```

### Deployment

```yaml
# kubernetes/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: imkitchen
  namespace: imkitchen
spec:
  replicas: 3
  selector:
    matchLabels:
      app: imkitchen
  template:
    metadata:
      labels:
        app: imkitchen
    spec:
      containers:
      - name: imkitchen
        image: imkitchen:latest
        ports:
        - containerPort: 3000
        envFrom:
        - configMapRef:
            name: imkitchen-config
        - secretRef:
            name: imkitchen-secrets
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 10
```

### Service and Ingress

```yaml
# kubernetes/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: imkitchen-service
  namespace: imkitchen
spec:
  selector:
    app: imkitchen
  ports:
  - port: 80
    targetPort: 3000
  type: ClusterIP

---
# kubernetes/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: imkitchen-ingress
  namespace: imkitchen
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - imkitchen.example.com
    secretName: imkitchen-tls
  rules:
  - host: imkitchen.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: imkitchen-service
            port:
              number: 80
```

## Database Migration

### Running Migrations

```bash
# Docker environment
docker-compose exec app sqlx migrate run

# Kubernetes
kubectl exec -n imkitchen deployment/imkitchen -- sqlx migrate run

# Binary deployment
cd /opt/imkitchen && ./imkitchen migrate
```

### Backup and Restore

```bash
# Backup database
pg_dump $DATABASE_URL > imkitchen_backup_$(date +%Y%m%d_%H%M%S).sql

# Restore database
psql $DATABASE_URL < imkitchen_backup.sql
```

## Monitoring and Observability

### Health Checks

The application provides a `/health` endpoint for monitoring:

```bash
# Check application health
curl http://localhost:3000/health

# Expected response
{
  "status": "healthy",
  "timestamp": "2025-01-01T12:00:00Z",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "services": {
    "database": {"status": "healthy", "response_time_ms": 5},
    "redis": {"status": "healthy", "response_time_ms": 2}
  },
  "system": {
    "memory_usage_mb": 256,
    "cpu_load": 0.15,
    "disk_usage_percent": 45.0
  }
}
```

### Logging Configuration

Configure structured logging for production:

```bash
# Environment variables
RUST_LOG=warn,imkitchen=info
LOG_FILE=/var/log/imkitchen/app.log
```

### Metrics and Alerting

Consider integrating with:
- **Prometheus** for metrics collection
- **Grafana** for visualization
- **AlertManager** for alerting
- **Jaeger** for distributed tracing

## Scaling Considerations

### Horizontal Scaling

- Run multiple application instances behind a load balancer
- Ensure session storage is in Redis (not in-memory)
- Use external file storage for uploads (S3, GCS, etc.)

### Database Scaling

- Configure database connection pooling
- Consider read replicas for read-heavy workloads
- Monitor query performance and add indexes as needed

### Caching Strategy

- Utilize Redis for caching frequently accessed data
- Implement cache invalidation strategies
- Consider CDN for static assets

## Rollback Procedures

### Docker Deployment

```bash
# Rollback to previous version
docker-compose -f docker-compose.prod.yml down
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d
```

### Kubernetes Deployment

```bash
# Rollback deployment
kubectl rollout undo deployment/imkitchen -n imkitchen

# Check rollout status
kubectl rollout status deployment/imkitchen -n imkitchen
```

## Troubleshooting

### Common Deployment Issues

1. **Database connection failures**
   - Verify network connectivity
   - Check credentials and permissions
   - Ensure database is running and accessible

2. **Memory issues**
   - Monitor memory usage
   - Adjust container/VM memory limits
   - Check for memory leaks in logs

3. **SSL/TLS certificate issues**
   - Verify certificate validity
   - Check certificate chain
   - Ensure proper nginx configuration

### Log Analysis

```bash
# Docker logs
docker-compose logs -f app

# Kubernetes logs
kubectl logs -f deployment/imkitchen -n imkitchen

# System logs
journalctl -u imkitchen -f
```

## Security Checklist

- [ ] Use strong, unique passwords for all services
- [ ] Enable SSL/TLS encryption
- [ ] Configure firewalls and network security groups
- [ ] Use secrets management systems
- [ ] Regular security updates and patches
- [ ] Monitor for security vulnerabilities
- [ ] Implement proper backup and disaster recovery
- [ ] Use least-privilege access principles

## Support and Maintenance

### Regular Maintenance Tasks

- Monitor system resources and performance
- Review and rotate secrets/passwords
- Update dependencies and security patches
- Backup database and critical data
- Monitor logs for errors and issues

### Performance Optimization

- Monitor database query performance
- Optimize slow queries with proper indexing
- Configure appropriate connection pool sizes
- Monitor and tune garbage collection
- Use profiling tools to identify bottlenecks