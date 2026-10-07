pub mod assets;
pub mod auth;
pub mod config;
pub mod language;
pub mod middleware;
pub mod services;
pub mod state;
pub mod template;

/// Used by `try_response!` so callers need not depend on `imkitchen-core` themselves.
#[doc(hidden)]
pub use imkitchen_core;

pub use state::{
    AdminImportError, AdminImportJobs, AdminImportProgress, AppState, SitemapPayload, SitemapStore,
};

rust_i18n::i18n!("../../locales", fallback = "en");
