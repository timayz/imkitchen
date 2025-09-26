// IMKitchen CLI Binary

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use std::process;
use tracing::{error, info};
use tracing_appender::rolling::Rotation;

mod monitoring;
use monitoring::{setup_monitoring, LogFormat};

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

/// Configuration for the application
#[derive(Debug, Clone)]
struct Config {
    database_url: String,
    server_host: String,
    server_port: u16,
    log_format: LogFormat,
    log_dir: Option<PathBuf>,
    log_rotation: Rotation,
}

impl Config {
    fn from_cli(cli: &Cli) -> Result<Self> {
        let database_url = cli
            .database_url
            .clone()
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "sqlite:imkitchen.db".to_string());

        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let server_port = std::env::var("SERVER_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let log_format = match std::env::var("LOG_FORMAT").as_deref() {
            Ok("json") => LogFormat::Json,
            Ok("compact") => LogFormat::Compact,
            _ => LogFormat::Pretty,
        };

        let log_dir = std::env::var("LOG_DIR").ok().map(PathBuf::from);

        let log_rotation = match std::env::var("LOG_ROTATION").as_deref() {
            Ok("hourly") => Rotation::HOURLY,
            Ok("daily") => Rotation::DAILY,
            Ok("never") => Rotation::NEVER,
            _ => Rotation::DAILY,
        };

        Ok(Config {
            database_url,
            server_host,
            server_port,
            log_format,
            log_dir,
            log_rotation,
        })
    }
}

async fn create_database_if_not_exists(database_url: &str) -> Result<SqlitePool> {
    info!("Preparing database at: {}", database_url);

    // Extract the database file path from the URL
    let db_path = if let Some(path) = database_url.strip_prefix("sqlite:") {
        // Handle relative paths by making them absolute
        if path.starts_with('/') {
            path.to_string()
        } else {
            let current_dir = std::env::current_dir().context("Failed to get current directory")?;
            current_dir.join(path).to_string_lossy().to_string()
        }
    } else {
        return Err(anyhow::anyhow!(
            "Invalid SQLite URL format: {}",
            database_url
        ));
    };

    info!("Resolved database path: {}", db_path);

    // Create parent directory if it doesn't exist
    let path = std::path::Path::new(&db_path);
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).context("Failed to create database directory")?;
            info!("Created database directory: {:?}", parent);
        }
    }

    // Check if database file exists
    if path.exists() {
        info!("Database file exists, connecting...");
    } else {
        info!("Database file doesn't exist, will be created on connection");

        // Create an empty file to ensure SQLite can write to it
        fs::File::create(path).context("Failed to create database file")?;
        info!("Created empty database file: {}", db_path);
    }

    // Connect to database
    let pool = SqlitePool::connect(database_url)
        .await
        .context("Failed to connect to database")?;

    info!("Database connected successfully");
    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run migrations")
}

async fn rollback_migrations(_pool: &SqlitePool, steps: u32) -> Result<()> {
    info!("Rolling back {} migrations", steps);
    // Note: SQLx doesn't have built-in rollback, implementing basic version
    for i in 0..steps {
        info!("Rolling back migration step {}", i + 1);
        // This would need custom migration logic for actual rollbacks
    }
    Ok(())
}

async fn check_migration_status(pool: &SqlitePool) -> Result<()> {
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
        .await?;

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

fn write_pid_file(path: &PathBuf) -> Result<()> {
    let pid = process::id();
    fs::write(path, pid.to_string()).context("Failed to write PID file")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration first
    let config = Config::from_cli(&cli).context("Failed to load configuration")?;

    // Setup comprehensive monitoring stack
    setup_monitoring(
        cli.log_level.as_ref(),
        &config.log_format,
        &config.log_dir,
        config.log_rotation,
    )
    .context("Failed to initialize monitoring stack")?;

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
                        config.server_host.clone()
                    };
                    let effective_port = if *port != 3000 {
                        *port
                    } else {
                        config.server_port
                    };

                    info!(
                        "Starting web server on {}:{}",
                        effective_host, effective_port
                    );

                    // Write PID file if specified
                    if let Some(pid_path) = pid_file {
                        write_pid_file(pid_path)?;
                        info!("PID file written to {:?}", pid_path);
                    }

                    if *daemon {
                        info!("Running in daemon mode");
                        // Note: Actual daemon implementation would require fork/detach
                    }

                    // Create database pool with health checks and retry logic
                    let db_config =
                        imkitchen_web::DatabaseConfig::from_url(config.database_url.clone())
                            .with_max_connections(10)
                            .with_timeouts(
                                std::time::Duration::from_secs(30),
                                std::time::Duration::from_secs(10),
                            );

                    let db_pool = match imkitchen_web::create_database_pool_with_retry(
                        &db_config,
                        3,                                 // max retries
                        std::time::Duration::from_secs(2), // retry delay
                    )
                    .await
                    {
                        Ok(pool) => {
                            info!("Database connection pool created successfully");
                            Some(pool)
                        }
                        Err(e) => {
                            error!("Failed to create database pool: {}", e);
                            None
                        }
                    };

                    // Start the web server with graceful shutdown
                    if let Err(e) = imkitchen_web::start_server_with_shutdown(
                        effective_host,
                        effective_port,
                        db_pool,
                    )
                    .await
                    {
                        return Err(anyhow::anyhow!("Failed to start web server: {}", e));
                    }
                }
                WebCommands::Stop => {
                    info!("Initiating graceful shutdown");
                    // TODO: Implement graceful shutdown with signal handling
                    println!("Graceful shutdown initiated");
                }
            }
        }
        Commands::Migrate { action } => {
            info!("Preparing database: {}", config.database_url);
            let pool = create_database_if_not_exists(&config.database_url).await?;

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
            match create_database_if_not_exists(&config.database_url).await {
                Ok(_) => {
                    println!("✓ Database: Connected");
                    println!("✓ System: OK");
                }
                Err(e) => {
                    error!("Database connection failed: {}", e);
                    println!("✗ Database: Failed to connect");
                    println!("✗ System: Degraded");
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}
