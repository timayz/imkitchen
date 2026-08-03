//! First-party audience measurement, event-sourced on its own dedicated
//! evento instance/SQLite database so beacon writes never contend with the
//! main application database.

pub mod audience_daily_stat;
pub mod daily_stat;
pub mod types;

mod record_visit;
mod tz_country;
mod ua;

pub use record_visit::RecordVisitInput;

use evento::Executor;
use sqlx_migrator::{Info, Migrator};
use std::ops::Deref;

pub(crate) mod m0001 {
    use sqlx_migrator::vec_box;

    pub struct Migration;

    sqlx_migrator::sqlite_migration!(
        Migration,
        "imkitchen-audience",
        "m0001",
        vec_box![],
        vec_box![
            crate::audience_daily_stat::m0001::CreateTable,
            crate::audience_daily_stat::m0001::CreateUk1,
        ]
    );
}

/// Migrator for the dedicated audience database: evento's own schema plus the
/// rollup table. Entirely separate from `imkitchen_db::migrator` — the two
/// databases' migration state never mixes.
pub fn migrator<DB: sqlx::Database>() -> Result<Migrator<DB>, sqlx_migrator::Error>
where
    evento::sql_migrator::InitMigration: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0002: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0003: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0004: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0005: sqlx_migrator::Migration<DB>,
    m0001::Migration: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new::<DB>()?;
    migrator.add_migrations(vec![Box::new(m0001::Migration)])?;

    Ok(migrator)
}

#[derive(Clone)]
pub struct State<E: Executor> {
    pub executor: E,
    pub read_db: sqlx::SqlitePool,
    pub write_db: sqlx::SqlitePool,
}

#[derive(Clone)]
pub struct Module<E: Executor>(State<E>);

impl<E: Executor> Deref for Module<E> {
    type Target = State<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Executor> Module<E> {
    pub fn new(state: State<E>) -> Self {
        Self(state)
    }
}
