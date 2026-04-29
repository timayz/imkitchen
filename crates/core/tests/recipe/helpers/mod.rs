use evento::Sqlite;
use evento::migrator::{Migrate, Plan};
use imkitchen_shared::State;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::PathBuf, str::FromStr};

pub async fn setup_test_state(path: PathBuf) -> anyhow::Result<State<Sqlite>> {
    let opts = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.to_str().unwrap()))?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;

    Ok(State {
        executor: pool.clone().into(),
        read_db: pool.clone(),
        write_db: pool,
    })
}
