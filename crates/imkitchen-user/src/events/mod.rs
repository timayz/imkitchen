// User domain events with Evento integration

pub mod user_logged_in;
pub mod user_password_changed;
pub mod user_profile_updated;
pub mod user_registered;

// Re-export all events
pub use user_logged_in::*;
pub use user_password_changed::*;
pub use user_profile_updated::*;
pub use user_registered::*;
