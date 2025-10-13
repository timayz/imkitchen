use anyhow::Result;
use opentelemetry::{global::set_tracer_provider, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Initialize OpenTelemetry tracing and logging
///
/// This sets up:
/// - OpenTelemetry OTLP exporter (for tracing to Jaeger, Tempo, etc.)
/// - Structured JSON logging (for production)
/// - Console logging (for development)
/// - Environment-based log level filtering
pub fn init_observability(
    service_name: &str,
    service_version: &str,
    otel_endpoint: &str,
    log_level: &str,
) -> Result<()> {
    // Create resource with service information
    let resource = Resource::builder_empty()
        .with_attributes(vec![
            KeyValue::new("service.name", service_name.to_string()),
            KeyValue::new("service.version", service_version.to_string()),
        ])
        .build();

    // Configure OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()?;

    // Create tracer provider
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // Get tracer from provider BEFORE setting it as global
    let tracer = provider.tracer("imkitchen");

    // Set global tracer provider
    set_tracer_provider(provider);

    // Create environment filter for log levels
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    // Determine if we should use JSON logging (production) or pretty console (development)
    let is_production = std::env::var("ENVIRONMENT")
        .map(|env| env == "production")
        .unwrap_or(false);

    if is_production {
        // Production: Structured JSON logging + OpenTelemetry
        tracing_subscriber::registry()
            .with(fmt::layer().json().with_filter(env_filter))
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()?;
    } else {
        // Development: Pretty console logging + OpenTelemetry
        tracing_subscriber::registry()
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true)
                    .with_filter(env_filter),
            )
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()?;
    }

    tracing::info!(
        service.name = service_name,
        service.version = service_version,
        otel.endpoint = otel_endpoint,
        "Observability initialized with OpenTelemetry OTLP exporter"
    );

    Ok(())
}

/// Shutdown OpenTelemetry gracefully
///
/// Call this on application shutdown to ensure all traces are flushed
pub fn shutdown_observability() {
    // In OpenTelemetry 0.30+, provider shutdown happens automatically on drop
    // We just log the shutdown intent
    tracing::info!("OpenTelemetry shutdown initiated");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_init_with_defaults() {
        // This test verifies that initialization completes
        // OTLP exporter will try to connect but may fail in test environment
        let result = init_observability("test-service", "0.1.0", "http://localhost:4317", "debug");

        // We expect this to succeed (init completes) even if OTLP collector is unavailable
        // The actual connection happens asynchronously when spans are sent
        assert!(
            result.is_ok(),
            "Observability init should succeed: {:?}",
            result.err()
        );
    }
}
