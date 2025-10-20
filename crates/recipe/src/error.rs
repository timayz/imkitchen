use thiserror::Error;

pub type RecipeResult<T> = Result<T, RecipeError>;

#[derive(Error, Debug)]
pub enum RecipeError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Recipe not found")]
    NotFound,

    #[error("Recipe limit reached - free tier users are limited to 10 recipes. Upgrade to premium for unlimited recipes.")]
    RecipeLimitReached,

    #[error("Permission denied - you do not own this recipe")]
    PermissionDenied,

    #[error("Recipe already copied - you have already added this recipe to your library")]
    AlreadyCopied,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Event store error: {0}")]
    EventStoreError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
