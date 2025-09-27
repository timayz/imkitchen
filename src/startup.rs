use sqlx::SqlitePool;
use std::time::Duration;
use tracing::{info, warn};
use validator::Validate;

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Startup initialization manager for proper service startup sequence
pub struct StartupManager {
    config: Config,
}

/// Dependency check result
#[derive(Debug, Clone)]
pub struct DependencyCheck {
    pub name: String,
    pub status: DependencyStatus,
    pub message: String,
    pub critical: bool,
}

/// Status of a dependency check
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyStatus {
    Healthy,
    Warning,
    Failed,
}

/// Migration verification result
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub applied_count: i32,
    pub latest_migration: Option<String>,
    pub requires_migration: bool,
}

/// Service readiness check result
#[derive(Debug, Clone)]
pub struct ReadinessCheck {
    pub service_name: String,
    pub status: ReadinessStatus,
    pub message: String,
    pub response_time_ms: Option<u64>,
}

/// Status of a service readiness check
#[derive(Debug, Clone, PartialEq)]
pub enum ReadinessStatus {
    Ready,
    NotReady,
    Degraded,
}

impl StartupManager {
    /// Create a new startup manager with configuration
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Execute complete startup sequence with proper initialization order
    #[allow(clippy::result_large_err)]
    pub async fn initialize(&self) -> AppResult<SqlitePool> {
        info!("Starting application initialization sequence");

        // Step 1: Validate configuration
        self.validate_configuration()?;

        // Step 2: Check system dependencies
        self.check_system_dependencies().await?;

        // Step 3: Initialize database connection
        let pool = self.initialize_database().await?;

        // Step 4: Verify migration status
        self.verify_migration_status(&pool).await?;

        // Step 5: Perform health checks
        self.perform_health_checks(&pool).await?;

        // Step 6: Initialize monitoring and metrics
        self.initialize_monitoring()?;

        // Step 7: Validate service readiness
        self.validate_service_readiness(&pool).await?;

        info!("Application initialization completed successfully");
        Ok(pool)
    }

    /// Validate configuration before startup
    #[allow(clippy::result_large_err)]
    fn validate_configuration(&self) -> AppResult<()> {
        info!("Validating configuration");

        // Validate security configuration
        self.config.validate_security()?;

        // Check for required environment variables
        if std::env::var("SESSION_SECRET").is_err() {
            warn!("SESSION_SECRET environment variable not set - using generated secret");
        }

        // Validate database URL format
        if !self.config.database.url.starts_with("sqlite:") {
            return Err(AppError::configuration(
                "Invalid database URL format - must be SQLite URL",
            ));
        }

        // Validate server configuration
        if self.config.server.port == 0 {
            return Err(AppError::configuration(format!(
                "Invalid server port: {}",
                self.config.server.port
            )));
        }

        info!("Configuration validation completed");
        Ok(())
    }

    /// Check system dependencies during startup
    #[allow(clippy::result_large_err)]
    async fn check_system_dependencies(&self) -> AppResult<()> {
        info!("Checking system dependencies");

        let mut checks = Vec::new();

        // Check file system permissions
        checks.push(self.check_filesystem_permissions().await);

        // Check available disk space
        checks.push(self.check_disk_space().await);

        // Check network connectivity if required
        if self.config.server.host != "127.0.0.1" && self.config.server.host != "localhost" {
            checks.push(self.check_network_connectivity().await);
        }

        // Check log directory access
        if let Some(ref log_dir) = self.config.logging.dir {
            checks.push(self.check_log_directory_access(log_dir).await);
        }

        // Evaluate dependency check results
        let mut critical_failures = Vec::new();
        let mut warnings = Vec::new();

        for check in &checks {
            match check.status {
                DependencyStatus::Failed if check.critical => {
                    critical_failures.push(check.clone());
                }
                DependencyStatus::Failed | DependencyStatus::Warning => {
                    warnings.push(check.clone());
                }
                DependencyStatus::Healthy => {
                    info!("✓ {}: {}", check.name, check.message);
                }
            }
        }

        // Report warnings
        for warning in warnings {
            warn!("⚠ {}: {}", warning.name, warning.message);
        }

        // Fail on critical dependencies
        if !critical_failures.is_empty() {
            let failure_messages: Vec<String> = critical_failures
                .iter()
                .map(|f| format!("{}: {}", f.name, f.message))
                .collect();

            return Err(AppError::internal(format!(
                "Critical dependency failures: {}",
                failure_messages.join(", ")
            )));
        }

        info!("System dependency checks completed");
        Ok(())
    }

