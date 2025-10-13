pub mod aggregate;
pub mod commands;
pub mod error;
pub mod events;
pub mod jwt;
pub mod password;
pub mod read_model;

// Re-export main types
pub use aggregate::UserAggregate;
pub use commands::{register_user, RegisterUserCommand};
pub use error::{UserError, UserResult};
pub use events::UserCreated;
pub use jwt::{generate_jwt, validate_jwt, Claims};
pub use password::{hash_password, verify_password};
pub use read_model::{query_user_by_email, user_projection};
