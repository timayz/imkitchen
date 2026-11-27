//! imkitchen app: the application in a browser, on a desktop as on a phone.
//!
//! app owns no aggregate and no command. Its pages are read from read
//! models of its own, never from the api: it derives them from the api's
//! events, followed through the api's listener for the sidecars
//! ([`events`]) into a store of its own ([`db`]).
//!
//! It owns no account either, and writes nothing itself: who reads it, the
//! api says, and what they do is a command of the api's, sent over HTTP
//! ([`api`]). What a browser keeps of its session is two cookies
//! ([`session`]).
//!
//! It shares its address with www: the gateway in front of imkitchen sends
//! the main domain to app when the browser keeps the marker of a session,
//! and to www when it does not. What app serves whoever asks is under two
//! prefixes the gateway routes to it either way: `/auth`, the pages of a
//! visitor, and `/_app`, what is not a page ([`assets`], the health).

pub mod api;
pub mod app;
pub mod assets;
pub mod config;
pub mod db;
pub mod dishes;
pub mod events;
pub mod groceries;
pub mod list;
pub mod recipes;
pub mod session;
pub mod settings;
pub mod subscriptions;
