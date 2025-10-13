use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// UserCreated event emitted when a new user registers
///
/// This event is the source of truth for user creation in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
///
/// Note: user_id duplicates aggregator_id for convenience in projections
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct UserCreated {
    pub user_id: String, // UUID as string (matches aggregator_id)
    pub email: String,
    pub password_hash: String,
    pub created_at: String, // RFC3339 formatted timestamp
}

/// PasswordChanged event emitted when a user successfully resets their password
///
/// This event records password changes in the audit trail. The old password is NOT stored
/// for security reasons - only the new hashed password is recorded.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct PasswordChanged {
    pub user_id: String,       // UUID as string (matches aggregator_id)
    pub password_hash: String, // New Argon2 hashed password
    pub changed_at: String,    // RFC3339 formatted timestamp
}
