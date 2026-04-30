use anyhow::Result;
use axum::response::IntoResponse;
use axum::routing::get;
use imkitchen_notification::EmailService;
use imkitchen_web_shared::AppState;
use imkitchen_web_shared::template::{NotFoundTemplate, Template};
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

async fn fallback(template: Template) -> impl IntoResponse {
    template.render(NotFoundTemplate)
}

pub async fn serve(
    config: imkitchen_web_shared::config::Config,
    host_override: Option<String>,
    port_override: Option<u16>,
) -> Result<()> {
    tracing::info!("Starting imkitchen server...");

    let host = host_override.unwrap_or(config.server.host.to_owned());
    let port = port_override.unwrap_or(config.server.port);

    let email_service = EmailService::new(&config.email)?;

    let write_pool = imkitchen::create_write_pool(&config.database.url).await?;
    let read_pool_size = config.database.max_connections;
    let read_pool = imkitchen::create_read_pool(&config.database.url, read_pool_size).await?;

    let executor: evento::sql::RwSqlite = (
        evento::Sqlite::from(read_pool.clone()),
        evento::Sqlite::from(write_pool.clone()),
    )
        .into();

    tracing::info!("Starting evento subscriptions...");

    let sub_notification_contact = imkitchen_notification::contact::subscription()
        .data(email_service.clone())
        .start(&executor)
        .await?;

    let sub_notification_user = imkitchen_notification::user::subscription()
        .data(email_service)
        .start(&executor)
        .await?;

    let sub_user_query = imkitchen_identity::query_subscription()
        .data((read_pool.clone(), write_pool.clone()))
        .start(&executor)
        .await?;

    let sub_user_shed = imkitchen_billing::shed_subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_user_global_stat = imkitchen_identity::global_stat::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_user_invoice = imkitchen_billing::invoice::subscription()
        .data((read_pool.clone(), write_pool.clone()))
        .data(config.email.clone())
        .start(&executor)
        .await?;

    let sub_contact_query = imkitchen_core::contact::query_subscription()
        .data((read_pool.clone(), write_pool.clone()))
        .start(&executor)
        .await?;

    let sub_contact_global_stat = imkitchen_core::contact::global_stat::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_recipe_command = imkitchen_core::recipe::subscription()
        .data((read_pool.clone(), write_pool.clone()))
        .start(&executor)
        .await?;

    let sub_recipe_query = imkitchen_core::recipe::query::subscription()
        .data((read_pool.clone(), write_pool.clone()))
        .start(&executor)
        .await?;

    let sub_recipe_comment = imkitchen_core::recipe::query::comment::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_recipe_user = imkitchen_core::recipe::query::user::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_recipe_user_fts = imkitchen_core::recipe::query::user_fts::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_recipe_thumbnail = imkitchen_core::recipe::query::thumbnail::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_recipe_user_stat = imkitchen_core::recipe::query::user_stat::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_mealplan_cmd = imkitchen_core::mealplan::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_mealplan_slot = imkitchen_core::mealplan::slot::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_shopping = imkitchen_core::shopping::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let sub_shopping_list = imkitchen_core::shopping::list::subscription()
        .data(write_pool.clone())
        .start(&executor)
        .await?;

    let stripe = stripe::ClientBuilder::new(&config.stripe.secret_key)
        .request_strategy(stripe::RequestStrategy::ExponentialBackoff(4))
        .build()?;

    let mut sched_billing =
        imkitchen_billing::scheduler(&executor, &read_pool, &write_pool, &stripe).await?;
    sched_billing.start().await?;

    let state = imkitchen_core::State {
        executor: executor.clone(),
        read_db: read_pool.clone(),
        write_db: write_pool.clone(),
    };

    let app_state = AppState {
        config,
        stripe,
        identity: imkitchen_identity::Module::new(state.clone()),
        billing: imkitchen_billing::Billing::new(state.clone()),
        core: imkitchen_core::Core::new(state.clone()),
        inner: state,
    };

    let app = axum::Router::new()
        .route("/health", get(imkitchen_web_public::routes::health::health))
        .route(
            "/_test-error",
            get(imkitchen_web_public::routes::health::test_error),
        )
        .route("/ready", get(imkitchen_web_public::routes::health::ready))
        .with_state(app_state.read_db.clone())
        .merge(imkitchen_web_kitchen::routes())
        .merge(imkitchen_web_menu::routes())
        .merge(imkitchen_web_recipe::routes())
        .merge(imkitchen_web_grocery::routes())
        .merge(imkitchen_web_settings::routes())
        .merge(imkitchen_web_public::routes())
        .merge(imkitchen_web_admin::routes())
        .fallback(fallback)
        .nest_service(
            "/static",
            imkitchen_web_shared::assets::AssetsService::new(),
        )
        .with_state(app_state)
        .layer(axum::middleware::from_fn(
            imkitchen_web_shared::middleware::cache_control_middleware,
        ))
        .layer(axum::middleware::map_response(
            imkitchen_web_shared::middleware::minify_html_middleware,
        ))
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

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

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    tracing::info!("Shutting down evento projections...");

    let results = futures::future::join_all(vec![
        sub_notification_contact.shutdown(),
        sub_notification_user.shutdown(),
        sub_user_query.shutdown(),
        sub_user_shed.shutdown(),
        sub_user_global_stat.shutdown(),
        sub_user_invoice.shutdown(),
        sub_contact_query.shutdown(),
        sub_contact_global_stat.shutdown(),
        sub_recipe_command.shutdown(),
        sub_recipe_query.shutdown(),
        sub_recipe_comment.shutdown(),
        sub_recipe_user.shutdown(),
        sub_recipe_user_fts.shutdown(),
        sub_recipe_user_stat.shutdown(),
        sub_recipe_thumbnail.shutdown(),
        sub_mealplan_cmd.shutdown(),
        sub_mealplan_slot.shutdown(),
        sub_shopping.shutdown(),
        sub_shopping_list.shutdown(),
    ])
    .await;

    for result in results {
        if let Err(e) = result {
            tracing::error!("{e}");
        }
    }

    sched_billing.shutdown().await?;

    tracing::info!("All projections shut down successfully");

    tracing::info!("Closing database pools...");
    read_pool.close().await;
    write_pool.close().await;
    tracing::info!("Database pools closed");

    tracing::info!("Graceful shutdown complete");

    Ok(())
}
