use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_basic_health_check_endpoint() {
    // Test that basic health check endpoint works
    let result = imkitchen_web::handlers::health::health_check().await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.0.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_health_check_state_creation() {
    use imkitchen_web::HealthCheckState;

    // Test creating health check state without database
    let state = HealthCheckState::new(None);
    assert!(state.db_pool.is_none());

    // Test creating health check state with in-memory database
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    let state_with_db = HealthCheckState::new(Some(pool));
    assert!(state_with_db.db_pool.is_some());
}

#[tokio::test]
async fn test_health_check_with_database() {
    use axum::extract::State;
    use imkitchen_web::{
        handlers::health::{health_check_with_deps, HealthCheckState},
        HealthStatus,
    };

    // Create in-memory database
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    let state = HealthCheckState::new(Some(pool));

    // Test health check with database
    let result = health_check_with_deps(State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(matches!(
        response.0.status,
        HealthStatus::Healthy | HealthStatus::Degraded
    ));
    assert!(response.0.checks.contains_key("database"));
    assert!(response.0.checks.contains_key("system"));
}

#[tokio::test]
async fn test_health_check_without_database() {
    use axum::extract::State;
    use imkitchen_web::{
        handlers::health::{health_check_with_deps, HealthCheckState},
        HealthStatus,
    };

    let state = HealthCheckState::new(None);

    // Test health check without database
    let result = health_check_with_deps(State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(matches!(response.0.status, HealthStatus::Unhealthy));
    assert!(response.0.checks.contains_key("database"));
    assert!(response.0.checks.contains_key("system"));
}

#[tokio::test]
async fn test_database_config_creation() {
    use imkitchen_web::DatabaseConfig;
    use std::time::Duration;

    let config = DatabaseConfig::default();
    assert_eq!(config.url, "sqlite:imkitchen.db");
    assert_eq!(config.max_connections, 10);
    assert_eq!(config.min_connections, 1);

    let custom_config = DatabaseConfig::from_url("sqlite:test.db".to_string())
        .with_max_connections(5)
        .with_timeouts(Duration::from_secs(10), Duration::from_secs(5));

    assert_eq!(custom_config.url, "sqlite:test.db");
    assert_eq!(custom_config.max_connections, 5);
    assert_eq!(custom_config.connect_timeout, Duration::from_secs(10));
    assert_eq!(custom_config.acquire_timeout, Duration::from_secs(5));
}

#[tokio::test]
async fn test_database_pool_creation() {
    use imkitchen_web::{create_database_pool_with_retry, DatabaseConfig};
    use std::time::Duration;

    // Test creating pool with in-memory database
    let config = DatabaseConfig::from_url("sqlite::memory:".to_string());
    let pool_result = create_database_pool_with_retry(&config, 2, Duration::from_millis(100)).await;

    assert!(pool_result.is_ok());

    let pool = pool_result.unwrap();
    assert!(!pool.is_closed());
    assert!(pool.size() > 0);
}

#[tokio::test]
async fn test_database_pool_retry_logic() {
    use imkitchen_web::{create_database_pool_with_retry, DatabaseConfig};
    use std::time::Duration;

    // Test with invalid database URL to trigger retry logic
    let config = DatabaseConfig::from_url("sqlite:/invalid/path/nonexistent.db".to_string());
    let pool_result = create_database_pool_with_retry(&config, 2, Duration::from_millis(10)).await;

    // Should fail after retries
    assert!(pool_result.is_err());
}

#[tokio::test]
async fn test_health_check_response_format() {
    use axum::extract::State;
    use imkitchen_web::handlers::health::{health_check_with_deps, HealthCheckState};

    let state = HealthCheckState::new(None);
    let result = health_check_with_deps(State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();

    // Verify response structure
    assert!(!response.0.version.is_empty());
    assert!(!response.0.timestamp.is_empty());
    // Uptime should be reasonable (not 0 unless just started)
    assert!(response.0.uptime_seconds < 3600); // Less than 1 hour for test
    assert!(!response.0.checks.is_empty());

    // Verify all expected health checks are present
    assert!(response.0.checks.contains_key("database"));
    assert!(response.0.checks.contains_key("system"));

    // Verify component health structure
    for (name, check) in response.0.checks {
        assert!(
            !check.message.is_empty(),
            "Health check '{}' should have a message",
            name
        );
        assert!(
            !check.checked_at.is_empty(),
            "Health check '{}' should have a timestamp",
            name
        );
        // response_time_ms might be None for some checks
    }
}

#[tokio::test]
async fn test_health_check_performance() {
    use axum::extract::State;
    use imkitchen_web::handlers::health::{health_check_with_deps, HealthCheckState};
    use std::time::Instant;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    let state = HealthCheckState::new(Some(pool));

    // Test that health check completes within reasonable time
    let start = Instant::now();
    let result = timeout(Duration::from_secs(5), health_check_with_deps(State(state))).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Health check timed out");
    assert!(result.unwrap().is_ok(), "Health check failed");
    assert!(
        elapsed < Duration::from_secs(2),
        "Health check took too long: {:?}",
        elapsed
    );
}

#[test]
fn test_database_config_builder_pattern() {
    use imkitchen_web::DatabaseConfig;
    use std::time::Duration;

    let config = DatabaseConfig::from_url("sqlite:custom.db".to_string())
        .with_max_connections(20)
        .with_timeouts(Duration::from_secs(15), Duration::from_secs(8));

    assert_eq!(config.url, "sqlite:custom.db");
    assert_eq!(config.max_connections, 20);
    assert_eq!(config.connect_timeout, Duration::from_secs(15));
    assert_eq!(config.acquire_timeout, Duration::from_secs(8));
    assert_eq!(config.min_connections, 1); // Should keep default
}

#[tokio::test]
async fn test_health_check_error_scenarios() {
    use axum::extract::State;
    use imkitchen_web::{
        handlers::health::{health_check_with_deps, HealthCheckState},
        HealthStatus,
    };

    // Test with closed database pool
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    pool.close().await;

    let state = HealthCheckState::new(Some(pool));
    let result = health_check_with_deps(State(state)).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    // Should be unhealthy due to closed database
    assert!(matches!(response.0.status, HealthStatus::Unhealthy));

    let db_check = response.0.checks.get("database").unwrap();
    assert!(matches!(db_check.status, HealthStatus::Unhealthy));
}
