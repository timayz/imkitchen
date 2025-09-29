pub mod commands;
pub mod domain;
pub mod event_store;
pub mod events;
pub mod projections;
pub mod queries;
pub mod services;

// Re-export main types
pub use commands::*;
pub use domain::*;
pub use event_store::*;
pub use events::*;

// Re-export projections (read models)
pub use projections::{
    MaintenanceConfig, MaintenanceStats, ProjectionCacheInfo, ProjectionMaintenanceManager,
    UserPreferencesProjectionBuilder, UserProfileProjectionBuilder,
};

// Re-export projection views
pub use projections::{
    UserPreferencesView as UserPreferencesProjection, UserProfileView as UserProfileProjection,
};

// Re-export query handlers and query-specific views
pub use queries::*;

pub use services::*;
