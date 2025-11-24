use std::{path::PathBuf, str::FromStr};

use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

pub struct TestState {
    pub evento: Sqlite,
    pub pool: SqlitePool,
}

pub async fn setup_test_state(path: PathBuf) -> anyhow::Result<TestState> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(TestState {
        evento: pool.clone().into(),
        pool,
    })
}
