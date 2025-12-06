mod list;

pub use list::*;

#[derive(Clone)]
pub struct Query(pub sqlx::SqlitePool);
