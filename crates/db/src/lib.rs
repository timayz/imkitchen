use sqlx_migrator::{Info, Migrator};

mod m0_9;
pub mod table;

pub fn migrator<DB: sqlx::Database>() -> Result<Migrator<DB>, sqlx_migrator::Error>
where
    evento::sql_migrator::InitMigration: sqlx_migrator::Migration<DB>,
    m0_9::M0_9: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new_migrator::<DB>()?;
    migrator.add_migrations(vec![Box::new(m0_9::M0_9)])?;

    Ok(migrator)
}
