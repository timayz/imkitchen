mod command;
pub mod contact;
mod date;
pub mod mealplan;
pub mod recipe;
pub mod shopping;
pub mod user;

pub use command::*;
pub use date::*;
use evento::Executor;

#[derive(Clone)]
pub struct State<E: Executor> {
    pub executor: E,
    pub read_db: sqlx::SqlitePool,
    pub write_db: sqlx::SqlitePool,
}
