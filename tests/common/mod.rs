use axum::{
    routing::{get, post},
    Router,
};
use evento::prelude::*;
use sqlx::SqlitePool;
use user::user_projection;

pub async fn setup_test_db() -> (SqlitePool, evento::Sqlite) {
    // Use the optimized pool creation from db module
    let pool = imkitchen::db::create_pool(":memory:", 1).await.unwrap();

    // Run SQLx migrations for read models
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    // Run evento migrations for event store tables
    let mut conn = pool.acquire().await.unwrap();
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
        .unwrap()
        .run(&mut conn, &Plan::apply_all())
        .await
        .unwrap();
    drop(conn);

    // Create evento executor
    let executor: evento::Sqlite = pool.clone().into();

    (pool, executor)
}

#[allow(dead_code)]
pub struct TestApp {
    pub router: Router,
    pub evento_executor: evento::Sqlite,
    pub pool: SqlitePool,
}

#[allow(dead_code)]
impl TestApp {
    /// Process all pending events synchronously
    pub async fn process_events(&self) {
        user_projection(self.pool.clone())
            .unsafe_oneshot(&self.evento_executor)
            .await
            .unwrap();
    }
}

#[allow(dead_code)]
pub async fn create_test_app((pool, evento_executor): (SqlitePool, evento::Sqlite)) -> TestApp {
    use axum::middleware as axum_middleware;
    use imkitchen::middleware::auth_middleware;
    use imkitchen::routes::{
        get_check_user, get_contact, get_help, get_login, get_onboarding, get_onboarding_skip,
        get_profile, get_register, get_subscription, get_subscription_success, post_contact,
        post_login, post_logout, post_onboarding_step_1, post_onboarding_step_2,
        post_onboarding_step_3, post_profile, post_register, post_subscription_upgrade, AppState,
    };

    let email_config = imkitchen::email::EmailConfig {
        smtp_host: "localhost".to_string(),
        smtp_port: 587,
        smtp_username: "test@example.com".to_string(),
        smtp_password: "password".to_string(),
        from_email: "noreply@imkitchen.app".to_string(),
        from_name: "imkitchen".to_string(),
    };

    let state = AppState {
        db_pool: pool.clone(),
        write_pool: pool.clone(),
        evento_executor: evento_executor.clone(),
        jwt_secret: "test_secret_key_minimum_32_characters_long".to_string(),
        email_config,
        base_url: "http://localhost:3000".to_string(),
        stripe_secret_key: "sk_test_mock_key".to_string(),
        stripe_webhook_secret: "whsec_test_mock_secret".to_string(),
        stripe_price_id: "price_test_mock_id".to_string(),
        vapid_public_key: "BEl62iUYgUivxIkv69yViEuiBIa-Ib9-SkvMeAtA3LFgDzkrxZJjSgSnfckjBJuBkr3qBUYIHBQFLXYp5Nqm50g".to_string(),
        generation_locks: std::sync::Arc::new(tokio::sync::Mutex::new(
            std::collections::HashMap::new(),
        )),
        bypass_premium: false, // Tests should verify premium logic works correctly
    };

    // Create protected routes with auth middleware
    let protected_router = Router::new()
        .route("/logout", post(post_logout))
        .route("/onboarding", get(get_onboarding))
        .route("/onboarding/step/1", post(post_onboarding_step_1))
        .route("/onboarding/step/2", post(post_onboarding_step_2))
        .route("/onboarding/step/3", post(post_onboarding_step_3))
        .route("/onboarding/skip", get(get_onboarding_skip))
        .route("/profile", get(get_profile))
        .route("/profile", post(post_profile))
        .route("/subscription", get(get_subscription))
        .route("/subscription/upgrade", post(post_subscription_upgrade))
        .route("/subscription/success", get(get_subscription_success))
        .route("/", get(|| async { "Dashboard" }))
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Create public routes
    let router = Router::new()
        .route("/register", get(get_register))
        .route("/register", post(post_register))
        .route("/register/check-user/{user_id}", get(get_check_user))
        .route("/login", get(get_login))
        .route("/login", post(post_login))
        // Support pages (public)
        .route("/help", get(get_help))
        .route("/contact", get(get_contact).post(post_contact))
        .merge(protected_router)
        .with_state(state);

    TestApp {
        router,
        evento_executor,
        pool,
    }
}
