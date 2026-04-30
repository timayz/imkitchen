use sqlx_migrator::{Info, Migrator};

pub(crate) mod m0001;

pub mod contact_admin;
pub mod contact_global_stat;
pub mod mealplan_recipe;
pub mod mealplan_slot;
pub mod recipe_comment;
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
    m0001::Migration: sqlx_migrator::Migration<DB>,
{
    let mut migrator = evento::sql_migrator::new::<DB>()?;
    migrator.add_migrations(vec![Box::new(m0001::Migration)])?;

    Ok(migrator)
}
