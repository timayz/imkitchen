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

    #[error("Internal server error")]
    InternalError(String),
}

#[derive(Template)]
#[template(path = "pages/error.html")]
struct ErrorPageTemplate {
    status_code: u16,
    error_title: String,
    error_message: String,
    user: Option<crate::middleware::auth::Auth>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_display = self.to_string();
        let (status_code, error_title, error_message) = match self {
            AppError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error".to_string(),
                msg,
            ),
            AppError::RecipeLimitError => (
                StatusCode::FORBIDDEN,
                "Recipe Limit Reached".to_string(),
                "Free tier users are limited to 10 recipes. Please upgrade to premium to create more recipes.".to_string(),
            ),
            AppError::RecipeError(RecipeError::RecipeLimitReached) => (
                StatusCode::FORBIDDEN,
                "Recipe Limit Reached".to_string(),
                "Free tier users are limited to 10 recipes. Please upgrade to premium to create more recipes.".to_string(),
            ),
            AppError::RecipeError(RecipeError::NotFound) => (
                StatusCode::NOT_FOUND,
                "Recipe Not Found".to_string(),
                "The requested recipe could not be found.".to_string(),
            ),
            AppError::RecipeError(RecipeError::PermissionDenied) => (
                StatusCode::FORBIDDEN,
                "Permission Denied".to_string(),
                "You do not have permission to access this recipe.".to_string(),
            ),
            AppError::RecipeError(RecipeError::ValidationError(msg)) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Validation Error".to_string(),
                msg,
            ),
            AppError::RecipeError(RecipeError::AlreadyCopied) => (
                StatusCode::CONFLICT,
                "Recipe Already Copied".to_string(),
                "You have already added this recipe to your library.".to_string(),
            ),
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                )
            }
            AppError::EventStoreError(e) => {
                tracing::error!("Event store error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing your request.".to_string(),
                )
            }
            AppError::RecipeError(RecipeError::EventStoreError(e)) => {
                tracing::error!("Event store error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing your request.".to_string(),
                )
            }
            AppError::RecipeError(RecipeError::DatabaseError(e)) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                )
            }
            AppError::RecipeError(RecipeError::SerializationError(e)) => {
                tracing::error!("Serialization error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred while processing data.".to_string(),
                )
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                    "An unexpected error occurred. Please try again later.".to_string(),
                )
            }
        };

        let template = ErrorPageTemplate {
            status_code: status_code.as_u16(),
            error_title,
            error_message,
            user: None, // Can be populated if we have auth context
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
