// Health check endpoint handler

use axum::{extract::State, http::StatusCode, response::Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

#[derive(Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
    pub checked_at: String,
    pub response_time_ms: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub checks: HashMap<String, ComponentHealth>,
}

/// Application state that includes health check dependencies
#[derive(Clone)]
pub struct HealthCheckState {
    pub db_pool: Option<SqlitePool>,
    pub start_time: std::time::Instant,
}

impl HealthCheckState {
    pub fn new(db_pool: Option<SqlitePool>) -> Self {
        Self {
            db_pool,
            start_time: std::time::Instant::now(),
        }
    }
}

/// Health check endpoint handler with comprehensive dependency checking
pub async fn health_check_with_deps(
    State(state): State<HealthCheckState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    info!("Performing comprehensive health check");

    let start_time = std::time::Instant::now();
    let mut checks = HashMap::new();
    let mut overall_status = HealthStatus::Healthy;

    // Check database connectivity
    let db_health = check_database_health(&state.db_pool).await;
    if matches!(db_health.status, HealthStatus::Unhealthy) {
        overall_status = HealthStatus::Unhealthy;
    } else if matches!(db_health.status, HealthStatus::Degraded)
        && matches!(overall_status, HealthStatus::Healthy)
    {
        overall_status = HealthStatus::Degraded;
    }
    checks.insert("database".to_string(), db_health);

    // Note: Event store uses the same database, so no separate check needed

    // Check system resources
    let system_health = check_system_health().await;
    if matches!(system_health.status, HealthStatus::Degraded)
        && matches!(overall_status, HealthStatus::Healthy)
    {
        overall_status = HealthStatus::Degraded;
    }
    checks.insert("system".to_string(), system_health);

    let response = HealthResponse {
        status: overall_status.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        checks,
    };

    let _status_code = match overall_status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // 200 but with warnings
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    info!(
        "Health check completed in {}ms with status: {:?}",
        start_time.elapsed().as_millis(),
        overall_status
    );

    Ok(Json(response))
}

/// Basic health check endpoint handler (for backward compatibility)
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    let response = HealthResponse {
        status: HealthStatus::Healthy,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: 0, // Unknown uptime for basic check
        checks: HashMap::new(),
    };

    Ok(Json(response))
}

/// Check database connectivity and health
async fn check_database_health(db_pool: &Option<SqlitePool>) -> ComponentHealth {
    let start_time = std::time::Instant::now();

    match db_pool {
        Some(pool) => {
            // Test database connectivity with a simple query
            match sqlx::query_scalar::<_, i32>("SELECT 1")
                .fetch_one(pool)
                .await
            {
                Ok(result) if result == 1 => {
                    let mut details = HashMap::new();
                    details.insert(
                        "connections_active".to_string(),
                        serde_json::Value::Number(pool.size().into()),
                    );
                    details.insert(
                        "connections_idle".to_string(),
                        serde_json::Value::Number((pool.num_idle() as u32).into()),
                    );

                    ComponentHealth {
                        status: HealthStatus::Healthy,
                        message: "Database connection successful".to_string(),
                        details: Some(details),
                        checked_at: chrono::Utc::now().to_rfc3339(),
                        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }
                }
                Ok(_) => {
                    warn!("Database query returned unexpected result");
                    ComponentHealth {
                        status: HealthStatus::Degraded,
                        message: "Database query returned unexpected result".to_string(),
                        details: None,
                        checked_at: chrono::Utc::now().to_rfc3339(),
                        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }
                }
                Err(e) => {
                    error!("Database health check failed: {}", e);
                    ComponentHealth {
                        status: HealthStatus::Unhealthy,
                        message: format!("Database connection failed: {}", e),
                        details: None,
                        checked_at: chrono::Utc::now().to_rfc3339(),
                        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
                    }
                }
            }
        }
        None => ComponentHealth {
            status: HealthStatus::Unhealthy,
            message: "No database pool configured".to_string(),
            details: None,
            checked_at: chrono::Utc::now().to_rfc3339(),
            response_time_ms: Some(start_time.elapsed().as_millis() as u64),
        },
    }
}

/// Check system health (memory, disk, etc.)
async fn check_system_health() -> ComponentHealth {
    let start_time = std::time::Instant::now();

    // Basic system health checks
    let mut details = HashMap::new();
    let mut issues = Vec::new();

    // Check if we can create temporary files (disk health)
    match std::fs::File::create_new("/tmp/imkitchen_health_check") {
        Ok(_) => {
            let _ = std::fs::remove_file("/tmp/imkitchen_health_check");
            details.insert("disk_writable".to_string(), serde_json::Value::Bool(true));
        }
        Err(_) => {
            details.insert("disk_writable".to_string(), serde_json::Value::Bool(false));
            issues.push("Cannot write to disk");
        }
    }

    // Add basic system information
    details.insert(
        "rust_version".to_string(),
        serde_json::Value::String(env!("CARGO_PKG_RUST_VERSION").to_string()),
    );

    let status = if issues.is_empty() {
        HealthStatus::Healthy
    } else {
        HealthStatus::Degraded
    };

    let message = if issues.is_empty() {
        "System health checks passed".to_string()
    } else {
        format!("System issues detected: {}", issues.join(", "))
    };

    ComponentHealth {
        status,
        message,
        details: Some(details),
        checked_at: chrono::Utc::now().to_rfc3339(),
        response_time_ms: Some(start_time.elapsed().as_millis() as u64),
    }
}
