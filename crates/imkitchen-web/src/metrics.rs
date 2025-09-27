use prometheus::{
    Gauge, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec, Opts, Registry,
};
use std::time::Instant;
use tracing::{error, info};

/// Application metrics collector
#[derive(Clone)]
pub struct AppMetrics {
    pub registry: Registry,

    // HTTP Request metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_requests_in_flight: IntGauge,

    // Database metrics
    pub db_connections_active: IntGauge,
    pub db_connections_idle: IntGauge,
    pub db_query_duration_seconds: HistogramVec,
    pub db_queries_total: IntCounterVec,

    // Application metrics
    pub app_info: IntGaugeVec,
    pub uptime_seconds: Gauge,

    // Health check metrics
    pub health_check_duration_seconds: HistogramVec,
    pub health_check_status: IntGaugeVec,

    // Event processing metrics (for Evento)
    pub events_processed_total: IntCounterVec,
    pub event_processing_duration_seconds: HistogramVec,
    pub events_in_flight: IntGauge,
}

impl AppMetrics {
    /// Create a new metrics collector with all metrics registered
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // HTTP Request metrics
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests")
                .namespace("imkitchen"),
            &["method", "endpoint", "status"],
        )?;

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .namespace("imkitchen")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
            &["method", "endpoint"],
        )?;

        let http_requests_in_flight = IntGauge::new(
            "imkitchen_http_requests_in_flight",
            "Number of HTTP requests currently being processed",
        )?;

        // Database metrics
        let db_connections_active = IntGauge::new(
            "imkitchen_db_connections_active",
            "Number of active database connections",
        )?;

        let db_connections_idle = IntGauge::new(
            "imkitchen_db_connections_idle",
            "Number of idle database connections",
        )?;

        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "db_query_duration_seconds",
                "Database query duration in seconds",
            )
            .namespace("imkitchen")
            .buckets(vec![0.0001, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
            &["query_type"],
        )?;

        let db_queries_total = IntCounterVec::new(
            Opts::new("db_queries_total", "Total number of database queries")
                .namespace("imkitchen"),
            &["query_type", "status"],
        )?;

        // Application metrics
        let app_info = IntGaugeVec::new(
            Opts::new("app_info", "Application information").namespace("imkitchen"),
            &["version", "rust_version"],
        )?;

        let uptime_seconds =
            Gauge::new("imkitchen_uptime_seconds", "Application uptime in seconds")?;

        // Health check metrics
        let health_check_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "health_check_duration_seconds",
                "Health check duration in seconds",
            )
            .namespace("imkitchen")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
            &["component"],
        )?;

        let health_check_status = IntGaugeVec::new(
            Opts::new(
                "health_check_status",
                "Health check status (0=unhealthy, 1=degraded, 2=healthy)",
            )
            .namespace("imkitchen"),
            &["component"],
        )?;

        // Event processing metrics
        let events_processed_total = IntCounterVec::new(
            Opts::new("events_processed_total", "Total number of events processed")
                .namespace("imkitchen"),
            &["event_type", "status"],
        )?;

        let event_processing_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "event_processing_duration_seconds",
                "Event processing duration in seconds",
            )
            .namespace("imkitchen")
            .buckets(vec![0.0001, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
            &["event_type"],
        )?;

        let events_in_flight = IntGauge::new(
            "imkitchen_events_in_flight",
            "Number of events currently being processed",
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(db_connections_active.clone()))?;
        registry.register(Box::new(db_connections_idle.clone()))?;
        registry.register(Box::new(db_query_duration_seconds.clone()))?;
        registry.register(Box::new(db_queries_total.clone()))?;
        registry.register(Box::new(app_info.clone()))?;
        registry.register(Box::new(uptime_seconds.clone()))?;
        registry.register(Box::new(health_check_duration_seconds.clone()))?;
        registry.register(Box::new(health_check_status.clone()))?;
        registry.register(Box::new(events_processed_total.clone()))?;
        registry.register(Box::new(event_processing_duration_seconds.clone()))?;
        registry.register(Box::new(events_in_flight.clone()))?;

        info!("Prometheus metrics initialized successfully");

        Ok(AppMetrics {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            db_connections_active,
            db_connections_idle,
            db_query_duration_seconds,
            db_queries_total,
            app_info,
            uptime_seconds,
            health_check_duration_seconds,
            health_check_status,
            events_processed_total,
            event_processing_duration_seconds,
            events_in_flight,
        })
    }

    /// Record an HTTP request
    pub fn record_http_request(
        &self,
        method: &str,
        endpoint: &str,
        status: u16,
        duration: std::time::Duration,
    ) {
        self.http_requests_total
            .with_label_values(&[method, endpoint, &status.to_string()])
            .inc();

        self.http_request_duration_seconds
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Start tracking an HTTP request in flight
    pub fn start_http_request(&self) -> HttpRequestGuard {
        self.http_requests_in_flight.inc();
        HttpRequestGuard {
            metrics: self.clone(),
            start_time: Instant::now(),
        }
    }

    /// Update database connection metrics
    pub fn update_db_connections(&self, active: u32, idle: u32) {
        self.db_connections_active.set(active as i64);
        self.db_connections_idle.set(idle as i64);
    }

    /// Record a database query
    pub fn record_db_query(&self, query_type: &str, status: &str, duration: std::time::Duration) {
        self.db_queries_total
            .with_label_values(&[query_type, status])
            .inc();

        self.db_query_duration_seconds
            .with_label_values(&[query_type])
            .observe(duration.as_secs_f64());
    }

    /// Start tracking a database query
    pub fn start_db_query(&self, query_type: &str) -> DbQueryGuard {
        DbQueryGuard {
            metrics: self.clone(),
            query_type: query_type.to_string(),
            start_time: Instant::now(),
        }
    }

    /// Update application info
    pub fn set_app_info(&self, version: &str, rust_version: &str) {
        self.app_info
            .with_label_values(&[version, rust_version])
            .set(1);
    }

    /// Update uptime
    pub fn update_uptime(&self, uptime: std::time::Duration) {
        self.uptime_seconds.set(uptime.as_secs_f64());
    }

    /// Record a health check
    pub fn record_health_check(&self, component: &str, status: i64, duration: std::time::Duration) {
        self.health_check_status
            .with_label_values(&[component])
            .set(status);

        self.health_check_duration_seconds
            .with_label_values(&[component])
            .observe(duration.as_secs_f64());
    }

    /// Record event processing
    pub fn record_event_processed(
        &self,
        event_type: &str,
        status: &str,
        duration: std::time::Duration,
    ) {
        self.events_processed_total
            .with_label_values(&[event_type, status])
            .inc();

        self.event_processing_duration_seconds
            .with_label_values(&[event_type])
            .observe(duration.as_secs_f64());
    }

    /// Start tracking event processing
    pub fn start_event_processing(&self, event_type: &str) -> EventProcessingGuard {
        self.events_in_flight.inc();
        EventProcessingGuard {
            metrics: self.clone(),
            event_type: event_type.to_string(),
            start_time: Instant::now(),
        }
    }

    /// Get metrics in Prometheus format
    pub fn gather(&self) -> String {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        match encoder.encode_to_string(&metric_families) {
            Ok(result) => result,
            Err(e) => {
                error!("Failed to encode metrics: {}", e);
                String::new()
            }
        }
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics")
    }
}

