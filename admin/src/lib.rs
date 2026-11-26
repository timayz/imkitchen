//! imkitchen admin: the administration of imkitchen.
//!
//! admin owns no aggregate and no command. What it shows of imkitchen it
//! derives from the api's events, followed through the api's listener for
//! the sidecars ([`events`]) into a store of its own ([`db`]).

pub mod app;
pub mod config;
pub mod db;
pub mod events;
pub mod subscriptions;
pub mod users;
