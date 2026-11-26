//! imkitchen www: the public and landing pages.
//!
//! www owns no aggregate and no command. What it shows of imkitchen it
//! derives from the api's events, followed through the api's listener for
//! the sidecars ([`events`]) into a store of its own ([`db`]); what it
//! writes, it sends to the api as a command over HTTP ([`api`]).

pub mod api;
pub mod app;
pub mod config;
pub mod db;
pub mod events;
pub mod recipes;
pub mod subscriptions;
