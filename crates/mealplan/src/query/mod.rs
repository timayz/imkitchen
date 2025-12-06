mod slot;
mod week;

pub use slot::*;
pub use week::*;

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);
