use imkitchen_shared::{AppConfig, AppError};
use imkitchen_web::start_server;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    info!("Starting imkitchen application with hot reload");

    let config = load_config().await?;

    match start_server(config).await {
        Ok(()) => {
            info!("Server shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("Server failed: {}", e);
            Err(e)
        }
    }
}

async fn load_config() -> Result<AppConfig, AppError> {
    use config::{Config, Environment, File};
    use std::env;

    let environment = env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    info!("Loading configuration for environment: {}", environment);

    let config_builder = Config::builder()
        .add_source(File::with_name(&format!("config/{}", environment)).required(false))
        .add_source(File::with_name("config/local").required(false))
        .add_source(Environment::with_prefix("IMKITCHEN").separator("_"));

    let settings = match config_builder.build() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(AppError::Config(format!("Configuration error: {}", e)));
        }
    };

    let app_config: AppConfig = match settings.try_deserialize() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to deserialize configuration: {}", e);
            return Err(AppError::Config(format!(
                "Configuration deserialization error: {}",
                e
            )));
        }
    };

    info!(
        "Loaded configuration: server={}:{}, database={}",
        app_config.server.host, app_config.server.port, app_config.database.url
    );

    Ok(app_config)
}
