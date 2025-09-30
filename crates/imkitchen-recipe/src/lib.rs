pub mod command_handlers;
pub mod commands;
pub mod domain;
pub mod events;
pub mod projections;
pub mod queries;
pub mod services;

// Re-export main types - using specific imports to avoid ambiguous glob re-exports
pub use command_handlers::{
    discovery as discovery_command_handlers,
    review_moderation::*,
};
pub use commands::{
    discovery as discovery_commands,
};
pub use domain::{
    collection::*,
    rating::*,
    // Note: discovery domain is accessible via discovery module
};
// Note: events module contains submodules that may conflict with domain names
// pub use events::*;
pub use projections::{
    discovery as discovery_projections,
};
pub use queries::{
    discovery as discovery_queries,
};
pub use services::{
    search::*,
    popularity::*,
    recommendation::*,
    discovery_data::*,
};
