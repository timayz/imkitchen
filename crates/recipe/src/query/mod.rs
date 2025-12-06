mod list;
mod user_stat;

pub use list::*;
pub use user_stat::*;

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);
