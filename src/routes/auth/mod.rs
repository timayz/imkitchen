//! Authentication route handlers

pub mod login;
pub mod profile;
pub mod register;

use sqlx::SqlitePool;

use crate::access_control::AccessControlService;
use crate::Config;

// Re-export route handlers
pub use login::{get_login, post_login, post_logout};
pub use profile::{get_profile, post_profile};
pub use register::{get_register, get_register_status, post_register};

// Re-export render_template helper from parent
pub(crate) use super::render_template;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub evento: evento::Sqlite,
    pub query_pool: SqlitePool,
    pub jwt_secret: String,
    pub jwt_lifetime_seconds: u64,
    pub config: Config,
    pub access_control: AccessControlService,
}
