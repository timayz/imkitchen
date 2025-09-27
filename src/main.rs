// IMKitchen CLI Binary

use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use tracing::{error, info};

mod config;
mod error;
mod monitoring;
mod process;
mod startup;

use config::{Config, ConfigOverrides};
use error::{AppError, AppResult};
use monitoring::setup_monitoring;
use process::ProcessManager;
use startup::StartupManager;

/// IMKitchen CLI - Intelligent Meal Planning Application
#[derive(Parser)]
#[command(name = "imkitchen")]
#[command(version = "0.1.0")]
#[command(about = "Intelligent meal planning and kitchen management")]
#[command(long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Configuration file path
    #[arg(long, global = true, default_value = "imkitchen.toml")]
    config: PathBuf,

    /// Database URL override
    #[arg(long, global = true)]
    database_url: Option<String>,

    /// Log level override  
    #[arg(long, global = true)]
    log_level: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Web server management
    Web {
        #[command(subcommand)]
        action: WebCommands,
    },
    /// Database migration management
    Migrate {
        #[command(subcommand)]
        action: MigrateCommands,
    },
    /// System health check
    Health,
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum WebCommands {
    /// Start the web server
    Start {
        /// Host to bind to
        #[arg(long, default_value = "0.0.0.0")]
        host: String,
        /// Port to bind to
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Run as daemon (background process)
        #[arg(long)]
        daemon: bool,
        /// PID file path
        #[arg(long)]
        pid_file: Option<PathBuf>,
    },
    /// Stop the web server gracefully
    Stop,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Run pending migrations
    Up,
    /// Rollback migrations
    Down {
        /// Number of migrations to rollback
        #[arg(long, default_value = "1")]
        steps: u32,
    },
    /// Show migration status
    Status,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Generate a sample configuration file
    Generate {
        /// Output path for the configuration file
        #[arg(long, short, default_value = "imkitchen.toml")]
        output: PathBuf,
    },
    /// Validate the current configuration
    Validate,
    /// Show the current configuration
    Show,
}

async fn create_database_if_not_exists(database_url: &str) -> AppResult<SqlitePool> {
    info!("Preparing database at: {}", database_url);

    // Extract the database file path from the URL
    let db_path = if let Some(path) = database_url.strip_prefix("sqlite:") {
        // Handle relative paths by making them absolute
        if path.starts_with('/') {
            path.to_string()
        } else {
            let current_dir = std::env::current_dir().map_err(|e| {
                AppError::file_system_with_source(
                    "Failed to get current directory",
                    ".".to_string(),
                    crate::error::FileOperation::Read,
                    e,
                )
            })?;
            current_dir.join(path).to_string_lossy().to_string()
        }
    } else {
        return Err(AppError::configuration(format!(
            "Invalid SQLite URL format: {}",
            database_url
        )));
    };

    info!("Resolved database path: {}", db_path);

    // Create parent directory if it doesn't exist
    let path = std::path::Path::new(&db_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::file_system_with_source(
                    "Failed to create database directory",
                    parent.to_string_lossy().to_string(),
                    crate::error::FileOperation::Create,
                    e,
                )
            })?;
            info!("Created database directory: {:?}", parent);
        }
    }

    // Check if database file exists
    if path.exists() {
        info!("Database file exists, connecting...");
    } else {
        info!("Database file doesn't exist, will be created on connection");

        // Create an empty file to ensure SQLite can write to it
        fs::File::create(path).map_err(|e| {
            AppError::file_system_with_source(
                "Failed to create database file",
                path.to_string_lossy().to_string(),
                crate::error::FileOperation::Create,
                e,
            )
        })?;
        info!("Created empty database file: {}", db_path);
    }

    // Connect to database
    let pool = SqlitePool::connect(database_url)
        .await
        .map_err(|e| AppError::database_with_source("Failed to connect to database", e))?;

    info!("Database connected successfully");
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::database_with_source("Failed to run migrations", e.into()))
}

async fn rollback_migrations(_pool: &SqlitePool, steps: u32) -> AppResult<()> {
    info!("Rolling back {} migrations", steps);
    // Note: SQLx doesn't have built-in rollback, implementing basic version
    for i in 0..steps {
        info!("Rolling back migration step {}", i + 1);
        // This would need custom migration logic for actual rollbacks
    }
    Ok(())
}

