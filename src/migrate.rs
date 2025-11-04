use anyhow::Result;
use evento::migrator::{Migrate, Plan};
use sqlx::migrate::MigrateDatabase;

pub async fn migrate(config: crate::config::Config) -> Result<()> {
    tracing::info!("Running database migrations...");

    // Create database if it doesn't exist
    if !sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::info!("Database does not exist, creating: {}", config.database.url);
        sqlx::Sqlite::create_database(&config.database.url).await?;
    }

    // Set up database connection pool with optimized PRAGMAs
    let pool = crate::db::create_pool(&config.database.url, 1).await?;

    // 2. Run evento migrations for event store tables
    let mut conn = pool.acquire().await?;
    evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);

    tracing::info!("Migrations completed successfully");

    Ok(())
}

pub async fn reset(config: crate::config::Config) -> Result<()> {
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
    migrate(config).await?;

    tracing::info!("Database reset completed successfully");

    Ok(())
}
