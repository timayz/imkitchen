use axum::{
    routing::{get, post},
    Router,
};
use evento::prelude::*;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use user::user_projection;

pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await
        .unwrap();

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

    pool
}

pub struct TestApp {
    pub router: Router,
    pub evento_executor: evento::Sqlite,
    pub pool: SqlitePool,
}

impl TestApp {
    /// Process all pending events synchronously
    pub async fn process_events(&self) {
        user_projection(self.pool.clone())
            .run_once(&self.evento_executor)
            .await
            .unwrap();
    }
}

pub async fn create_test_app(pool: SqlitePool) -> TestApp {
    use imkitchen::routes::{get_login, get_register, post_login, post_register, AppState};

    // Create evento executor
    let evento_executor: evento::Sqlite = pool.clone().into();

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
        evento_executor: evento_executor.clone(),
        jwt_secret: "test_secret_key_minimum_32_characters_long".to_string(),
        email_config,
        base_url: "http://localhost:3000".to_string(),
    };

    let router = Router::new()
        .route("/register", get(get_register))
        .route("/register", post(post_register))
        .route("/login", get(get_login))
        .route("/login", post(post_login))
        .route("/dashboard", get(|| async { "Dashboard" }))
        .with_state(state);

    TestApp {
        router,
        evento_executor,
        pool,
    }
}
