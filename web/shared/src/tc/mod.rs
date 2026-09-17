//! topcoat side of the shared web layer.
//!
//! Lives next to the askama [`template`](crate::template) wrapper while routes
//! migrate crate by crate; both stacks read the same config, i18n catalog and
//! `auth_token` cookie.

pub mod auth;
pub mod context;
pub mod error;
pub mod pages;
pub mod router;
pub mod view;
