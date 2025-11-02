//! User events

use bincode::{Decode, Encode};

/// Event metadata containing user context and request tracing
#[derive(Encode, Decode, Clone)]
pub struct EventMetadata {
    /// Optional user ID who triggered the event
    pub user_id: Option<String>,
    /// Unique request ID (ULID) for tracing event chains
    pub request_id: String,
}

/// User registration initiated
#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserRegistered {
    pub email: String,
    pub hashed_password: String,
}

/// User registration succeeded after validation
#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserRegistrationSucceeded {
    pub email: String,
    pub hashed_password: String,
}

/// User registration failed during validation
#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserRegistrationFailed {
    pub error: String,
}

/// User logged in successfully
#[derive(evento::AggregatorName, Encode, Decode)]
pub struct UserLoggedIn {
    // Timestamp is handled automatically by Evento
}
