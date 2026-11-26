use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use imkitchen_api::auth::Auth;
use imkitchen_api::config::{Config, MonitoringConfig};
use imkitchen_api::mail::Smtp;
use imkitchen_api::recipes::Web;
use imkitchen_api::server::{self, AppState};
use imkitchen_api::{cluster, db, sidecars, subscriptions};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// How often the node looks at what it keeps for the nodes that are away.
const CLUSTER_WATCH: Duration = Duration::from_secs(30);

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
    /// Create the store if needed and apply its pending migrations
    Migrate,
    /// Print the hash of the password read from standard input, for auth.root.password_hash
    HashPassword,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Serve {
        host: None,
        port: None,
    });
    // Needs no configuration: its output goes into one.
    if matches!(command, Commands::HashPassword) {
        return hash_password().await;
    }
    let config = Config::load(cli.config.as_deref())?;

    init_tracing(&config.monitoring)?;

    match command {
        Commands::Serve { host, port } => serve(config, host, port).await,
        Commands::Migrate => migrate(config).await,
        Commands::HashPassword => unreachable!("handled above"),
    }
}

async fn hash_password() -> Result<()> {
    let mut password = String::new();
    std::io::stdin()
        .read_line(&mut password)
        .context("cannot read the password from standard input")?;
    let password = password.trim_end_matches(['\r', '\n']);
    println!("{}", timada_identity::hash_password(password).await?);
    Ok(())
}

async fn migrate(config: Config) -> Result<()> {
    // Closed again right away.
    drop(db::migrate(&config.database.path).await?);
    Ok(())
}

async fn serve(config: Config, host: Option<String>, port: Option<u16>) -> Result<()> {
    let host = host.unwrap_or_else(|| config.server.host.clone());
    let port = port.unwrap_or(config.server.port);
    let auth = Arc::new(Auth::new(&config.auth)?);
    let mailer = Arc::new(Smtp::new(&config.mail)?);
    let pages = Arc::new(Web::new()?);
    let store = db::open(&config.database.path).await?;
    let db = db::Db::from(store.clone());
    let (cluster, executor) = cluster::open(config.cluster, &config.database.path, store).await?;
    let node = format!("api-{}", cluster.health().id.0);
    let subscriptions = subscriptions::start(&executor, &db, &auth, mailer, &node).await?;
    // Warns when this node nears the writes it may keep for a node that is
    // away, and about each node it stopped waiting for.
    let watch = cluster.watch(CLUSTER_WATCH);
    let sidecars = match &config.sidecars {
        Some(config) => {
            let listener = TcpListener::bind(config.listen)
                .await
                .with_context(|| format!("failed to bind sidecars.listen {}", config.listen))?;
            Some(sidecars::serve(listener, config, &executor)?)
        }
        None => None,
    };
    let listener = TcpListener::bind((host.as_str(), port))
        .await
        .with_context(|| format!("failed to bind {host}:{port}"))?;

    tracing::info!("listening on http://{}", listener.local_addr()?);

    let served = server::serve(
        listener,
        AppState {
            cluster: cluster.clone(),
            executor,
            db,
            auth,
            sign_ins: Arc::default(),
            pages,
        },
        config.server.shutdown_timeout(),
        server::shutdown_signal(),
    )
    .await;

    if let Some(sidecars) = sidecars {
        sidecars.shutdown().await;
    }
    subscriptions::shutdown(subscriptions).await;
    watch.abort();
    cluster.shutdown().await;
    served
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
