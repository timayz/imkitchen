mod assets;
mod auth;
mod config;
mod db;
mod middleware;
mod migrate;
mod routes;
mod server;
mod template;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

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
        Commands::Serve { host, port } => crate::server::serve(config, host, port).await,
        Commands::Migrate => crate::migrate::migrate(config).await,
        Commands::Reset => crate::migrate::reset(config).await,
    }
}

rust_i18n::i18n!("locales", fallback = "en");

pub(crate) mod filters {
    pub fn t(value: &str, _values: &dyn askama::Values) -> askama::Result<String> {
        // let preferred_language = askama::get_value::<String>(values, "preferred_language")
        //     .expect("Unable to get preferred_language from askama::get_value");

        Ok(rust_i18n::t!(value, locale = "fr" /*locale = preferred_language*/).to_string())
    }

    // pub fn assets(value: &str, values: &dyn askama::Values) -> askama::Result<String> {
    //     let config = askama::get_value::<crate::axum_extra::TemplateConfig>(values, "config")
    //         .expect("Unable to get config from askama::get_value");
    //
    //     Ok(format!("{}/{value}", config.assets_base_url))
    // }
}
