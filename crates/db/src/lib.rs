use sqlx_migrator::{Info, Migrator};

pub(crate) mod m0001;
pub mod table;

pub fn migrator<DB: sqlx::Database>() -> Result<Migrator<DB>, sqlx_migrator::Error>
where
    evento::sql_migrator::InitMigration: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0002: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0003: sqlx_migrator::Migration<DB>,
    m0001::Migration: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new::<DB>()?;
    migrator.add_migrations(vec![Box::new(m0001::Migration)])?;

    Ok(migrator)
}