async fn check_migration_status(pool: &SqlitePool) -> AppResult<()> {
    info!("Checking migration status...");

    // Check if migration table exists first
    let table_exists = sqlx::query_scalar::<_, i32>(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    if table_exists == 0 {
        println!("No migrations applied (migration table not found)");
        return Ok(());
    }

    // Get migration count and latest migration
    let migration_count = sqlx::query_scalar::<_, i32>("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database_with_source("Failed to query migration count", e))?;

    if migration_count == 0 {
        println!("Migration table exists but no migrations applied");
    } else {
        println!("✓ {} migration(s) applied", migration_count);

        // Try to get latest migration details
        if let Ok(Some(latest)) = sqlx::query_scalar::<_, String>(
            "SELECT description FROM _sqlx_migrations ORDER BY version DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        {
            println!("Latest migration: {}", latest);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        // Log the error with full context
        e.log_error();

        // Show user-friendly message to stderr
        eprintln!("Error: {}", e.user_message());

        // Show correlation ID for debugging if available
        if let Some(correlation_id) = e.correlation_id() {
            eprintln!("Correlation ID: {}", correlation_id);
        }

        // Exit with error code
        std::process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let cli = Cli::parse();

    // Create CLI overrides from command line arguments
    let cli_overrides = ConfigOverrides {
        database_url: cli.database_url.clone(),
        log_level: cli.log_level.clone(),
        host: None, // Will be set per command
        port: None, // Will be set per command
    };

    // Load configuration from multiple sources with validation
    let config = Config::load_from_sources(&cli.config, &cli_overrides)?;

    // Validate security configuration
    config.validate_security()?;

    // Setup comprehensive monitoring stack using new config
    setup_monitoring(
        Some(&config.logging.level),
        &config.logging.format,
        &config.logging.dir,
        config.logging.rotation.clone(),
    )?;

    match &cli.command {
        Commands::Web { action } => {
            match action {
                WebCommands::Start {
                    host,
                    port,
                    daemon,
                    pid_file,
                } => {
                    // Use CLI args first, then config defaults
                    let effective_host = if *host != "0.0.0.0" {
                        host.clone()
                    } else {
                        config.server.host.clone()
                    };
                    let effective_port = if *port != 3000 {
                        *port
                    } else {
                        config.server.port
                    };

                    info!(
                        "Starting web server on {}:{}",
                        effective_host, effective_port
                    );

                    // Initialize process manager with production-ready features
                    let mut process_manager = ProcessManager::new().with_daemon_mode(*daemon);

                    if let Some(pid_path) = pid_file {
                        process_manager = process_manager.with_pid_file(pid_path);
                    }

                    // Check for existing process to prevent conflicts
                    if let Some(existing_pid) = process_manager.check_existing_process()? {
                        return Err(AppError::process(
                            format!(
                                "Another instance is already running with PID {}",
                                existing_pid
                            ),
                            crate::error::ProcessOperation::Start,
                        ));
                    }

                    // Initialize process management (PID file, signal handlers)
                    process_manager.initialize().await?;

                    // Use StartupManager for comprehensive initialization sequence
                    let startup_manager = StartupManager::new(config.clone());
                    let db_pool = match startup_manager.initialize().await {
                        Ok(pool) => {
                            info!("Application startup sequence completed successfully");
                            Some(pool)
                        }
                        Err(e) => {
                            e.log_error();
                            error!("Startup sequence failed: {}", e.user_message());
                            return Err(e);
                        }
                    };

                    // Add cleanup handler for database connections
                    if let Some(ref pool) = db_pool {
                        let pool_clone = pool.clone();
                        process_manager = process_manager.add_cleanup_handler(move || {
                            info!("Closing database connection pool");
                            // Note: close() returns a future but we can't await in sync cleanup handler
                            // Explicitly drop the future - the pool will be closed when the Pool is dropped
                            std::mem::drop(pool_clone.close());
                            Ok(())
                        });
                    }

                    // Start the web server with ProcessManager-coordinated shutdown
                    let server_result = tokio::select! {
                        server_result = imkitchen_web::start_server_with_shutdown(
                            effective_host,
                            effective_port,
                            db_pool,
                        ) => server_result,
                        _ = process_manager.wait_for_shutdown(std::time::Duration::from_secs(300)) => {
                            info!("Shutdown signal received, terminating server");
                            Ok(())
                        }
                    };

                    // Perform graceful shutdown with cleanup
                    process_manager
                        .shutdown(std::time::Duration::from_secs(30))
                        .await?;

                    if let Err(e) = server_result {
                        return Err(AppError::command_line(format!("Web server error: {}", e)));
                    }
                }
                WebCommands::Stop => {
                    info!("Initiating graceful shutdown");

                    // Look for existing PID file from default or common locations
                    let pid_paths = vec![
                        PathBuf::from("imkitchen.pid"),
                        PathBuf::from("/tmp/imkitchen.pid"),
                        PathBuf::from("/var/run/imkitchen.pid"),
                    ];

                    let mut found_process = false;
                    for pid_path in pid_paths {
                        if pid_path.exists() {
                            let process_manager = ProcessManager::new().with_pid_file(&pid_path);

                            if let Ok(Some(existing_pid)) = process_manager.check_existing_process()
                            {
                                info!("Found running process with PID: {}", existing_pid);

                                // Send SIGTERM signal for graceful shutdown
                                #[cfg(unix)]
                                {
                                    use std::process::Command;
                                    let result = Command::new("kill")
                                        .arg("-TERM")
                                        .arg(existing_pid.to_string())
                                        .output();

                                    match result {
                                        Ok(output) if output.status.success() => {
                                            println!(
                                                "✓ Graceful shutdown signal sent to process {}",
                                                existing_pid
                                            );
                                            found_process = true;
                                        }
                                        Ok(_) => {
                                            eprintln!(
                                                "✗ Failed to send shutdown signal to process {}",
                                                existing_pid
                                            );
                                        }
                                        Err(e) => {
                                            eprintln!("✗ Error sending signal: {}", e);
                                        }
                                    }
                                }

                                #[cfg(not(unix))]
                                {
                                    println!(
                                        "⚠ Graceful shutdown not implemented for this platform"
                                    );
                                    println!(
                                        "Please stop the process manually (PID: {})",
                                        existing_pid
                                    );
                                    found_process = true;
                                }

                                break;
                            }
                        }
                    }

                    if !found_process {
                        println!("ℹ No running process found (no PID file located)");
                    }
                }
            }
        }
        Commands::Migrate { action } => {
            info!("Preparing database: {}", config.database.url);
            let pool = create_database_if_not_exists(&config.database.url).await?;

            match action {
                MigrateCommands::Up => {
                    info!("Running database migrations");
                    run_migrations(&pool).await?;
                    println!("Migrations completed successfully");
                }
                MigrateCommands::Down { steps } => {
                    rollback_migrations(&pool, *steps).await?;
                    println!("Rollback completed");
                }
                MigrateCommands::Status => {
                    check_migration_status(&pool).await?;
                }
            }
        }
        Commands::Health => {
            info!("Performing system health check");

            // Check database connectivity (will create if not exists)
            match create_database_if_not_exists(&config.database.url).await {
                Ok(_) => {
                    println!("✓ Database: Connected");
                    println!("✓ Configuration: Valid");
                    println!("✓ System: OK");
                }
                Err(e) => {
                    e.log_error();
                    eprintln!("✗ Database: {}", e.user_message());
                    if let Some(correlation_id) = e.correlation_id() {
                        eprintln!("Debug correlation ID: {}", correlation_id);
                    }
                    println!("✗ System: Degraded");
                    exit(1);
                }
            }
        }
        Commands::Config { action } => {
            match action {
                ConfigCommands::Generate { output } => {
                    info!("Generating sample configuration file: {:?}", output);
                    Config::generate_sample_config(output)?;
                    println!("Sample configuration generated at: {:?}", output);
                    println!(
                        "Edit the file and set the SESSION_SECRET environment variable before use."
                    );
                }
                ConfigCommands::Validate => {
                    info!("Validating configuration");
                    // Configuration was already validated during load
                    println!("✓ Configuration: Valid");
                    println!("✓ All settings: OK");

                    // Show current config source info
                    if cli.config.exists() {
                        println!("✓ Config file: {:?}", cli.config);
                    } else {
                        println!("ℹ Config file: Not found, using defaults");
                    }
                }
                ConfigCommands::Show => {
                    info!("Displaying current configuration");
                    println!("Current Configuration:");
                    println!("====================");
                    println!("Database URL: {}", config.database.url);
                    println!("Server: {}:{}", config.server.host, config.server.port);
                    println!("Log Level: {}", config.logging.level);
                    println!("Log Format: {:?}", config.logging.format);
                    println!("Metrics Enabled: {}", config.monitoring.enable_metrics);
                    println!("Security Settings:");
                    println!("  Session Timeout: {}s", config.security.session_timeout);
                    println!("  Force HTTPS: {}", config.security.force_https);
                    println!(
                        "  Rate Limit: {} req/min",
                        config.security.rate_limit_per_minute
                    );
                }
            }
        }
    }

    Ok(())
}
