use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use imkitchen_www::api::Api;
use imkitchen_www::config::{Config, MonitoringConfig};
use imkitchen_www::events::Events;
use imkitchen_www::{app, db, subscriptions};
use tokio::net::TcpListener;
use topcoat::asset::{AssetBundle, RouterBuilderAssetExt};
use topcoat::router::RouterService;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// How long www waits for the api before it starts without it.
const API_PATIENCE: Duration = Duration::from_secs(5);

/// imkitchen www
#[derive(Parser)]
#[command(name = "imkitchen-www", about = "imkitchen www", long_about = None)]
struct Cli {
    /// Configuration file. Omitted: built-in defaults plus IMKITCHEN_WWW__* environment variables.
    #[arg(long, global = true, env = "IMKITCHEN_WWW_CONFIG")]
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
    /// Create the store if needed and apply its pending migrations
    Migrate,
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
        Commands::Migrate => migrate(config).await,
    }
}

async fn migrate(config: Config) -> Result<()> {
    // Closed again right away.
    drop(db::migrate(&config.database.path).await?);
    Ok(())
}

async fn serve(config: Config, host: Option<String>, port: Option<u16>) -> Result<()> {
    let host = host.unwrap_or_else(|| config.server.host.clone());
    let port = port.unwrap_or(config.server.port);
    // Built by `topcoat dev` after each build, by `topcoat asset bundle`
    // for a release: the stylesheet, next to the executable.
    let assets = AssetBundle::load().context(
        "cannot load the assets: bundle them with `topcoat asset bundle -p imkitchen-www`",
    )?;
    let store = db::open(&config.database.path).await?;
    let db = db::Db::from(store.clone());
    let events = Events::connect(&config.api, store)?;
    // www serves what its store holds whether the api answers or not; its
    // subscriptions go on once it does.
    if !events.reached(API_PATIENCE).await? {
        tracing::warn!(
            api = %config.api.events,
            "the api does not answer yet: serving what the store holds until it does"
        );
    }
    let subscriptions = subscriptions::start(events.executor(), &db).await?;
    let site = app::Site {
        url: config.server.base_url()?,
        app: config.app.clone(),
        contact: config.contact.clone(),
    };
    let api = Api::new(&config.api)?;
    let listener = TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("failed to bind {host}:{port}"))?;

    tracing::info!("listening on http://{}", listener.local_addr()?);

    let service = RouterService::new(app::router(db, events, api, site).assets(assets).build())
        .shutdown_timeout(config.server.shutdown_timeout());
    // Stops on Ctrl+C or SIGTERM, the requests in flight given
    // `server.shutdown_timeout_secs` to finish.
    let served = topcoat::serve(listener, service).await;
    tracing::info!("shutting down");

    subscriptions::shutdown(subscriptions).await;
    served.context("the server failed")
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
