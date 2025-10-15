use anyhow::Result;
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use evento::prelude::*;
use imkitchen::middleware::auth_middleware;
use imkitchen::routes::{
    get_collections, get_ingredient_row, get_instruction_row, get_login, get_onboarding,
    get_onboarding_skip, get_password_reset, get_password_reset_complete, get_profile,
    get_recipe_detail, get_recipe_edit_form, get_recipe_form, get_recipe_list, get_register,
    get_subscription, get_subscription_success, health, post_add_recipe_to_collection,
    post_create_collection, post_create_recipe, post_delete_collection, post_delete_recipe,
    post_login, post_logout, post_onboarding_step_1, post_onboarding_step_2,
    post_onboarding_step_3, post_onboarding_step_4, post_password_reset,
    post_password_reset_complete, post_profile, post_register, post_remove_recipe_from_collection,
    post_stripe_webhook, post_subscription_upgrade, post_update_collection, post_update_recipe,
    post_update_recipe_tags, ready, AppState, AssetsService,
};
use recipe::{collection_projection, recipe_projection};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use tower_http::trace::TraceLayer;
use user::user_projection;

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
    let config = imkitchen::config::Config::load(cli.config.clone())?;
    config.validate().map_err(|e| anyhow::anyhow!(e))?;

    // Initialize observability (tracing + logging)
    imkitchen::observability::init_observability(
        "imkitchen",
        env!("CARGO_PKG_VERSION"),
        &config.observability.otel_endpoint,
        &config.observability.log_level,
    )?;

    let result = match cli.command {
        Commands::Serve { host, port } => serve_command(config, host, port).await,
        Commands::Migrate => migrate_command(config).await,
        Commands::Reset => reset_command(config).await,
    };

    // Graceful shutdown of observability
    imkitchen::observability::shutdown_observability();

    result
}

#[tracing::instrument(skip(config))]
async fn serve_command(
    config: imkitchen::config::Config,
    host_override: Option<String>,
    port_override: Option<u16>,
) -> Result<()> {
    tracing::info!("Starting imkitchen server...");

    // Use CLI overrides if provided, otherwise use config
    let host = host_override.unwrap_or(config.server.host);
    let port = port_override.unwrap_or(config.server.port);

    // Set up database connection pool
    let db_pool = SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    // Create evento executor
    let evento_executor: evento::Sqlite = db_pool.clone().into();

    // Set up evento subscription for read model projections
    user_projection(db_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'user-read-model' started");

    recipe_projection(db_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'recipe-read-model' started");

    collection_projection(db_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'collection-read-model' started");

    // Create app state
    let email_config = imkitchen::email::EmailConfig {
        smtp_host: config.email.smtp_host,
        smtp_port: config.email.smtp_port,
        smtp_username: config.email.smtp_username,
        smtp_password: config.email.smtp_password,
        from_email: config.email.from_email,
        from_name: config.email.from_name,
    };

    let state = AppState {
        db_pool,
        evento_executor,
        jwt_secret: config.jwt.secret,
        email_config,
        base_url: config.email.base_url,
        stripe_secret_key: config.stripe.secret_key,
        stripe_webhook_secret: config.stripe.webhook_secret,
        stripe_price_id: config.stripe.price_id,
    };

    // Build protected routes with auth middleware
    let protected_routes = Router::new()
        .route("/logout", post(post_logout))
        .route("/onboarding", get(get_onboarding))
        .route("/onboarding/step/1", post(post_onboarding_step_1))
        .route("/onboarding/step/2", post(post_onboarding_step_2))
        .route("/onboarding/step/3", post(post_onboarding_step_3))
        .route("/onboarding/step/4", post(post_onboarding_step_4))
        .route("/onboarding/skip", get(get_onboarding_skip))
        .route("/profile", get(get_profile).post(post_profile))
        .route("/subscription", get(get_subscription))
        .route("/subscription/upgrade", post(post_subscription_upgrade))
        .route("/subscription/success", get(get_subscription_success))
        .route("/dashboard", get(dashboard_handler))
        // Recipe routes
        .route("/recipes", get(get_recipe_list).post(post_create_recipe))
        .route("/recipes/new", get(get_recipe_form))
        .route("/recipes/{id}", get(get_recipe_detail))
        .route("/recipes/{id}/edit", get(get_recipe_edit_form))
        .route("/recipes/{id}", post(post_update_recipe))
        .route("/recipes/{id}/delete", post(post_delete_recipe))
        .route("/recipes/{id}/tags", post(post_update_recipe_tags))
        .route("/recipes/ingredient-row", get(get_ingredient_row))
        .route("/recipes/instruction-row", get(get_instruction_row))
        // Collection routes
        .route(
            "/collections",
            get(get_collections).post(post_create_collection),
        )
        .route("/collections/{id}/update", post(post_update_collection))
        .route("/collections/{id}/delete", post(post_delete_collection))
        .route(
            "/collections/{collection_id}/recipes/{recipe_id}/add",
            post(post_add_recipe_to_collection),
        )
        .route(
            "/collections/{collection_id}/recipes/{recipe_id}/remove",
            post(post_remove_recipe_from_collection),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Build router with health checks using db_pool state
    let app = Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(state.db_pool.clone())
        .merge(
            Router::new()
                // Auth routes (public)
                .route("/register", get(get_register))
                .route("/register", post(post_register))
                .route("/login", get(get_login))
                .route("/login", post(post_login))
                // Password reset routes (public)
                .route("/password-reset", get(get_password_reset))
                .route("/password-reset", post(post_password_reset))
                .route("/password-reset/{token}", get(get_password_reset_complete))
                .route(
                    "/password-reset/{token}",
                    post(post_password_reset_complete),
                )
                // Stripe webhook (public, no auth - verified via signature)
                .route("/webhooks/stripe", post(post_stripe_webhook))
                // Merge protected routes
                .merge(protected_routes)
                // Static assets (no auth)
                .nest_service("/static", AssetsService::new())
                .with_state(state),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

#[tracing::instrument(skip(config))]
async fn migrate_command(config: imkitchen::config::Config) -> Result<()> {
    tracing::info!("Running database migrations...");

    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::info!("Database does not exist, creating: {}", config.database.url);
        sqlx::Sqlite::create_database(&config.database.url).await?;
    }

    // Set up database connection pool
    let db_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&config.database.url)
        .await?;

    // Run migrations
    run_migrations(&db_pool).await?;

    tracing::info!("Migrations completed successfully");

    Ok(())
}

#[tracing::instrument(skip(config))]
async fn reset_command(config: imkitchen::config::Config) -> Result<()> {
    tracing::info!("Resetting database...");

    // Drop database if it exists
    if sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::warn!("Dropping existing database: {}", config.database.url);
        sqlx::Sqlite::drop_database(&config.database.url).await?;
        tracing::info!("Database dropped successfully");
    } else {
        tracing::info!("Database does not exist, nothing to drop");
    }

    // Run migrate command to recreate and apply migrations
    migrate_command(config).await?;

    tracing::info!("Database reset completed successfully");

    Ok(())
}

#[tracing::instrument(skip(pool))]
async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<()> {
    // 1. Run SQLx migrations for read models
    sqlx::migrate!("./migrations").run(pool).await?;

    // 2. Run evento migrations for event store tables
    let mut conn = pool.acquire().await?;
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);

    Ok(())
}

// Placeholder dashboard handler
async fn dashboard_handler() -> &'static str {
    "Dashboard - Welcome!"
}
