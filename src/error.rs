use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use recipe::RecipeError;
use shopping::ShoppingListError;
use thiserror::Error;
use user::error::UserError;

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

    #[error("Shopping list error: {0}")]
    ShoppingListError(#[from] ShoppingListError),

    #[error("Notification error: {0}")]
    NotificationError(#[from] notifications::commands::NotificationError),

    #[error("User error: {0}")]
    UserError(#[from] UserError),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Notification not found")]
    NotificationNotFound,

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
    current_path: String,
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
            // Story 4.3: Week validation errors (AC #7, #5)
            AppError::ShoppingListError(ShoppingListError::PastWeekNotAccessibleError) => (
                StatusCode::NOT_FOUND,
                "Past Week Not Accessible".to_string(),
                "Past weeks are not accessible in the current version. Please select the current week or a future week (up to 4 weeks ahead).".to_string(),
                None,
            ),
            AppError::ShoppingListError(ShoppingListError::FutureWeekOutOfRangeError) => (
                StatusCode::BAD_REQUEST,
                "Future Week Out of Range".to_string(),
                "You can only access shopping lists up to 4 weeks ahead. Please select a week within the allowed range.".to_string(),
                None,
            ),
            AppError::ShoppingListError(ShoppingListError::InvalidWeekError(msg)) => (
                StatusCode::BAD_REQUEST,
                "Invalid Week".to_string(),
                format!("Invalid week parameter: {}", msg),
                None,
            ),
            AppError::ShoppingListError(e) => {
                tracing::error!("Shopping list error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Shopping List Error".to_string(),
                    format!("An error occurred while processing your shopping list: {}", e),
                    None,
                )
            }
            AppError::NotificationError(e) => {
                tracing::error!("Notification error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Notification Error".to_string(),
                    format!("An error occurred while processing your notification: {}", e),
                    None,
                )
            }
            AppError::UserError(e) => {
                tracing::error!("User error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "User Error".to_string(),
                    format!("An error occurred: {}", e),
                    None,
                )
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "Bad Request".to_string(),
                msg,
                None,
            ),
            AppError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                "Permission Denied".to_string(),
                "You do not have permission to access this resource.".to_string(),
                None,
            ),
            AppError::NotificationNotFound => (
                StatusCode::NOT_FOUND,
                "Notification Not Found".to_string(),
                "The requested notification could not be found.".to_string(),
                None,
            ),
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
            current_path: String::new(), // Error pages don't have meaningful path context
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
