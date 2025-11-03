//! Web server implementation using Axum

use axum::{routing::get, Router};
use imkitchen::access_control::AccessControlService;
use imkitchen::assets::AssetsService;
use imkitchen::routes::admin::admin_routes;
use imkitchen::routes::auth::{
    get_login, get_profile, get_register, get_register_status, post_login, post_logout,
    post_profile, post_register, AppState,
};
use imkitchen::routes::contact::{get_contact, post_contact};
use imkitchen::Config;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing::info;

/// Start the web server
pub async fn serve(config: &Config, port: u16) -> anyhow::Result<()> {
    info!("Initializing databases...");

    // Initialize database connections
    let evento_pool = SqlitePool::connect(&config.database.evento_db).await?;
    let validation_pool = SqlitePool::connect(&config.database.validation_db).await?;
    let query_pool = SqlitePool::connect(&config.database.queries_db).await?;

    let evento = evento::Sqlite::from(evento_pool);

    info!("Starting event subscriptions...");

    // Start event subscriptions
    imkitchen_user::command::subscribe_user_command::<evento::Sqlite>(validation_pool.clone())
        .run(&evento)
        .await?;

    imkitchen::queries::user::subscribe_user_query::<evento::Sqlite>(query_pool.clone())
        .run(&evento)
        .await?;

    // Create email service
    let email_service = imkitchen::email::EmailService::new(&config.email)?;

    imkitchen::queries::contact::subscribe_contact_query::<evento::Sqlite>(
        query_pool.clone(),
        email_service,
    )
    .run(&evento)
    .await?;

    info!("Creating application state...");

    // Create access control service
    let access_control = AccessControlService::new(config.clone(), query_pool.clone());

    // Create application state
    let state = AppState {
        evento: evento.clone(),
        query_pool,
        jwt_secret: config.auth.jwt_secret.clone(),
        jwt_lifetime_seconds: config.auth.jwt_lifetime_seconds,
        config: config.clone(),
        access_control,
    };

    // Create auth state for middleware
    let auth_state = imkitchen::auth::middleware::AuthState {
        jwt_secret: config.auth.jwt_secret.clone(),
        evento,
    };

    let app = create_router(state, auth_state);

    // Parse host from config
    let host_parts: Vec<u8> = if config.server.host == "0.0.0.0" {
        vec![0, 0, 0, 0]
    } else {
        config
            .server
            .host
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let host = if host_parts.len() == 4 {
        [host_parts[0], host_parts[1], host_parts[2], host_parts[3]]
    } else {
        [0, 0, 0, 0]
    };

    let addr = SocketAddr::from((host, port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the application router
fn create_router(state: AppState, auth_state: imkitchen::auth::middleware::AuthState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        // Auth routes
        .route("/auth/register", get(get_register).post(post_register))
        .route("/auth/register/status/{id}", get(get_register_status))
        .route("/auth/login", get(get_login).post(post_login))
        .route("/auth/logout", axum::routing::post(post_logout))
        .route("/auth/profile", get(get_profile).post(post_profile))
        // Contact routes (public access)
        .route("/contact", get(get_contact).post(post_contact))
        // Admin routes (protected by auth + admin middleware)
        .merge(admin_routes(auth_state))
        .nest_service("/static", AssetsService::new())
        .with_state(state)
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
}

/// Root handler
async fn root() -> &'static str {
    "ImKitchen - Meal Planning Application"
}

/// Health check handler
async fn health() -> &'static str {
    "OK"
}
