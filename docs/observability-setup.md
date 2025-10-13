# Observability Setup Guide

This guide explains how to use the OpenTelemetry (OTEL) collector and Jaeger for distributed tracing in imkitchen.

## Architecture

```
┌──────────────┐      OTLP/gRPC      ┌──────────────────┐      gRPC       ┌──────────┐
│  imkitchen   │ ──────────────────> │ OTEL Collector   │ ──────────────> │  Jaeger  │
│ (Rust app)   │     :4317           │  (Aggregator)    │    :14250       │   (UI)   │
└──────────────┘                     └──────────────────┘                 └──────────┘
                                              │
                                              │ Metrics
                                              ▼
                                     ┌──────────────────┐
                                     │   Prometheus     │
                                     │   (:8888/metrics)│
                                     └──────────────────┘
```

## Quick Start

### 1. Start the Observability Stack

```bash
# Start OTEL collector, Jaeger, and MailDev
docker compose up -d

# Check services are running
docker compose ps

# Expected output:
# imkitchen-otel      running   0.0.0.0:4317->4317/tcp, 0.0.0.0:4318->4318/tcp
# imkitchen-jaeger    running   0.0.0.0:16686->16686/tcp
# imkitchen-maildev   running   0.0.0.0:1080->1080/tcp, 0.0.0.0:1025->1025/tcp
```

### 2. Run imkitchen Application

```bash
# The app is already configured to send traces to localhost:4319 (OTEL collector)
cargo run -- serve

# OTEL is initialized automatically on startup
# You should see: "Observability initialized with OpenTelemetry OTLP exporter"
```

### 3. Access UIs

| Service | URL | Purpose |
|---------|-----|---------|
| **Jaeger UI** | http://localhost:16686 | View distributed traces, service graph |
| **Prometheus Metrics** | http://localhost:9090/metrics | Raw metrics from OTEL collector |
| **OTEL Collector Health** | http://localhost:13133 | Collector health check |
| **OTEL zPages** | http://localhost:55679/debug/servicez | Collector debugging info |
| **MailDev** | http://localhost:1080 | Email testing UI |

### 4. Generate Traces

Traces are generated automatically for all HTTP requests:

```bash
# Make some requests to generate traces
curl http://localhost:3000/health
curl http://localhost:3000/login
curl -X POST http://localhost:3000/register -d '{"email":"test@example.com","password":"test123"}'
```

### 5. View Traces in Jaeger

1. Open http://localhost:16686
2. Select "imkitchen" from Service dropdown
3. Click "Find Traces"
4. Click on any trace to see span details

## Trace Attributes

The application automatically captures:
- **HTTP spans**: Method, path, status code, duration
- **Database spans**: SQL queries, execution time
- **Event sourcing spans**: Event type, aggregate ID
- **Custom spans**: Business logic operations

## Configuration

### Application Config

Edit `config/default.toml`:

```toml
[observability]
otel_endpoint = "http://localhost:4319"  # OTLP gRPC endpoint
log_level = "info"                        # trace, debug, info, warn, error
```

### Environment Variables

Override config with environment variables:

```bash
# Change OTEL endpoint
export IMKITCHEN__OBSERVABILITY__OTEL_ENDPOINT="http://otel-collector:4319"

# Change log level
export IMKITCHEN__OBSERVABILITY__LOG_LEVEL="debug"

# Or use RUST_LOG for more control
export RUST_LOG="imkitchen=debug,axum=info,sqlx=warn"

# Enable production JSON logging
export ENVIRONMENT="production"
```

## OTEL Collector Configuration

The collector config is in `otel-collector-config.yaml`:

- **Receivers**: OTLP gRPC (:4319) and HTTP (:4320)
- **Processors**: Batching, memory limiting, resource attributes
- **Exporters**: Jaeger (traces via OTLP), Prometheus (:9090/metrics), Debug (logging)

## Troubleshooting

### Traces not appearing in Jaeger

1. **Check OTEL collector logs:**
   ```bash
   docker compose logs otel-collector
   ```

2. **Verify OTEL endpoint:**
   ```bash
   # Should show 4319 is listening
   netstat -an | grep 4319
   ```

3. **Check application logs:**
   ```bash
   cargo run -- serve 2>&1 | grep -i otel
   # Should see: "Observability initialized with OpenTelemetry OTLP exporter"
   ```

4. **Test OTEL collector health:**
   ```bash
   curl http://localhost:13133
   # Should return: {"status":"Server available","upSince":"..."}
   ```

### High memory usage

Adjust OTEL collector memory limits in `otel-collector-config.yaml`:

```yaml
processors:
  memory_limiter:
    check_interval: 1s
    limit_mib: 256      # Reduce from 512
    spike_limit_mib: 64 # Reduce from 128
```

### Missing spans

Check if tracing is enabled for the code:

```rust
#[tracing::instrument(skip(state))]  // <-- Add this attribute
pub async fn my_handler(State(state): State<AppState>) -> Response {
    // Handler code
}
```

## Production Deployment

For production, use a managed OTEL collector service:

1. **Cloud Providers:**
   - AWS X-Ray (via OTEL collector)
   - Google Cloud Trace
   - Azure Monitor
   - Datadog APM
   - New Relic
   - Honeycomb

2. **Update endpoint:**
   ```bash
   export IMKITCHEN__OBSERVABILITY__OTEL_ENDPOINT="https://your-collector.example.com:4317"
   # Note: Most managed services use standard OTLP port 4317
   ```

3. **Enable authentication:**
   Update `otel-collector-config.yaml` to add API keys/tokens for your backend.

## Additional Resources

- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [OTEL Collector Docs](https://opentelemetry.io/docs/collector/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Distributed Tracing Best Practices](https://opentelemetry.io/docs/concepts/observability-primer/)

## Metrics Dashboard (Future)

To visualize Prometheus metrics, add Grafana:

```yaml
# Add to compose.yml
grafana:
  image: grafana/grafana:latest
  ports:
    - "3001:3000"
  environment:
    - GF_SECURITY_ADMIN_PASSWORD=admin
  volumes:
    - grafana-data:/var/lib/grafana
```

Then import Grafana dashboards for OTEL metrics visualization.
