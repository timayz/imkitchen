pub mod meal_preferences;
pub mod password;

pub(crate) mod query;
pub(crate) mod repository;
mod root;

pub use query::{admin, global_stat, login, query_subscription};
pub use root::*;