/// RAII guard for HTTP request tracking
pub struct HttpRequestGuard {
    metrics: AppMetrics,
    start_time: Instant,
}

impl Drop for HttpRequestGuard {
    fn drop(&mut self) {
        self.metrics.http_requests_in_flight.dec();
    }
}

impl HttpRequestGuard {
    /// Complete the HTTP request with status and endpoint info
    pub fn complete(self, method: &str, endpoint: &str, status: u16) {
        let duration = self.start_time.elapsed();
        self.metrics
            .record_http_request(method, endpoint, status, duration);
        // Drop happens automatically, decrementing in_flight counter
    }
}

/// RAII guard for database query tracking
pub struct DbQueryGuard {
    metrics: AppMetrics,
    query_type: String,
    start_time: Instant,
}

impl DbQueryGuard {
    /// Complete the database query with status
    pub fn complete(self, status: &str) {
        let duration = self.start_time.elapsed();
        self.metrics
            .record_db_query(&self.query_type, status, duration);
    }
}

/// RAII guard for event processing tracking
pub struct EventProcessingGuard {
    metrics: AppMetrics,
    event_type: String,
    start_time: Instant,
}

impl Drop for EventProcessingGuard {
    fn drop(&mut self) {
        self.metrics.events_in_flight.dec();
    }
}