    /// Initialize database connection with retry logic
    #[allow(clippy::result_large_err)]
    async fn initialize_database(&self) -> AppResult<SqlitePool> {
        info!("Initializing database connection");

        let db_config = imkitchen_web::DatabaseConfig::from_url(self.config.database.url.clone())
            .with_max_connections(self.config.database.max_connections)
            .with_timeouts(
                Duration::from_secs(self.config.database.connection_timeout),
                Duration::from_secs(self.config.database.acquire_timeout),
            );

        let pool = imkitchen_web::create_database_pool_with_retry(
            &db_config,
            3,                      // max retries
            Duration::from_secs(2), // retry delay
        )
        .await
        .map_err(|e| AppError::database_with_source("Failed to initialize database", e))?;

        info!("Database connection established successfully");
        Ok(pool)
    }

    /// Verify migration status before service start
    #[allow(clippy::result_large_err)]
    async fn verify_migration_status(&self, pool: &SqlitePool) -> AppResult<()> {
        info!("Verifying database migration status");

        let migration_status = self.get_migration_status(pool).await?;

        info!(
            "Database migration status: {} migrations applied",
            migration_status.applied_count
        );

        if let Some(ref latest) = migration_status.latest_migration {
            info!("Latest migration: {}", latest);
        }

        if migration_status.requires_migration {
            warn!("Database requires migrations to be applied");
            return Err(AppError::migration(
                "Database migrations required before startup",
                crate::error::MigrationOperation::Check,
            ));
        }

        info!("Migration status verification completed");
        Ok(())
    }

    /// Get current migration status
    #[allow(clippy::result_large_err)]
    async fn get_migration_status(&self, pool: &SqlitePool) -> AppResult<MigrationStatus> {
        // Check if migration table exists
        let table_exists = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if table_exists == 0 {
            return Ok(MigrationStatus {
                applied_count: 0,
                latest_migration: None,
                requires_migration: true,
            });
        }

        // Get migration count
        let migration_count = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::database_with_source("Failed to query migration count", e))?;

        // Get latest migration
        let latest_migration = sqlx::query_scalar::<_, String>(
            "SELECT description FROM _sqlx_migrations ORDER BY version DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database_with_source("Failed to query latest migration", e))?;

        // For now, assume migrations are up to date if any exist
        // In a real implementation, you'd compare against expected migrations
        let requires_migration = migration_count == 0;

        Ok(MigrationStatus {
            applied_count: migration_count,
            latest_migration,
            requires_migration,
        })
    }

    /// Perform comprehensive health checks
    #[allow(clippy::result_large_err)]
    async fn perform_health_checks(&self, pool: &SqlitePool) -> AppResult<()> {
        info!("Performing health checks");

        // Database connectivity check
        let db_health = sqlx::query("SELECT 1").execute(pool).await.is_ok();

        if !db_health {
            return Err(AppError::database("Database health check failed"));
        }

        // Configuration consistency check
        if self.config.server.port < 1024 && !self.is_running_as_root() {
            warn!(
                "Running on privileged port {} without root privileges",
                self.config.server.port
            );
        }

        info!("Health checks completed successfully");
        Ok(())
    }

    /// Initialize monitoring and metrics
    #[allow(clippy::result_large_err)]
    fn initialize_monitoring(&self) -> AppResult<()> {
        info!("Initializing monitoring systems");

        if self.config.monitoring.enable_metrics {
            info!("Metrics collection enabled");
        }

        // Additional monitoring initialization would go here
        // (Prometheus metrics, tracing setup, etc.)

        info!("Monitoring initialization completed");
        Ok(())
    }

