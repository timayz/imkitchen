pub mod config;
pub mod email;
pub mod error;
pub mod middleware;
pub mod observability;
pub mod routes;

pub use routes::AppState;

/// Create app router for testing
///
/// This function creates the Axum router with all routes configured,
/// useful for integration testing without starting the full server.
pub async fn create_app(
    db_pool: sqlx::SqlitePool,
    evento_executor: evento::Sqlite,
) -> anyhow::Result<axum::Router> {
    use axum::{
        middleware as axum_middleware,
        routing::{get, post},
        Router,
    };
    use middleware::auth_middleware;
    use routes::{
        browser_support, check_recipe_exists, get_ingredient_row, get_instruction_row, get_login,
        get_password_reset, get_password_reset_complete, get_recipe_detail, get_recipe_edit_form,
        get_recipe_form, get_recipe_waiting, get_register, health, offline, post_create_recipe,
        post_login, post_logout, post_password_reset, post_password_reset_complete, post_register,
        post_update_recipe, ready, AssetsService,
    };

    let email_config = email::EmailConfig {
        smtp_host: "localhost".to_string(),
        smtp_port: 1025,
        smtp_username: "test".to_string(),
        smtp_password: "test".to_string(),
        from_email: "test@test.com".to_string(),
        from_name: "Test".to_string(),
    };

    let state = AppState {
        db_pool: db_pool.clone(),
        evento_executor,
        jwt_secret: "test-secret-key-32-bytes-long!!".to_string(),
        email_config,
        base_url: "http://localhost:3000".to_string(),
        stripe_secret_key: "sk_test_".to_string(),
        stripe_webhook_secret: "whsec_test".to_string(),
        stripe_price_id: "price_test".to_string(),
        vapid_public_key: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nqm50g".to_string(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
    };

    // Build protected routes with auth middleware
    let protected_routes = Router::new()
        .route("/logout", post(post_logout))
        // Recipe routes
        .route("/recipes/new", get(get_recipe_form))
        .route("/recipes", post(post_create_recipe))
        .route("/recipes/{id}/waiting", get(get_recipe_waiting))
        .route("/recipes/{id}/check", get(check_recipe_exists))
        .route("/recipes/{id}", get(get_recipe_detail))
        .route("/recipes/{id}/edit", get(get_recipe_edit_form))
        .route("/recipes/{id}", post(post_update_recipe))
        .route("/recipes/ingredient-row", get(get_ingredient_row))
        .route("/recipes/instruction-row", get(get_instruction_row))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .with_state(db_pool)
        .merge(
            Router::new()
                // Offline fallback page (public, no auth)
                .route("/offline", get(offline))
                // Browser compatibility information page (public, no auth) - Story 5.7
                .route("/browser-support", get(browser_support))
                .route("/register", get(get_register))
                .route("/register", post(post_register))
                .route("/login", get(get_login))
                .route("/login", post(post_login))
                .route("/password-reset", get(get_password_reset))
                .route("/password-reset", post(post_password_reset))
                .route("/password-reset/{token}", get(get_password_reset_complete))
                .route(
                    "/password-reset/{token}",
                    post(post_password_reset_complete),
                )
                .merge(protected_routes)
                .nest_service("/static", AssetsService::new())
                .with_state(state),
        );

    Ok(app)
}
