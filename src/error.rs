use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use recipe::RecipeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Event store error: {0}")]
    EventStoreError(String),

    #[error("Recipe limit exceeded")]
    RecipeLimitError,

    #[error("Recipe error: {0}")]
    RecipeError(#[from] RecipeError),

    #[error("Meal planning error: {0}")]
    MealPlanningError(#[from] meal_planning::MealPlanningError),

    #[error("Insufficient recipes: need at least {required}, have {current}")]
    InsufficientRecipes { current: usize, required: usize },

    #[error("Concurrent generation in progress")]
    ConcurrentGenerationInProgress,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal server error")]
    InternalError(String),
}

// Manual From implementations for errors that don't have automatic derives
impl From<bincode::error::EncodeError> for AppError {
    fn from(err: bincode::error::EncodeError) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::EventStoreError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerializationError(err.to_string())
    }
}

#[derive(Template)]
#[template(path = "pages/error.html")]
struct ErrorPageTemplate {
    status_code: u16,
    error_title: String,
    error_message: String,
    user: Option<crate::middleware::auth::Auth>,
    error_type: Option<String>, // For custom error styling (e.g., "insufficient_recipes")
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_display = self.to_string();
        let (status_code, error_title, error_message, error_type) = match self {
            AppError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error".to_string(),
                msg,
                None,
            ),
            AppError::RecipeLimitError => (
                StatusCode::FORBIDDEN,
                "Recipe Limit Reached".to_string(),
                "Free tier users are limited to 10 recipes. Please upgrade to premium to create more recipes.".to_string(),
                None,
            ),
            AppError::RecipeError(RecipeError::RecipeLimitReached) => (
                StatusCode::FORBIDDEN,
                "Recipe Limit Reached".to_string(),
                "Free tier users are limited to 10 recipes. Please upgrade to premium to create more recipes.".to_string(),
                None,
            ),
            AppError::RecipeError(RecipeError::NotFound) => (
                StatusCode::NOT_FOUND,
                "Recipe Not Found".to_string(),
                "The requested recipe could not be found.".to_string(),
                None,
            ),
            AppError::RecipeError(RecipeError::PermissionDenied) => (
                StatusCode::FORBIDDEN,
                "Permission Denied".to_string(),
                "You do not have permission to access this recipe.".to_string(),
                None,
            ),
            AppError::RecipeError(RecipeError::ValidationError(msg)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error".to_string(),
                msg,
                None,
            ),
            AppError::RecipeError(RecipeError::AlreadyCopied) => (
                StatusCode::CONFLICT,
                "Recipe Already Copied".to_string(),
                "You have already added this recipe to your library.".to_string(),
                None,
            ),
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                    None,
                )
            }
            AppError::EventStoreError(e) => {
                tracing::error!("Event store error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing your request.".to_string(),
                    None,
                )
            }
            AppError::RecipeError(RecipeError::EventStoreError(e)) => {
                tracing::error!("Event store error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing your request.".to_string(),
                    None,
                )
            }
            AppError::RecipeError(RecipeError::DatabaseError(e)) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                    None,
                )
            }
            AppError::RecipeError(RecipeError::SerializationError(e)) => {
                tracing::error!("Serialization error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing data.".to_string(),
                    None,
                )
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                    None,
                )
            }
            AppError::MealPlanningError(e) => {
                tracing::error!("Meal planning error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Meal Planning Error".to_string(),
                    format!("Failed to generate meal plan: {}", e),
                    None,
                )
            }
            // Story 3.10: AC-5 - Friendly error type for custom styling
            AppError::InsufficientRecipes { current, required } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Not Enough Recipes".to_string(),
                format!(
                    "You need at least {} favorite recipes to generate a meal plan. You currently have {}. Add {} more recipe{} to get started!",
                    required,
                    current,
                    required - current,
                    if required - current > 1 { "s" } else { "" }
                ),
                Some("insufficient_recipes".to_string()),
            ),
            AppError::ConcurrentGenerationInProgress => (
                StatusCode::CONFLICT,
                "Generation In Progress".to_string(),
                "A meal plan generation is already in progress. Please wait for it to complete before starting a new one.".to_string(),
                None,
            ),
            AppError::SerializationError(e) => {
                tracing::error!("Serialization error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing data.".to_string(),
                    None,
                )
            }
        };

        let template = ErrorPageTemplate {
            status_code: status_code.as_u16(),
            error_title,
            error_message,
            user: None, // Can be populated if we have auth context
            error_type,
        };

        match template.render() {
            Ok(html) => (status_code, Html(html)).into_response(),
            Err(e) => {
                tracing::error!("Failed to render error page: {:?}", e);
                (status_code, format!("An error occurred: {}", error_display)).into_response()
            }
        }
    }
}
