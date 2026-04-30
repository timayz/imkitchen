use anyhow::Result;
use evento::migrator::{Migrate, Plan};
use sqlx::migrate::MigrateDatabase;

pub async fn migrate(config: imkitchen_web_shared::config::Config) -> Result<()> {
    tracing::info!("Running database migrations...");

    if !sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::info!("Database does not exist, creating: {}", config.database.url);
        sqlx::Sqlite::create_database(&config.database.url).await?;
    }

    let pool = imkitchen::create_pool(&config.database.url, 1).await?;

    let mut conn = pool.acquire().await?;
    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);

    tracing::info!("Migrations completed successfully");

    Ok(())
}

pub async fn reset(config: imkitchen_web_shared::config::Config) -> Result<()> {
    tracing::info!("Resetting database...");

    if sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::warn!("Dropping existing database: {}", config.database.url);
        sqlx::Sqlite::drop_database(&config.database.url).await?;
        tracing::info!("Database dropped successfully");
    } else {
        tracing::info!("Database does not exist, nothing to drop");
    }

    migrate(config).await?;

    tracing::info!("Database reset completed successfully");

    Ok(())
}
