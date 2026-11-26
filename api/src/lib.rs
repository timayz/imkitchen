pub mod account;
mod admin;
pub mod auth;
pub mod cluster;
pub mod config;
pub mod db;
pub mod error;
pub mod groceries;
mod health;
pub mod images;
pub mod list;
pub mod locale;
pub mod mail;
pub mod openapi;
pub mod plats;
pub mod recipes;
pub mod server;
pub mod settings;
pub mod sidecars;
pub mod subscriptions;
pub mod tutorial;

// The sentences of `locales/`, for `locale::Locale` to read.
rust_i18n::i18n!("locales", fallback = "en");
