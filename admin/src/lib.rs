//! imkitchen admin: the administration of imkitchen.
//!
//! admin owns no aggregate and no command. What it shows of imkitchen it
//! derives from the api's events, followed through the api's listener for
//! the sidecars ([`events`]) into a store of its own ([`db`]). Who reads it,
//! the api says ([`api`]): only an administrator does.
//! What an administrator does is a command of the api's, sent over HTTP
//! ([`api`]): admin writes nothing itself.

pub mod api;
pub mod app;
pub mod config;
pub mod db;
pub mod events;
pub mod images;
pub mod imports;
pub mod listing;
pub mod messages;
pub mod recipes;
pub mod session;
pub mod subscriptions;
pub mod users;
