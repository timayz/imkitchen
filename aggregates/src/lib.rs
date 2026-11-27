//! The aggregates of imkitchen and their events: what the api writes, and
//! what www, admin and app build their read models from.
//!
//! The api is the only one that writes them. A sidecar follows them through
//! the api's listener (`[sidecars]` in the api's configuration), which serves
//! each one the aggregates [`Sidecar::reads`] lists and nothing else.
//!
//! Every variant and every field of an event is frozen once it ships
//! (bitcode is positional): evolve by adding variants.

pub mod contact;
pub mod groceries;
pub mod identity;
pub mod image;
pub mod list;
pub mod plat;
pub mod recipe;
pub mod recipe_import;
pub mod settings;
mod sidecar;
pub mod suspension;
pub mod tutorial;

pub use sidecar::Sidecar;
