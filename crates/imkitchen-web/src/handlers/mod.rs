pub mod async_validation;
pub mod auth;
pub mod dashboard;
pub mod health;
pub mod metrics;
pub mod profile;

// Re-export handlers
pub use async_validation::*;
pub use auth::*;
pub use dashboard::*;
pub use health::*;
pub use metrics::*;
pub use profile::*;
