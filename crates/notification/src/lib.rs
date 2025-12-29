// mod contact;
mod service;
pub(crate) mod template;
// mod user;

// pub use contact::*;
pub use service::*;
// pub use user::*;

rust_i18n::i18n!("locales", fallback = "en");
