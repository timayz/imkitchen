use anyhow::Result;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::routes::AppState;

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
    let write_pool = imkitchen::create_write_pool(&config.database.url).await?;

    // Read pool: Multiple connections for read-only queries
    // Use CPU cores as a reasonable default for max connections
    let read_pool_size = config.database.max_connections;
    let read_pool = imkitchen::create_read_pool(&config.database.url, read_pool_size).await?;

    let evento_executor: evento::Sqlite = write_pool.clone().into();
    let user_command = imkitchen_user::Command(evento_executor.clone(), read_pool.clone());
    let user_subscription_command =
        imkitchen_user::subscription::Command(evento_executor.clone(), read_pool.clone());
    let user_meal_preference_command =
        imkitchen_user::meal_preferences::Command(evento_executor.clone(), read_pool.clone());
    let user_query = imkitchen_user::Query(read_pool.clone());
    let contact_command = imkitchen_contact::Command(evento_executor.clone(), read_pool.clone());
    let contact_query = imkitchen_contact::Query(read_pool.clone());
    let recipe_command = imkitchen_recipe::Command(evento_executor.clone(), read_pool.clone());
    let recipe_query = imkitchen_recipe::Query(read_pool.clone());
    let mealplan_command = imkitchen_mealplan::Command(evento_executor.clone(), read_pool.clone());
    let mealplan_query = imkitchen_mealplan::Query(read_pool.clone());

    // Start background notification worker
    tracing::info!("Starting evento subscriptions...");

    let sub_user_command = imkitchen_user::subscribe_command()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_user_list = imkitchen_user::subscribe_list()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_user_stat = imkitchen_user::subscribe_stat()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_contact_list = imkitchen_contact::subscribe_list()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_contact_stat = imkitchen_contact::subscribe_stat()
        .data(write_pool.clone())
        .data(contact_query.clone())
        .run(&evento_executor)
        .await?;

    let sub_recipe_list = imkitchen_recipe::subscribe_list()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_recipe_user_stat = imkitchen_recipe::subscribe_user_stat()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_mealplan_command = imkitchen_mealplan::subscribe_command()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_mealplan_week = imkitchen_mealplan::subscribe_week()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    let sub_mealplan_slot = imkitchen_mealplan::subscribe_slot()
        .data(write_pool.clone())
        .run(&evento_executor)
        .await?;

    // let mut sched_mealplan = imkitchen_mealplan::scheduler(&evento_executor, &read_pool).await?;
    // sched_mealplan.start().await?;

    let state = AppState {
        config,
        user_command,
        user_subscription_command,
        user_meal_preference_command,
        user_query,
        contact_command,
        contact_query,
        recipe_command,
        recipe_query,
        mealplan_command,
        mealplan_query,
        pool: read_pool.clone(),
    };

    // Build router with health checks using read pool state
    let app = crate::routes::router(state)
        // Health check endpoints (no auth required)
        // Add cache control middleware (no-cache for HTML, cache for static files)
        .layer(axum::middleware::from_fn(
            crate::middleware::cache_control_middleware,
        ))
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
    let results = futures::future::join_all(vec![
        sub_user_command.shutdown_and_wait(),
        sub_user_stat.shutdown_and_wait(),
        sub_user_list.shutdown_and_wait(),
        sub_contact_list.shutdown_and_wait(),
        sub_contact_stat.shutdown_and_wait(),
        sub_recipe_list.shutdown_and_wait(),
        sub_recipe_user_stat.shutdown_and_wait(),
        sub_mealplan_command.shutdown_and_wait(),
        sub_mealplan_week.shutdown_and_wait(),
        sub_mealplan_slot.shutdown_and_wait(),
    ])
    .await;

    for result in results {
        if let Err(e) = result {
            tracing::error!("{e}");
        }
    }

    // sched_mealplan.shutdown().await?;

    tracing::info!("All projections shut down successfully");

    // Close database pools
    tracing::info!("Closing database pools...");
    read_pool.close().await;
    write_pool.close().await;
    tracing::info!("Database pools closed");

    tracing::info!("Graceful shutdown complete");

    Ok(())
}
