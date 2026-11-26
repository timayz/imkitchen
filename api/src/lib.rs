pub mod account;
mod admin;
pub mod auth;
pub mod cluster;
pub mod config;
pub mod db;
pub mod error;
mod health;
pub mod locale;
pub mod mail;
pub mod openapi;
pub mod server;
pub mod subscriptions;

// The sentences of `locales/`, for `locale::Locale` to read.
rust_i18n::i18n!("locales", fallback = "en");
