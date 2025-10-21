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
    browser_support, check_recipe_exists, check_shopping_item, complete_prep_task_handler,
    dashboard_handler, dismiss_notification, generate_shopping_list_handler, get_check_user,
    get_collections, get_discover, get_discover_detail, get_import_modal, get_ingredient_row,
    get_instruction_row, get_landing, get_login, get_meal_alternatives, get_meal_plan,
    get_notification_status, get_onboarding, get_onboarding_skip, get_password_reset,
    get_password_reset_complete, get_privacy, get_profile, get_recipe_detail, get_recipe_edit_form,
    get_recipe_form, get_recipe_list, get_recipe_waiting, get_regenerate_confirm, get_register,
    get_subscription, get_subscription_success, get_terms, health, list_notifications,
    notifications_page, offline, post_add_recipe_to_collection, post_add_to_library,
    post_create_collection, post_create_recipe, post_delete_collection, post_delete_recipe,
    post_delete_review, post_favorite_recipe, post_generate_meal_plan, post_import_recipes,
    post_login, post_logout, post_onboarding_step_1, post_onboarding_step_2,
    post_onboarding_step_3, post_onboarding_step_4, post_password_reset,
    post_password_reset_complete, post_profile, post_rate_recipe, post_regenerate_meal_plan,
    post_register, post_remove_recipe_from_collection, post_replace_meal, post_share_recipe,
    post_stripe_webhook, post_subscription_upgrade, post_update_collection, post_update_recipe,
    post_update_recipe_tags, ready, record_permission_change, refresh_shopping_list,
    reset_shopping_list_handler, show_shopping_list, snooze_notification, subscribe_push, AppState,
    AssetsService,
};
use meal_planning::meal_plan_projection;
use notifications::{meal_plan_subscriptions, notification_projections};
use recipe::{collection_projection, recipe_projection};
use shopping::shopping_projection;
use sqlx::migrate::MigrateDatabase;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
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
    /// Upgrade or downgrade user subscription tier
    SetTier {
        /// User email address
        #[arg(long)]
        email: String,

        /// New subscription tier (free or premium)
        #[arg(long)]
        tier: String,
    },
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
        Commands::SetTier { email, tier } => set_tier_command(config, email, tier).await,
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

    // Set up database connection pools with optimized PRAGMAs
    // Write pool: 1 connection for evento and all write operations
    let write_pool = imkitchen::db::create_write_pool(&config.database.url).await?;

    // Read pool: Multiple connections for read-only queries
    // Use CPU cores as a reasonable default for max connections
    let read_pool_size = config.database.max_connections;
    let read_pool = imkitchen::db::create_read_pool(&config.database.url, read_pool_size).await?;

    // Create evento executor using the write pool (single connection for writes)
    let evento_executor: evento::Sqlite = write_pool.clone().into();

    // Set up evento subscription for read model projections
    // Projections write to read models, so they use the write pool
    user_projection(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'user-read-model' started");

    recipe_projection(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'recipe-read-model' started");

    collection_projection(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'collection-read-model' started");

    meal_plan_projection(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'meal-plan-read-model' started");

    shopping_projection(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'shopping-read-model' started");

    // Notification projections
    notification_projections(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'notification-projections' started");

    // Notification event subscriptions (business logic)
    meal_plan_subscriptions(write_pool.clone())
        .run(&evento_executor)
        .await?;
    tracing::info!("Evento subscription 'notification-meal-plan-listeners' started");

    // Clone references for background worker before moving into state
    let worker_pool = write_pool.clone();
    let worker_executor = evento_executor.clone();

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
        db_pool: read_pool.clone(),     // Read pool for queries
        write_pool: write_pool.clone(), // Write pool for inserts/updates
        evento_executor,
        jwt_secret: config.jwt.secret,
        email_config,
        base_url: config.email.base_url,
        stripe_secret_key: config.stripe.secret_key,
        stripe_webhook_secret: config.stripe.webhook_secret,
        stripe_price_id: config.stripe.price_id,
        vapid_public_key: config.vapid.public_key.clone(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
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
        .route("/recipes/import-modal", get(get_import_modal)) // Story 2.12: Batch import modal
        .route("/recipes/import", post(post_import_recipes)) // Story 2.12: Batch import handler
        .route("/discover", get(get_discover))
        .route("/discover/{id}", get(get_discover_detail))
        .route("/discover/{id}/add", post(post_add_to_library))
        .route("/discover/{id}/rate", post(post_rate_recipe))
        .route("/discover/{id}/review/delete", post(post_delete_review))
        .route("/recipes/{id}/waiting", get(get_recipe_waiting))
        .route("/recipes/{id}/check", get(check_recipe_exists))
        .route("/recipes/{id}", get(get_recipe_detail))
        .route("/recipes/{id}/edit", get(get_recipe_edit_form))
        .route("/recipes/{id}", post(post_update_recipe))
        .route("/recipes/{id}/delete", post(post_delete_recipe))
        .route("/recipes/{id}/favorite", post(post_favorite_recipe))
        .route("/recipes/{id}/share", post(post_share_recipe))
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
        // Meal planning routes
        .route("/plan", get(get_meal_plan))
        .route("/plan/generate", post(post_generate_meal_plan))
        .route("/plan/regenerate/confirm", get(get_regenerate_confirm))
        .route("/plan/regenerate", post(post_regenerate_meal_plan))
        .route(
            "/plan/meal/{assignment_id}/alternatives",
            get(get_meal_alternatives),
        )
        .route(
            "/plan/meal/{assignment_id}/replace",
            post(post_replace_meal),
        )
        // Shopping list routes
        .route("/shopping", get(show_shopping_list))
        .route("/shopping/generate", post(generate_shopping_list_handler))
        .route("/shopping/refresh", get(refresh_shopping_list))
        .route("/shopping/items/{id}/check", post(check_shopping_item))
        .route("/shopping/{week}/reset", post(reset_shopping_list_handler))
        // Notification routes
        .route("/notifications", get(notifications_page))
        .route("/api/notifications", get(list_notifications))
        .route(
            "/api/notifications/{id}/dismiss",
            post(dismiss_notification),
        )
        .route("/api/notifications/{id}/snooze", post(snooze_notification))
        .route(
            "/api/notifications/{id}/complete",
            post(complete_prep_task_handler),
        )
        .route("/api/notifications/subscribe", post(subscribe_push))
        .route(
            "/api/notifications/permission",
            post(record_permission_change),
        )
        .route("/api/notifications/status", get(get_notification_status))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Build router with health checks using read pool state
    let app = Router::new()
        // Health check endpoints (no auth required)
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(read_pool.clone())
        .merge(
            Router::new()
                // Offline fallback page (public, no auth)
                .route("/offline", get(offline))
                // Browser compatibility information page (public, no auth) - Story 5.7
                .route("/browser-support", get(browser_support))
                .with_state(state.clone()),
        )
        .merge(
            Router::new()
                // Landing page (public)
                .route("/", get(get_landing))
                // Legal pages (public)
                .route("/privacy", get(get_privacy))
                .route("/terms", get(get_terms))
                // Auth routes (public)
                .route("/register", get(get_register))
                .route("/register", post(post_register))
                .route("/register/check-user/{user_id}", get(get_check_user))
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
        // Enable Brotli and Gzip compression for all text assets (Story 5.9)
        .layer(CompressionLayer::new().br(true).gzip(true))
        .layer(TraceLayer::new_for_http());

    // Start background notification worker
    tracing::info!("Starting notification background worker...");

    // Convert VAPID config to WebPushConfig if keys are provided
    let web_push_config =
        if !config.vapid.public_key.is_empty() && !config.vapid.private_key.is_empty() {
            Some(notifications::WebPushConfig {
                vapid_public_key: config.vapid.public_key.clone(),
                vapid_private_key: config.vapid.private_key.clone(),
                subject: config.vapid.subject.clone(),
            })
        } else {
            tracing::warn!("VAPID keys not configured - push notifications will be disabled");
            None
        };

    let notification_worker = Arc::new(notifications::NotificationWorker::new(
        worker_pool,
        Arc::new(worker_executor),
        web_push_config,
    ));
    tokio::spawn(async move {
        notification_worker.run().await;
    });
    tracing::info!("Notification worker started");

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

    // Set up database connection pool with optimized PRAGMAs
    let db_pool = imkitchen::db::create_pool(&config.database.url, 1).await?;

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

#[tracing::instrument(skip(config))]
async fn set_tier_command(
    config: imkitchen::config::Config,
    user_email: String,
    tier: String,
) -> Result<()> {
    use user::commands::{upgrade_subscription, UpgradeSubscriptionCommand};
    use user::types::SubscriptionTier;

    tracing::info!("Setting subscription tier for user: {}", user_email);

    // Validate and parse tier
    let subscription_tier: SubscriptionTier = tier
        .parse()
        .map_err(|e: String| anyhow::anyhow!("Invalid tier: {}", e))?;

    tracing::info!(
        "Parsed tier: {} ({})",
        subscription_tier,
        subscription_tier.as_str()
    );

    // Set up database connection pool with optimized PRAGMAs
    let db_pool =
        imkitchen::db::create_pool(&config.database.url, config.database.max_connections).await?;

    // Set up evento executor
    let evento_executor: evento::Sqlite = db_pool.clone().into();

    // Query user by email
    let user = sqlx::query_as::<_, (String, String, String)>(
        r#"SELECT id, email, tier FROM users WHERE email = ? LIMIT 1"#,
    )
    .bind(&user_email)
    .fetch_optional(&db_pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("User with email '{}' not found", user_email))?;

    let (user_id, email, current_tier) = user;
    tracing::info!("Found user: {} ({})", email, user_id);
    tracing::info!("Current tier: {}", current_tier);

    // Check if tier is already set
    if current_tier == subscription_tier.as_str() {
        tracing::warn!(
            "User {} is already on {} tier. No changes made.",
            email,
            subscription_tier
        );
        println!(
            "⚠️  User {} is already on {} tier",
            email, subscription_tier
        );
        return Ok(());
    }

    // Create upgrade command (works for both upgrade and downgrade)
    let command = UpgradeSubscriptionCommand {
        user_id: user_id.clone(),
        new_tier: subscription_tier.as_str().to_string(),
        stripe_customer_id: None, // No Stripe metadata for manual tier changes
        stripe_subscription_id: None,
    };

    // Execute subscription upgrade command
    upgrade_subscription(command, &evento_executor).await?;

    // Run projections to persist event to read model
    user::read_model::user_projection(db_pool.clone())
        .unsafe_oneshot(&evento_executor)
        .await?;

    tracing::info!(
        "✅ Subscription tier updated successfully: {} -> {}",
        current_tier,
        subscription_tier
    );
    println!("✅ Subscription tier updated successfully!");
    println!("   User: {} ({})", email, user_id);
    println!("   Old tier: {}", current_tier);
    println!("   New tier: {}", subscription_tier);

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
