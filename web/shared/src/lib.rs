pub mod assets;
pub mod auth;
pub mod config;
pub mod language;
pub mod middleware;
pub mod state;
pub mod template;

pub use state::AppState;

rust_i18n::i18n!("../../locales", fallback = "en");