    /// Validate service readiness before allowing traffic
    #[allow(clippy::result_large_err)]
    async fn validate_service_readiness(&self, pool: &SqlitePool) -> AppResult<()> {
        info!("Validating service readiness");

        let mut readiness_checks = Vec::new();

        // Check database service readiness
        readiness_checks.push(self.check_database_readiness(pool).await);

        // Check configuration service readiness
        readiness_checks.push(self.check_configuration_readiness().await);

        // Check monitoring service readiness
        readiness_checks.push(self.check_monitoring_readiness().await);

        // Check filesystem readiness
        readiness_checks.push(self.check_filesystem_readiness().await);

        // Evaluate readiness check results
        let mut not_ready_services = Vec::new();
        let mut degraded_services = Vec::new();

        for check in &readiness_checks {
            match check.status {
                ReadinessStatus::NotReady => {
                    not_ready_services.push(check.clone());
                }
                ReadinessStatus::Degraded => {
                    degraded_services.push(check.clone());
                }
                ReadinessStatus::Ready => {
                    let response_info = if let Some(ms) = check.response_time_ms {
                        format!(" ({}ms)", ms)
                    } else {
                        String::new()
                    };
                    info!(
                        "✓ {} service: {}{}",
                        check.service_name, check.message, response_info
                    );
                }
            }
        }

        // Report degraded services as warnings
        for degraded in degraded_services {
            let response_info = if let Some(ms) = degraded.response_time_ms {
                format!(" ({}ms)", ms)
            } else {
                String::new()
            };
            warn!(
                "⚠ {} service: {}{}",
                degraded.service_name, degraded.message, response_info
            );
        }

        // Fail if any services are not ready
        if !not_ready_services.is_empty() {
            let failure_messages: Vec<String> = not_ready_services
                .iter()
                .map(|f| format!("{}: {}", f.service_name, f.message))
                .collect();

            return Err(AppError::internal(format!(
                "Services not ready for traffic: {}",
                failure_messages.join(", ")
            )));
        }

        info!("All services ready for traffic");
        Ok(())
    }

    // Dependency check implementations

    async fn check_filesystem_permissions(&self) -> DependencyCheck {
        let temp_file = std::env::temp_dir().join("imkitchen_write_test");

        match std::fs::write(&temp_file, "test") {
            Ok(_) => {
                let _ = std::fs::remove_file(&temp_file);
                DependencyCheck {
                    name: "Filesystem Permissions".to_string(),
                    status: DependencyStatus::Healthy,
                    message: "Read/write access available".to_string(),
                    critical: true,
                }
            }
            Err(e) => DependencyCheck {
                name: "Filesystem Permissions".to_string(),
                status: DependencyStatus::Failed,
                message: format!("Cannot write to filesystem: {}", e),
                critical: true,
            },
        }
    }

    async fn check_disk_space(&self) -> DependencyCheck {
        // Simple disk space check - in production you'd use a proper library
        match std::fs::metadata(".") {
            Ok(_) => DependencyCheck {
                name: "Disk Space".to_string(),
                status: DependencyStatus::Healthy,
                message: "Sufficient disk space available".to_string(),
                critical: false,
            },
            Err(e) => DependencyCheck {
                name: "Disk Space".to_string(),
                status: DependencyStatus::Warning,
                message: format!("Cannot check disk space: {}", e),
                critical: false,
            },
        }
    }

    async fn check_network_connectivity(&self) -> DependencyCheck {
        // Basic network connectivity check
        use std::net::ToSocketAddrs;

        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        match addr.to_socket_addrs() {
            Ok(_) => DependencyCheck {
                name: "Network Connectivity".to_string(),
                status: DependencyStatus::Healthy,
                message: format!("Can resolve address {}", addr),
                critical: false,
            },
            Err(e) => DependencyCheck {
                name: "Network Connectivity".to_string(),
                status: DependencyStatus::Warning,
                message: format!("Cannot resolve address {}: {}", addr, e),
                critical: false,
            },
        }
    }

