//! Test harness: a real `AppState` over a throwaway SQLite database, the same
//! wiring `src/cli/server.rs` uses minus the background subscriptions. Tests
//! that need a projection (recipe lists, the grocery list…) call
//! [`TestApp::drain`] after writing: it runs the read models to completion
//! with `run_once`, so the next read is deterministic and nothing is polled.

use std::str::FromStr;

use evento::migrator::{Migrate, Plan};
use imkitchen_web_shared::AppState;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use temp_dir::TempDir;

pub struct TestApp {
    pub state: AppState,
    #[allow(dead_code)]
    pub pool: SqlitePool,
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
            _dir: dir,
        })
    }

    /// Drains the recipe and shopping read models (same wiring as
    /// `src/cli/server.rs`), in dependency order: the shopping tables are
    /// built from the recipe projections, which are built from the recipe
    /// stream. `run_once` processes everything committed before the call and
    /// returns, so after this every read reflects every write made so far.
    ///
    /// Cursors are persisted per subscription key, so repeated drains only
    /// process what is new. Never start the same subscriptions in the
    /// background as well: a worker on the same key would take ownership away
    /// from the drain.
    #[allow(dead_code)]
    pub async fn drain(&self) -> anyhow::Result<()> {
        let executor = &self.state.inner.executor;
        let pool = self.pool.clone();

        imkitchen_core::recipe::subscription()
            .data((pool.clone(), pool.clone()))
            .no_retry()
            .run_once(executor)
            .await?;
        imkitchen_core::recipe::query::user::create_projection()
            .data((pool.clone(), pool.clone()))
            .subscription("recipe-query")
            .any_routing_key()
            .no_retry()
            .run_once(executor)
            .await?;
        imkitchen_core::recipe::query::user_fts::subscription()
            .data(pool.clone())
            .any_routing_key()
            .no_retry()
            .run_once(executor)
            .await?;
        imkitchen_core::recipe::query::user_stat::subscription()
            .data(pool.clone())
            .any_routing_key()
            .no_retry()
            .run_once(executor)
            .await?;
        imkitchen_core::shopping::pool::subscription()
            .data(pool.clone())
            .no_retry()
            .run_once(executor)
            .await?;
        imkitchen_core::shopping::subscription()
            .data(pool.clone())
            .no_retry()
            .run_once(executor)
            .await?;

        Ok(())
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
