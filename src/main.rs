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
    }

    Ok(())
}