impl EventProcessingGuard {
    /// Complete the event processing with status
    pub fn complete(self, status: &str) {
        let duration = self.start_time.elapsed();
        self.metrics
            .record_event_processed(&self.event_type, status, duration);
        // Drop happens automatically, decrementing in_flight counter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_creation() {
        let metrics = AppMetrics::new();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_http_request_recording() {
        let metrics = AppMetrics::new().unwrap();
        metrics.record_http_request("GET", "/health", 200, Duration::from_millis(50));

        let output = metrics.gather();
        assert!(output.contains("imkitchen_http_requests_total"));
        assert!(output.contains("imkitchen_http_request_duration_seconds"));
    }

    #[test]
    fn test_db_connection_metrics() {
        let metrics = AppMetrics::new().unwrap();
        metrics.update_db_connections(5, 3);

        let output = metrics.gather();
        assert!(output.contains("imkitchen_db_connections_active"));
        assert!(output.contains("imkitchen_db_connections_idle"));
    }

    #[test]
    fn test_app_info_metrics() {
        let metrics = AppMetrics::new().unwrap();
        metrics.set_app_info("1.0.0", "1.70.0");

        let output = metrics.gather();
        assert!(output.contains("imkitchen_app_info"));
    }

    #[test]
    fn test_health_check_metrics() {
        let metrics = AppMetrics::new().unwrap();
        metrics.record_health_check("database", 2, Duration::from_millis(10));

        let output = metrics.gather();
        assert!(output.contains("imkitchen_health_check_status"));
        assert!(output.contains("imkitchen_health_check_duration_seconds"));
    }

    #[test]
    fn test_event_processing_metrics() {
        let metrics = AppMetrics::new().unwrap();
        metrics.record_event_processed("UserCreated", "success", Duration::from_millis(5));

        let output = metrics.gather();
        assert!(output.contains("imkitchen_events_processed_total"));
        assert!(output.contains("imkitchen_event_processing_duration_seconds"));
    }

    #[test]
    fn test_http_request_guard() {
        let metrics = AppMetrics::new().unwrap();

        // Test guard creation and completion
        {
            let guard = metrics.start_http_request();
            guard.complete("GET", "/health", 200);
        }

        let output = metrics.gather();
        assert!(output.contains("imkitchen_http_requests_total"));
    }

    #[test]
    fn test_db_query_guard() {
        let metrics = AppMetrics::new().unwrap();

        // Test guard creation and completion
        {
            let guard = metrics.start_db_query("health_check");
            guard.complete("success");
        }

        let output = metrics.gather();
        assert!(output.contains("imkitchen_db_queries_total"));
    }

    #[test]
    fn test_event_processing_guard() {
        let metrics = AppMetrics::new().unwrap();

        // Test guard creation and completion
        {
            let guard = metrics.start_event_processing("UserCreated");
            guard.complete("success");
        }

        let output = metrics.gather();
        assert!(output.contains("imkitchen_events_processed_total"));
    }
}
