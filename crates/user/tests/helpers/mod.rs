use evento::{
    Sqlite,
    migrator::{Migrate, Plan},
};
use sqlx::SqlitePool;
use sqlx_migrator::Info;

pub struct TestState {
    pub evento: Sqlite,
    pub pool: SqlitePool,
}

pub async fn setup_test_state() -> anyhow::Result<TestState> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    let mut migrator = evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?;
    migrator.add_migration(Box::new(imkitchen_user::sql::user::Migration01))?;
    let mut conn = pool.acquire().await?;
    migrator.run(&mut conn, &Plan::apply_all()).await?;

    Ok(TestState {
        evento: pool.clone().into(),
        pool,
    })
}