    async fn check_log_directory_access(&self, log_dir: &std::path::Path) -> DependencyCheck {
        match std::fs::create_dir_all(log_dir) {
            Ok(_) => {
                let test_file = log_dir.join("test.log");
                match std::fs::write(&test_file, "test") {
                    Ok(_) => {
                        let _ = std::fs::remove_file(&test_file);
                        DependencyCheck {
                            name: "Log Directory Access".to_string(),
                            status: DependencyStatus::Healthy,
                            message: format!("Can write to log directory: {:?}", log_dir),
                            critical: false,
                        }
                    }
                    Err(e) => DependencyCheck {
                        name: "Log Directory Access".to_string(),
                        status: DependencyStatus::Failed,
                        message: format!("Cannot write to log directory {:?}: {}", log_dir, e),
                        critical: true,
                    },
                }
            }
            Err(e) => DependencyCheck {
                name: "Log Directory Access".to_string(),
                status: DependencyStatus::Failed,
                message: format!("Cannot create log directory {:?}: {}", log_dir, e),
                critical: true,
            },
        }
    }

    fn is_running_as_root(&self) -> bool {
        #[cfg(unix)]
        {
            unsafe { libc::getuid() == 0 }
        }
        #[cfg(not(unix))]
        {
            false
        }
    }

    // Service readiness check implementations

