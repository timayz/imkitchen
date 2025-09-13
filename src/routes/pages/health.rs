use axum::{http::StatusCode, response::Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    pub services: ServiceStatus,
    pub system: SystemInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ServiceStatus {
    pub database: ServiceHealth,
    pub redis: ServiceHealth,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SystemInfo {
    pub memory_usage_mb: u64,
    pub cpu_load: f64,
    pub disk_usage_percent: f64,
}

static START_TIME: std::sync::OnceLock<SystemTime> = std::sync::OnceLock::new();

pub async fn health_check() -> Result<(StatusCode, Json<HealthResponse>), StatusCode> {
    let start_time = START_TIME.get_or_init(|| SystemTime::now());
    let uptime = SystemTime::now()
        .duration_since(*start_time)
        .unwrap_or_default()
        .as_secs();

    // Check database connection (placeholder - will be implemented with actual connections)
    let database_status = check_database_health().await;

    // Check Redis connection (placeholder - will be implemented with actual connections)
    let redis_status = check_redis_health().await;

    // Get system information
    let system_info = get_system_info();

    // Determine overall status
    let overall_status = if database_status.status == "healthy" && redis_status.status == "healthy"
    {
        "healthy"
    } else {
        "degraded"
    };

    let health_response = HealthResponse {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        services: ServiceStatus {
            database: database_status.clone(),
            redis: redis_status.clone(),
        },
        system: system_info,
    };

    // Return appropriate HTTP status based on health
    match overall_status {
        "healthy" => Ok((StatusCode::OK, Json(health_response))),
        _ => {
            tracing::warn!("Health check failed: overall status is {}, database error: {:?}, redis error: {:?}", 
                          overall_status, 
                          &database_status.error, 
                          &redis_status.error);
            // Return JSON response even when degraded so we can see the errors
            Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health_response)))
        }
    }
}

async fn check_database_health() -> ServiceHealth {
    use crate::config::{database, Settings};
    use std::time::Instant;

    let start = Instant::now();

    // Get database URL from settings
    let settings = match Settings::new() {
        Ok(settings) => settings,
        Err(e) => {
            return ServiceHealth {
                status: "unhealthy".to_string(),
                response_time_ms: None,
                error: Some(format!("Failed to load configuration: {}", e)),
            };
        }
    };

    // Test database connection
    match database::create_pool(&settings.database.url).await {
        Ok(pool) => match database::test_connection(&pool).await {
            Ok(_) => {
                let response_time = start.elapsed().as_millis() as u64;
                ServiceHealth {
                    status: "healthy".to_string(),
                    response_time_ms: Some(response_time),
                    error: None,
                }
            }
            Err(e) => ServiceHealth {
                status: "unhealthy".to_string(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(format!("Database query failed: {}", e)),
            },
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(format!("Database connection failed: {}", e)),
        },
    }
}

async fn check_redis_health() -> ServiceHealth {
    use crate::config::{redis, Settings};
    use std::time::Instant;

    let start = Instant::now();

    // Get Redis URL from settings
    let settings = match Settings::new() {
        Ok(settings) => settings,
        Err(e) => {
            return ServiceHealth {
                status: "unhealthy".to_string(),
                response_time_ms: None,
                error: Some(format!("Failed to load configuration: {}", e)),
            };
        }
    };

    // Test Redis connection
    match redis::create_client(&settings.redis.url).await {
        Ok(client) => match redis::test_connection(&client).await {
            Ok(_) => {
                let response_time = start.elapsed().as_millis() as u64;
                ServiceHealth {
                    status: "healthy".to_string(),
                    response_time_ms: Some(response_time),
                    error: None,
                }
            }
            Err(e) => ServiceHealth {
                status: "unhealthy".to_string(),
                response_time_ms: Some(start.elapsed().as_millis() as u64),
                error: Some(format!("Redis command failed: {}", e)),
            },
        },
        Err(e) => ServiceHealth {
            status: "unhealthy".to_string(),
            response_time_ms: Some(start.elapsed().as_millis() as u64),
            error: Some(format!("Redis connection failed: {}", e)),
        },
    }
}

fn get_system_info() -> SystemInfo {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    // Get memory usage in MB
    let used_memory = sys.used_memory();
    let memory_usage_mb = (used_memory / 1024 / 1024) as u64;

    // Get average CPU usage
    let cpu_load =
        sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32 / 100.0;

    // Get disk usage - simplified approach since sysinfo API varies by version
    let disk_usage_percent = 45.0; // Placeholder - will be improved in future version

    SystemInfo {
        memory_usage_mb,
        cpu_load: cpu_load as f64,
        disk_usage_percent,
    }
}
