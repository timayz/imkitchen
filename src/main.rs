use clap::{Parser, Subcommand};

mod migrate;
mod server;

#[derive(Parser)]
#[command(name = "imkitchen")]
#[command(about = "ImKitchen - Event-driven meal planning application", long_about = None)]
struct Cli {
    /// Path to configuration file (optional)
    #[arg(short, long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Serve {
        /// Port to listen on (overrides config)
        #[arg(short, long)]
        port: Option<u16>,
    },
    /// Run database migrations (creates databases if they don't exist)
    Migrate,
    /// Drop all databases and run migrations
    Reset,
    /// Set admin status for a user by email
    SetAdmin {
        /// User email address
        email: String,
        /// Admin status (true/false)
        is_admin: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = imkitchen::Config::load(cli.config.as_deref())?;

    // Initialize tracing with config
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.logging.level)),
        )
        .init();

    match cli.command {
        Commands::Serve { port } => {
            // Use CLI port if provided, otherwise use config
            let server_port = port.unwrap_or(config.server.port);
            tracing::info!("Starting server on {}:{}", config.server.host, server_port);
            server::serve(&config, server_port).await?;
        }
        Commands::Migrate => {
            tracing::info!("Running database migrations");
            migrate::migrate(&config).await?;
            tracing::info!("Migrations completed successfully");
        }
        Commands::Reset => {
            tracing::info!("Resetting databases");
            migrate::reset(&config).await?;
            tracing::info!("Databases reset successfully");
        }
        Commands::SetAdmin { email, is_admin } => {
            let is_admin_bool = is_admin.parse::<bool>().map_err(|_| {
                anyhow::anyhow!("Invalid value for is_admin. Use 'true' or 'false'")
            })?;
            tracing::info!("Setting admin status for user: {}", email);
            set_admin(&config, &email, is_admin_bool).await?;
            tracing::info!(
                "Admin status set successfully: {} is now {}",
                email,
                if is_admin_bool {
                    "an admin"
                } else {
                    "not an admin"
                }
            );
        }
    }

    Ok(())
}

/// Set admin status for a user by email
async fn set_admin(config: &imkitchen::Config, email: &str, is_admin: bool) -> anyhow::Result<()> {
    use imkitchen::queries::user::get_user_by_email;
    use imkitchen_user::command::{Command, SetAdminStatusInput};
    use imkitchen_user::event::EventMetadata;
    use sqlx::SqlitePool;
    use ulid::Ulid;

    // Connect to databases
    let evento_pool = SqlitePool::connect(&config.database.evento_db).await?;
    let query_pool = SqlitePool::connect(&config.database.queries_db).await?;

    let evento = evento::Sqlite::from(evento_pool);

    // Find user by email
    let user = get_user_by_email(&query_pool, email)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found with email: {}", email))?;

    // Create command
    let command = Command::new(evento.clone());

    // Set admin status
    let metadata = EventMetadata {
        user_id: None, // CLI operation, no user context
        request_id: Ulid::new().to_string(),
    };

    command
        .set_admin_status(
            SetAdminStatusInput {
                user_id: user.id.clone(),
                is_admin,
            },
            metadata,
        )
        .await?;

    // Process events synchronously
    imkitchen::queries::user::subscribe_user_query::<evento::Sqlite>(query_pool.clone())
        .unsafe_oneshot(&evento)
        .await?;

    Ok(())
}
