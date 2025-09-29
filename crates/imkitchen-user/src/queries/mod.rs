// CQRS queries for user data

pub mod email_queries;
pub mod user_queries;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export email query handlers
pub use email_queries::*;

// Re-export user query handlers
pub use user_queries::*;
