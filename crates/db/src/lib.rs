use sqlx_migrator::{Info, Migrator};

pub(crate) mod m0_10;
pub(crate) mod m0_11;
pub(crate) mod m0_12;
mod m0_9;
pub mod table;
pub mod types;

pub fn migrator<DB: sqlx::Database>() -> Result<Migrator<DB>, sqlx_migrator::Error>
where
    evento::sql_migrator::InitMigration: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0002: sqlx_migrator::Migration<DB>,
    m0_9::Migration: sqlx_migrator::Migration<DB>,
    m0_10::Migration: sqlx_migrator::Migration<DB>,
    m0_11::Migration: sqlx_migrator::Migration<DB>,
    m0_12::Migration: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new::<DB>()?;
    migrator.add_migrations(vec![
        Box::new(m0_9::Migration),
        Box::new(m0_10::Migration),
        Box::new(m0_11::Migration),
        Box::new(m0_12::Migration),
    ])?;

    Ok(migrator)
}
