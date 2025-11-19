mod assets;
mod auth;
mod cli;
mod config;
// mod email;
mod language;
mod middleware;
mod routes;
mod template;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

rust_i18n::i18n!("locales", fallback = "en");

/// imkitchen - Intelligent Meal Planning
#[derive(Parser)]
#[command(name = "imkitchen")]
#[command(about = "Intelligent meal planning and cooking optimization", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(long, global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Server host address (overrides config file)
        #[arg(long)]
        host: Option<String>,

        /// Server port (overrides config file)
        #[arg(long)]
        port: Option<u16>,
    },
    /// Run database migrations
    Migrate,
    /// Drop database if exists and recreate with migrations
    Reset,
    /// Set user role
    UserRole {
        #[arg(long)]
        email: String,

        #[arg(long)]
        role: cli::Role,
    },
}

#[derive(Subcommand)]
enum User {
    MadeAdmin,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = crate::config::Config::load(cli.config.clone())?;

    let env_filter = EnvFilter::new(&config.monitoring.log_level);
    if config.monitoring.log_json {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(config.monitoring.log_target)
                    .with_line_number(config.monitoring.log_line_number)
                    .with_filter(env_filter),
            )
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(config.monitoring.log_target)
                    .with_line_number(config.monitoring.log_line_number)
                    .with_filter(env_filter),
            )
            .try_init()?;
    }

    match cli.command {
        Commands::Serve { host, port } => crate::cli::serve(config, host, port).await,
        Commands::Migrate => crate::cli::migrate(config).await,
        Commands::Reset => crate::cli::reset(config).await,
        Commands::UserRole { email, role } => crate::cli::set_role(config, email, role).await,
    }
}
