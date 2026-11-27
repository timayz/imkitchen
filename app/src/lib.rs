//! imkitchen app: the application in a browser, on a desktop as on a phone.
//!
//! app owns no aggregate and no command. Its pages are read from read
//! models of its own, never from the api: it derives them from the api's
//! events, followed through the api's listener for the sidecars
//! ([`events`]) into a store of its own ([`db`]).

pub mod app;
pub mod config;
pub mod db;
pub mod events;
pub mod recipes;
pub mod subscriptions;
