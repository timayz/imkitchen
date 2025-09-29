// User domain events with Evento integration

pub mod dietary_restrictions_changed;
pub mod family_size_changed;
pub mod user_account_deleted;
pub mod user_logged_in;
pub mod user_password_changed;
pub mod user_profile_updated;
pub mod user_registered;

// Re-export all events
pub use dietary_restrictions_changed::*;
pub use family_size_changed::*;
pub use user_account_deleted::*;
pub use user_logged_in::*;
pub use user_password_changed::*;
pub use user_profile_updated::*;
pub use user_registered::*;

/// Unified event enum for user domain events
#[derive(Debug, Clone)]
pub enum UserEvent {
    UserRegistered(UserRegistered),
    FamilySizeChanged(FamilySizeChanged),
    DietaryRestrictionsChanged(DietaryRestrictionsChanged),
    UserProfileUpdated(UserProfileUpdated),
    UserLoggedIn(UserLoggedIn),
    UserPasswordChanged(UserPasswordChanged),
    UserAccountDeleted(UserAccountDeleted),
}
