use thiserror::Error;

/// Domain-specific errors for user operations
///
/// These errors represent business logic failures that should be
/// handled explicitly in the application layer (e.g., showing specific
/// error messages to users).
#[derive(Debug, Error)]
pub enum UserError {
    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password must be at least 8 characters")]
    PasswordTooShort,

    #[error("Password hashing failed")]
    HashingError(String),

    #[error("Password verification failed")]
    VerificationError,

    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Event store error")]
    EventStoreError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Result type for user operations that may fail with UserError
pub type UserResult<T> = Result<T, UserError>;
