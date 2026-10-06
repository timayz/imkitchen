//! Test harness: a real `AppState` over a throwaway SQLite database, the same
//! wiring `src/cli/server.rs` uses minus the background subscriptions. Tests
//! that need a projection (recipe lists, thumbnails…) start the relevant
//! subscription themselves, like `web/admin/tests/thumbnail.rs` does.

use std::str::FromStr;

use evento::migrator::{Migrate, Plan};
use imkitchen_web_shared::AppState;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use temp_dir::TempDir;

pub struct TestApp {
    pub state: AppState,
    #[allow(dead_code)]
    pub pool: SqlitePool,
    /// Background subscriptions started by [`TestApp::with_recipes`]; dropped
    /// with the app.
    #[allow(dead_code)]
    subscriptions: Vec<evento::subscription::Subscription>,
    // Dropped last: deleting the directory while the pool is open is fine on
    // Linux but keeping it explicit documents the lifetime.
    _dir: TempDir,
}

impl TestApp {
    pub async fn new() -> anyhow::Result<Self> {
        let dir = TempDir::new()?;
        let path = dir.child("db.sqlite3");
        let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(opts).await?;
        {
            let mut conn = pool.acquire().await?;
            imkitchen_db::migrator::<sqlx::Sqlite>()?
                .run(&mut conn, &Plan::apply_all())
                .await?;
        }

        // Low stability margin keeps subscription drains fast in tests.
        let stable_margin = std::time::Duration::from_millis(100);
        let rw: evento::sql::RwSqlite = (
            evento::Sqlite::from(pool.clone()).stable_margin(stable_margin),
            evento::Sqlite::from(pool.clone()).stable_margin(stable_margin),
        )
            .into();
        let executor = evento::Evento::new(rw);

        let inner = imkitchen_core::State {
            executor,
            read_db: pool.clone(),
            write_db: pool.clone(),
        };

        // Defaults only: no config file, no env. `[premium]` is absent, so
        // every user is premium (same as a dev checkout).
        let config = imkitchen_web_shared::config::Config::load(Some(
            "/nonexistent/imkitchen-test.toml".to_owned(),
        ))?;
        let stripe = stripe::ClientBuilder::new(&config.stripe.secret_key).build()?;

        let state = AppState {
            config,
            stripe,
            identity: imkitchen_identity::Module::new(inner.clone()),
            billing: imkitchen_billing::Billing::new(inner.clone()),
            core: imkitchen_core::Core::new(inner.clone()),
            audience: None,
            import_jobs: Default::default(),
            sitemap: Default::default(),
            inner,
        };

        Ok(Self {
            state,
            pool,
            subscriptions: vec![],
            _dir: dir,
        })
    }

    /// Starts the recipe and shopping read models (same wiring as
    /// `src/cli/server.rs`), needed by anything that lists recipes or
    /// builds the candidate pool. Writes become visible after a short delay:
    /// poll with [`wait_until`].
    #[allow(dead_code)]
    pub async fn with_recipes() -> anyhow::Result<Self> {
        let mut app = Self::new().await?;
        let executor = &app.state.inner.executor;
        let pool = app.pool.clone();

        app.subscriptions.push(
            imkitchen_core::recipe::subscription()
                .data((pool.clone(), pool.clone()))
                .start(executor)
                .await?,
        );
        app.subscriptions.push(
            imkitchen_core::recipe::query::user::create_projection()
                .data((pool.clone(), pool.clone()))
                .subscription("recipe-query")
                .any_routing_key()
                .start(executor)
                .await?,
        );
        app.subscriptions.push(
            imkitchen_core::recipe::query::user_fts::subscription()
                .data(pool.clone())
                .any_routing_key()
                .start(executor)
                .await?,
        );
        app.subscriptions.push(
            imkitchen_core::recipe::query::user_stat::subscription()
                .data(pool.clone())
                .any_routing_key()
                .start(executor)
                .await?,
        );
        app.subscriptions.push(
            imkitchen_core::shopping::pool::subscription()
                .data(pool.clone())
                .start(executor)
                .await?,
        );
        app.subscriptions.push(
            imkitchen_core::shopping::subscription()
                .data(pool.clone())
                .start(executor)
                .await?,
        );

        Ok(app)
    }

    /// The API router with state applied, ready for `tower::ServiceExt::oneshot`.
    pub fn router(&self) -> axum::Router {
        imkitchen_web_api::routes().with_state(self.state.clone())
    }
}

/// Reads a JSON body. Panics with the raw body on invalid JSON so failures are
/// readable.
pub async fn json(response: axum::response::Response) -> serde_json::Value {
    use http_body_util::BodyExt;

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    if bytes.is_empty() {
        return serde_json::Value::Null;
    }
    serde_json::from_slice(&bytes).unwrap_or_else(|e| {
        panic!(
            "invalid JSON body: {e}\n{}",
            String::from_utf8_lossy(&bytes)
        )
    })
}

/// Polls `check` every 50 ms until it returns `Some`, for at most 5 s.
#[allow(dead_code)]
pub async fn wait_until<T, F, Fut>(mut check: F) -> T
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    for _ in 0..100 {
        if let Some(value) = check().await {
            return value;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("condition not met within 5s");
}
