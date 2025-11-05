use std::time::Duration;

use anyhow::Result;
use axum::{Router, routing::get};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::Config,
    pub user_command: imkitchen_user::Command<evento::Sqlite>,
}

pub async fn serve(
    config: crate::config::Config,
    host_override: Option<String>,
    port_override: Option<u16>,
) -> Result<()> {
    tracing::info!("Starting imkitchen server...");

    // Use CLI overrides if provided, otherwise use config
    let host = host_override.unwrap_or(config.server.host.to_owned());
    let port = port_override.unwrap_or(config.server.port);

    // Set up database connection pools with optimized PRAGMAs
    // Write pool: 1 connection for evento and all write operations
    let write_pool = crate::db::create_write_pool(&config.database.url).await?;

    // Read pool: Multiple connections for read-only queries
    // Use CPU cores as a reasonable default for max connections
    let read_pool_size = config.database.max_connections;
    let read_pool = crate::db::create_read_pool(&config.database.url, read_pool_size).await?;

    let evento_executor: evento::Sqlite = write_pool.clone().into();
    let user_command = imkitchen_user::Command(evento_executor.clone());

    // Start background notification worker
    tracing::info!("Starting evento subscriptions...");

    let sub_user_command = imkitchen_user::subscribe_command()
        .data(write_pool.clone())
        .delay(Duration::from_secs(10))
        .run(&evento_executor)
        .await?;

    let state = AppState {
        config,
        user_command,
    };

    // Build router with health checks using read pool state
    let app = Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(crate::routes::health::health))
        .route("/ready", get(crate::routes::health::ready))
        .with_state(read_pool.clone())
        .route("/", get(crate::routes::index::page))
        .route("/help", get(crate::routes::help::page))
        .route("/terms", get(crate::routes::terms::page))
        .route("/policy", get(crate::routes::policy::page))
        .route(
            "/register",
            get(crate::routes::register::page).post(crate::routes::register::action),
        )
        .route(
            "/register/status/{id}",
            get(crate::routes::register::status),
        )
        .route("/login", get(crate::routes::login::page))
        .fallback(crate::routes::fallback)
        .route("/sw.js", get(crate::routes::service_worker::sw))
        .nest_service("/static", crate::assets::AssetsService::new())
        .with_state(state)
        // Add cache control middleware (no-cache for HTML, cache for static files)
        .layer(axum::middleware::from_fn(
            crate::middleware::cache_control_middleware,
        ))
        // LiveReload layer for development (debug builds only) - must be before minification
        .layer({
            #[cfg(debug_assertions)]
            {
                tower_livereload::LiveReloadLayer::new()
            }
            #[cfg(not(debug_assertions))]
            {
                use axum::{body::Body, extract::Request, response::Response};
                axum::middleware::from_fn(|req: Request, next: axum::middleware::Next| async move {
                    next.run(req).await
                })
            }
        })
        // Minify HTML responses before compression
        .layer(axum::middleware::map_response(
            crate::middleware::minify_html_middleware,
        ))
        // Enable Brotli and Gzip compression for all text assets (Story 5.9)
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    // Set up graceful shutdown signal handler
    let shutdown_signal = async {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!("Received Ctrl+C signal");
            },
            _ = terminate => {
                tracing::info!("Received SIGTERM signal");
            },
        }

        tracing::info!("Starting graceful shutdown...");
    };

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    tracing::info!("Shutting down evento projections...");

    // Shutdown all projection subscriptions
    let results = futures::future::join_all(vec![sub_user_command.shutdown_and_wait()]).await;
    for result in results {
        if let Err(e) = result {
            tracing::error!("{e}");
        }
    }

    tracing::info!("All projections shut down successfully");

    // Close database pools
    tracing::info!("Closing database pools...");
    read_pool.close().await;
    write_pool.close().await;
    tracing::info!("Database pools closed");

    tracing::info!("Graceful shutdown complete");

    Ok(())
}