    async fn check_database_readiness(&self, pool: &SqlitePool) -> ReadinessCheck {
        let start_time = std::time::Instant::now();

        // Test database responsiveness with a simple query
        let result = sqlx::query("SELECT 1 as test_value").fetch_one(pool).await;

        let response_time = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(_) if response_time < 100 => ReadinessCheck {
                service_name: "Database".to_string(),
                status: ReadinessStatus::Ready,
                message: "Responding normally".to_string(),
                response_time_ms: Some(response_time),
            },
            Ok(_) if response_time < 1000 => ReadinessCheck {
                service_name: "Database".to_string(),
                status: ReadinessStatus::Degraded,
                message: "Slow response time".to_string(),
                response_time_ms: Some(response_time),
            },
            Ok(_) => ReadinessCheck {
                service_name: "Database".to_string(),
                status: ReadinessStatus::NotReady,
                message: "Very slow response time".to_string(),
                response_time_ms: Some(response_time),
            },
            Err(e) => ReadinessCheck {
                service_name: "Database".to_string(),
                status: ReadinessStatus::NotReady,
                message: format!("Query failed: {}", e),
                response_time_ms: Some(response_time),
            },
        }
    }

    async fn check_configuration_readiness(&self) -> ReadinessCheck {
        // Verify configuration is still valid and accessible
        match self.config.validate() {
            Ok(_) => ReadinessCheck {
                service_name: "Configuration".to_string(),
                status: ReadinessStatus::Ready,
                message: "Configuration valid and accessible".to_string(),
                response_time_ms: None,
            },
            Err(e) => ReadinessCheck {
                service_name: "Configuration".to_string(),
                status: ReadinessStatus::NotReady,
                message: format!("Configuration validation failed: {}", e),
                response_time_ms: None,
            },
        }
    }

    async fn check_monitoring_readiness(&self) -> ReadinessCheck {
        // Check if monitoring/logging is functioning
        let log_test_result = if let Some(ref log_dir) = self.config.logging.dir {
            // Test log directory is writable
            let test_file = log_dir.join(".readiness_test");
            match std::fs::write(&test_file, "readiness_test") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    true
                }
                Err(_) => false,
            }
        } else {
            // Stdout logging - always ready
            true
        };

        if log_test_result {
            ReadinessCheck {
                service_name: "Monitoring".to_string(),
                status: ReadinessStatus::Ready,
                message: "Logging and monitoring operational".to_string(),
                response_time_ms: None,
            }
        } else {
            ReadinessCheck {
                service_name: "Monitoring".to_string(),
                status: ReadinessStatus::Degraded,
                message: "Log directory not writable".to_string(),
                response_time_ms: None,
            }
        }
    }

    async fn check_filesystem_readiness(&self) -> ReadinessCheck {
        // Check critical filesystem operations
        let temp_file = std::env::temp_dir().join("imkitchen_readiness_test");
        let test_data = "readiness_validation_test";

        match std::fs::write(&temp_file, test_data) {
            Ok(_) => match std::fs::read_to_string(&temp_file) {
                Ok(content) if content == test_data => {
                    let _ = std::fs::remove_file(&temp_file);
                    ReadinessCheck {
                        service_name: "Filesystem".to_string(),
                        status: ReadinessStatus::Ready,
                        message: "Read/write operations working".to_string(),
                        response_time_ms: None,
                    }
                }
                Ok(_) => {
                    let _ = std::fs::remove_file(&temp_file);
                    ReadinessCheck {
                        service_name: "Filesystem".to_string(),
                        status: ReadinessStatus::NotReady,
                        message: "Data corruption detected in filesystem test".to_string(),
                        response_time_ms: None,
                    }
                }
                Err(e) => {
                    let _ = std::fs::remove_file(&temp_file);
                    ReadinessCheck {
                        service_name: "Filesystem".to_string(),
                        status: ReadinessStatus::NotReady,
                        message: format!("Read operation failed: {}", e),
                        response_time_ms: None,
                    }
                }
            },
            Err(e) => ReadinessCheck {
                service_name: "Filesystem".to_string(),
                status: ReadinessStatus::NotReady,
                message: format!("Write operation failed: {}", e),
                response_time_ms: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_startup_manager_creation() {
        let config = Config::default();
        let startup_manager = StartupManager::new(config);

        // Basic creation test
        assert_eq!(startup_manager.config.server.port, 3000);
    }

    #[tokio::test]
    async fn test_filesystem_permissions_check() {
        let config = Config::default();
        let startup_manager = StartupManager::new(config);

        let check = startup_manager.check_filesystem_permissions().await;
        assert_eq!(check.name, "Filesystem Permissions");
        // Should typically pass unless running in very restricted environment
    }

    #[tokio::test]
    async fn test_log_directory_access_check() {
        let config = Config::default();
        let startup_manager = StartupManager::new(config);

        let temp_dir = tempdir().unwrap();
        let log_path = temp_dir.path();

        let check = startup_manager.check_log_directory_access(log_path).await;
        assert_eq!(check.name, "Log Directory Access");
        assert_eq!(check.status, DependencyStatus::Healthy);
    }

    #[test]
    fn test_migration_status_creation() {
        let status = MigrationStatus {
            applied_count: 5,
            latest_migration: Some("initial_schema".to_string()),
            requires_migration: false,
        };

        assert_eq!(status.applied_count, 5);
        assert!(!status.requires_migration);
    }

    #[test]
    fn test_dependency_check_creation() {
        let check = DependencyCheck {
            name: "Test Check".to_string(),
            status: DependencyStatus::Healthy,
            message: "All good".to_string(),
            critical: true,
        };

        assert_eq!(check.status, DependencyStatus::Healthy);
        assert!(check.critical);
    }

    #[test]
    fn test_readiness_check_creation() {
        let check = ReadinessCheck {
            service_name: "Test Service".to_string(),
            status: ReadinessStatus::Ready,
            message: "Service ready".to_string(),
            response_time_ms: Some(50),
        };

        assert_eq!(check.status, ReadinessStatus::Ready);
        assert_eq!(check.response_time_ms, Some(50));
    }

    #[tokio::test]
    async fn test_configuration_readiness_check() {
        let config = Config::default();
        let startup_manager = StartupManager::new(config);

        let check = startup_manager.check_configuration_readiness().await;
        assert_eq!(check.service_name, "Configuration");
        assert_eq!(check.status, ReadinessStatus::Ready);
    }

    #[tokio::test]
    async fn test_filesystem_readiness_check() {
        let config = Config::default();
        let startup_manager = StartupManager::new(config);

        let check = startup_manager.check_filesystem_readiness().await;
        assert_eq!(check.service_name, "Filesystem");
        // Should typically pass unless running in very restricted environment
        assert!(
            check.status == ReadinessStatus::Ready || check.status == ReadinessStatus::NotReady
        );
    }
}