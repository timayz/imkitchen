//! User bounded context for imkitchen application
//!
//! This crate handles user management including registration, authentication,
//! profile management, and admin operations.

pub mod aggregate;
pub mod command;
pub mod event;

// Re-export commonly used types
pub use aggregate::User;
pub use command::{Command, LoginUserInput, RegisterUserInput};
pub use event::EventMetadata;
