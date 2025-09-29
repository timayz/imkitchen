// Async validation handlers for TwinSpark integration

use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    Form,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use imkitchen_user::commands::{
    CheckEmailExistsCommand, EmailValidationService, ValidateUsernameAvailabilityCommand,
};

/// Query parameters for async email validation
#[derive(Debug, Deserialize)]
pub struct EmailValidationQuery {
    pub email: String,
    #[serde(rename = "ts-req")]
    pub ts_req: Option<String>,
}

/// Query parameters for async username validation
#[derive(Debug, Deserialize)]
pub struct UsernameValidationQuery {
    pub username: String,
    #[serde(rename = "ts-req")]
    pub ts_req: Option<String>,
}

/// Form data for email validation
#[derive(Debug, Deserialize)]
pub struct EmailValidationForm {
    pub email: String,
}

/// Template for email validation response fragment
#[derive(Template)]
#[template(path = "fragments/email_validation.html")]
pub struct EmailValidationFragment {
    pub email: String,
    pub exists: bool,
    pub error: String,
    pub has_error: bool,
    pub suggestions: Vec<String>,
}

/// Template for username validation response fragment
#[derive(Template)]
#[template(path = "fragments/username_validation.html")]
pub struct UsernameValidationFragment {
    pub username: String,
    pub available: bool,
    pub error: String,
    pub has_error: bool,
    pub suggestions: Vec<String>,
}

/// Async email existence validation endpoint
/// This is called by TwinSpark with ts-req parameter for async validation
pub async fn validate_email_async(
    State(db_pool): State<SqlitePool>,
    Query(query): Query<EmailValidationQuery>,
) -> impl IntoResponse {
    let validation_service = EmailValidationService::new(db_pool);
    let command = CheckEmailExistsCommand::new(query.email.clone());

    match validation_service.handle_email_exists_check(command).await {
        Ok(response) => {
            let fragment = EmailValidationFragment {
                email: response.email,
                exists: response.exists,
                error: String::new(),
                has_error: false,
                suggestions: vec![],
            };
            Html(fragment.to_string()).into_response()
        }
        Err(err) => {
            let fragment = EmailValidationFragment {
                email: query.email,
                exists: false,
                error: err.to_string(),
                has_error: true,
                suggestions: vec![],
            };
            Html(fragment.to_string()).into_response()
        }
    }
}

/// Async username availability validation endpoint
/// This is called by TwinSpark with ts-req parameter for async validation
pub async fn validate_username_async(
    State(db_pool): State<SqlitePool>,
    Query(query): Query<UsernameValidationQuery>,
) -> impl IntoResponse {
    let validation_service = EmailValidationService::new(db_pool);
    let command = ValidateUsernameAvailabilityCommand::new(query.username.clone());

    match validation_service
        .handle_username_availability_check(command)
        .await
    {
        Ok(response) => {
            let fragment = UsernameValidationFragment {
                username: response.username,
                available: response.available,
                error: String::new(),
                has_error: false,
                suggestions: response.suggestions,
            };
            Html(fragment.to_string()).into_response()
        }
        Err(err) => {
            let fragment = UsernameValidationFragment {
                username: query.username,
                available: false,
                error: err.to_string(),
                has_error: true,
                suggestions: vec![],
            };
            Html(fragment.to_string()).into_response()
        }
    }
}

/// POST endpoint for email validation (form submission)
pub async fn validate_email_form(
    State(db_pool): State<SqlitePool>,
    Form(form): Form<EmailValidationForm>,
) -> impl IntoResponse {
    let validation_service = EmailValidationService::new(db_pool);
    let command = CheckEmailExistsCommand::new(form.email.clone());

    match validation_service.handle_email_exists_check(command).await {
        Ok(response) => {
            let fragment = EmailValidationFragment {
                email: response.email,
                exists: response.exists,
                error: String::new(),
                has_error: false,
                suggestions: vec![],
            };
            Html(fragment.to_string()).into_response()
        }
        Err(err) => {
            let fragment = EmailValidationFragment {
                email: form.email,
                exists: false,
                error: err.to_string(),
                has_error: true,
                suggestions: vec![],
            };
            Html(fragment.to_string()).into_response()
        }
    }
}