//! Framework-agnostic application services shared by the HTML routes
//! (askama + twinspark today, topcoat tomorrow) and the JSON API
//! (`web/api`). They take `&AppState` and plain inputs, return domain
//! results, and never touch axum request or response types.

pub mod auth;
pub mod grocery;
pub mod kitchen;
pub mod recipe;
pub mod settings;
