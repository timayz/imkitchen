// CQRS queries for user data

pub mod email_queries;
pub mod user_queries;

// Re-export email query handlers
pub use email_queries::*;

// Re-export user query handlers
pub use user_queries::*;
