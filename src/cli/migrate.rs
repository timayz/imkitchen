use anyhow::Result;
use evento::migrator::{Migrate, Plan};
use sqlx::migrate::MigrateDatabase;

/// Migrates the dedicated audience database (evento schema + rollup table).
/// Also called from `serve` startup: the audience DB is operational telemetry,
/// so it migrates automatically rather than requiring a manual step.
pub async fn migrate_audience(database_url: &str) -> Result<()> {
    tracing::info!("Running audience database migrations...");

    if !sqlx::Sqlite::database_exists(database_url).await? {
        tracing::info!("Audience database does not exist, creating: {database_url}");
        sqlx::Sqlite::create_database(database_url).await?;
    }

    let pool = imkitchen::create_pool(database_url, 1).await?;
    let mut conn = pool.acquire().await?;

    imkitchen_audience::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);
    pool.close().await;

    Ok(())
}

pub async fn migrate(config: imkitchen_web_shared::config::Config) -> Result<()> {
    tracing::info!("Running database migrations...");

    if !sqlx::Sqlite::database_exists(&config.database.url).await? {
        tracing::info!("Database does not exist, creating: {}", config.database.url);
        sqlx::Sqlite::create_database(&config.database.url).await?;
    }

    let pool = imkitchen::create_pool(&config.database.url, 1).await?;

    let mut conn = pool.acquire().await?;

    // Collapse any leftover `-wal` from a previous unclean shutdown up front, so the
    // migration transaction doesn't hit SQLITE_BUSY while WAL recovery runs lazily. The
    // long busy_timeout on this pool (see create_pool) rides out recovery on slow storage.
    sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&mut *conn)
        .await?;

    imkitchen_db::migrator::<sqlx::Sqlite>()?
        .run(&mut conn, &Plan::apply_all())
        .await?;
    drop(conn);

    // Reclaim free pages left behind by data migrations that shrink rows (e.g.
    // m0009 stripping ~1.9GB of image bytes out of the event log). VACUUM
    // rewrites the whole file and cannot run inside the migration transaction,
    // so it runs here on a fresh connection. Gate it on the freelist so routine
    // deploys with nothing to reclaim don't pay the cost.
    let mut conn = pool.acquire().await?;
    let freelist: i64 = sqlx::query_scalar("PRAGMA freelist_count")
        .fetch_one(&mut *conn)
        .await?;
    if freelist > 0 {
        tracing::info!("Reclaiming {freelist} free pages (VACUUM)...");
        sqlx::query("VACUUM").execute(&mut *conn).await?;
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut *conn)
            .await?;
    }
    drop(conn);

    if let Some(audience) = config.audience.as_ref() {
        migrate_audience(&audience.database_url).await?;
    }

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

    if let Some(audience) = config.audience.as_ref()
        && sqlx::Sqlite::database_exists(&audience.database_url).await?
    {
        tracing::warn!("Dropping audience database: {}", audience.database_url);
        sqlx::Sqlite::drop_database(&audience.database_url).await?;
    }

    migrate(config).await?;

    tracing::info!("Database reset completed successfully");

    Ok(())
}
