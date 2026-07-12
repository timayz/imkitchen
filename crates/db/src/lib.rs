use sqlx_migrator::{Info, Migrator};

pub(crate) mod m0001;
pub(crate) mod m0002;
pub(crate) mod m0003;
pub(crate) mod m0004;
pub(crate) mod m0005;
pub(crate) mod m0006;
pub(crate) mod m0007;
pub(crate) mod m0008;
pub(crate) mod m0009;
pub(crate) mod m0010;

pub mod contact_admin;
pub mod contact_global_stat;
pub mod fts;
pub mod mealplan_recipe;
pub mod mealplan_slot;
pub mod notification_recipient;
pub mod origin_framing;
pub mod recipe_owner;
pub mod recipe_thumbnail;
pub mod recipe_user;
pub mod recipe_user_stat;
pub mod shopping_list;
pub mod shopping_recipe;
pub mod shopping_slot;
pub mod user;
pub mod user_admin;
pub mod user_global_stat;
pub mod user_invoice_user;
pub mod user_login;
pub mod user_subscription;

pub fn migrator<DB: sqlx::Database>() -> Result<Migrator<DB>, sqlx_migrator::Error>
where
    evento::sql_migrator::InitMigration: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0002: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0003: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0004: sqlx_migrator::Migration<DB>,
    evento::sql_migrator::M0005: sqlx_migrator::Migration<DB>,
    m0001::Migration: sqlx_migrator::Migration<DB>,
    m0002::Migration: sqlx_migrator::Migration<DB>,
    m0003::Migration: sqlx_migrator::Migration<DB>,
    m0004::Migration: sqlx_migrator::Migration<DB>,
    m0005::Migration: sqlx_migrator::Migration<DB>,
    m0006::Migration: sqlx_migrator::Migration<DB>,
    m0007::Migration: sqlx_migrator::Migration<DB>,
    m0008::Migration: sqlx_migrator::Migration<DB>,
    m0009::Migration: sqlx_migrator::Migration<DB>,
    m0010::Migration: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new::<DB>()?;
    migrator.add_migrations(vec![
        Box::new(m0001::Migration),
        Box::new(m0002::Migration),
        Box::new(m0003::Migration),
        Box::new(m0004::Migration),
        Box::new(m0005::Migration),
        Box::new(m0006::Migration),
        Box::new(m0007::Migration),
        Box::new(m0008::Migration),
        Box::new(m0009::Migration),
        Box::new(m0010::Migration),
    ])?;

    Ok(migrator)
}
