# 14. Deployment Architecture

## Cloud-Agnostic Kubernetes Deployment

### Production Architecture
- **Load Balancer**: NGINX Ingress Controller
- **Container Orchestration**: Kubernetes 1.28+
- **Database**: Managed PostgreSQL (Cloud SQL/RDS equivalent)
- **Cache**: Redis Cluster
- **Object Storage**: MinIO or S3-compatible service
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack (Elasticsearch, Logstash, Kibana)

### Kubernetes Manifests
```yaml
# kubernetes/backend-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: imkitchen-backend
  namespace: imkitchen
spec:
  replicas: 3
  selector:
    matchLabels:
      app: imkitchen-backend
  template:
    metadata:
      labels:
        app: imkitchen-backend
    spec:
      containers:
      - name: backend
        image: imkitchen-backend:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: imkitchen-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: imkitchen-secrets  
              key: redis-url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
```
