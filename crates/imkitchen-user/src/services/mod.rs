// Services for user domain operations

pub mod login_service;
pub mod profile_service;
pub mod recommendation_service;

// Re-export services
pub use login_service::*;
pub use profile_service::*;
pub use recommendation_service::*;
