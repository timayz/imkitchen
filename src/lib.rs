//! ImKitchen - Event-driven meal planning application
//!
//! This crate contains shared application types, server implementation,
//! and database migration utilities.

pub mod assets;
pub mod auth;
pub mod config;
pub mod queries;
pub mod routes;

pub use config::Config;
