use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use imkitchen_api::config::{Config, MonitoringConfig};
use imkitchen_api::server;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// imkitchen api
#[derive(Parser)]
#[command(name = "imkitchen-api", about = "imkitchen api", long_about = None)]
struct Cli {
    /// Configuration file. Omitted: built-in defaults plus IMKITCHEN_API__* environment variables.
    #[arg(long, global = true, env = "IMKITCHEN_API_CONFIG")]
    config: Option<String>,

    /// Defaults to `serve`.
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the HTTP server
    Serve {
        /// Bind address (overrides server.host)
        #[arg(long)]
        host: Option<String>,

        /// Port (overrides server.port)
        #[arg(long)]
        port: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;

    init_tracing(&config.monitoring)?;

    match cli.command.unwrap_or(Commands::Serve {
        host: None,
        port: None,
    }) {
        Commands::Serve { host, port } => serve(config, host, port).await,
    }
}

async fn serve(config: Config, host: Option<String>, port: Option<u16>) -> Result<()> {
    let host = host.unwrap_or_else(|| config.server.host.clone());
    let port = port.unwrap_or(config.server.port);
    let listener = TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("failed to bind {host}:{port}"))?;

    tracing::info!("listening on http://{}", listener.local_addr()?);

    server::serve(
        listener,
        config.server.shutdown_timeout(),
        server::shutdown_signal(),
    )
    .await
}

fn init_tracing(monitoring: &MonitoringConfig) -> Result<()> {
    let env_filter = EnvFilter::try_new(&monitoring.log_level)
        .context("monitoring.log_level is not a valid filter")?;
    if monitoring.log_json {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_target(monitoring.log_target)
                    .with_line_number(monitoring.log_line_number)
                    .with_filter(env_filter),
            )
            .try_init()?;
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(monitoring.log_target)
                    .with_line_number(monitoring.log_line_number)
                    .with_filter(env_filter),
            )
            .try_init()?;
    }
    Ok(())
}
