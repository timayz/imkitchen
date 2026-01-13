pub mod contact;
mod service;
pub(crate) mod template;
pub mod user;

pub use service::*;

rust_i18n::i18n!("locales", fallback = "en");
