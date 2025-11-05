use axum::{
    extract::{Form, Path, State},
    response::IntoResponse,
};
use imkitchen_user::{Metadata, RegisterInput};
use serde::Deserialize;

use crate::filters;
use crate::{extract::template::Template, server::AppState};

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub processing: Option<String>,
    pub error_message: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn page(template: Template<RegisterTemplate>) -> impl IntoResponse {
    template.render(RegisterTemplate {
        processing: None,
        error_message: None,
        email: None,
        password: None,
        confirm_password: None,
    })
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
    pub password: String,
    pub confirm_password: String,
}

pub async fn action(
    template: Template<RegisterTemplate>,
    State(state): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    if input.password != input.confirm_password {
        return template.render(RegisterTemplate {
            email: Some(input.email),
            password: Some(input.password),
            confirm_password: Some(input.confirm_password),
            processing: None,
            error_message: Some(
                "Passwords don't match. Please make sure both fields are identical.".to_owned(),
            ),
        });
    }

    match state
        .user_command
        .register(
            RegisterInput {
                email: input.email.to_owned(),
                password: input.password.to_owned(),
            },
            Metadata::default(),
        )
        .await
    {
        Ok(id) => template.render(RegisterTemplate {
            email: Some(input.email),
            password: Some(input.password),
            confirm_password: Some(input.confirm_password),
            processing: Some(id),
            error_message: None,
        }),
        Err(e) => template.render(RegisterTemplate {
            email: Some(input.email),
            password: Some(input.password),
            confirm_password: Some(input.confirm_password),
            processing: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[derive(askama::Template)]
#[template(path = "partials/register-status.html")]
pub struct RegisterStatusTemplate {
    pub id: String,
}

#[derive(askama::Template)]
#[template(path = "partials/register-status-error.html")]
pub struct RegisterStatusErrorTemplate {
    pub error_message: String,
}

pub async fn status(
    template: Template<RegisterStatusTemplate>,
    error_template: Template<RegisterStatusErrorTemplate>,
    State(state): State<AppState>,
    Path((id,)): Path<(String,)>,
) -> impl IntoResponse {
    let user = match state.user_command.load(&id).await {
        Ok(user) => user,
        Err(e) => {
            tracing::error!("{e}");
            return (
            [
                (
                    "ts-swap-push",
                    "replace: #processing-alert <= #processing-error,afterend: #processing-error <= button",
                ),
                ("ts-swap", "skip"),
            ],
                error_template
                .render(RegisterStatusErrorTemplate {
                    error_message: "Something went wrong, please retry later".to_owned(),
                }))
                .into_response();
        }
    };

    match user.item.status {
        imkitchen_user::Status::Idle => ([("ts-location", "/")], "").into_response(),
        imkitchen_user::Status::Failed(reason) => (
            [
                (
                    "ts-swap-push",
                    "replace: #processing-alert <= #processing-error,afterend: #processing-error <= button",
                ),
                ("ts-swap", "skip"),
            ],
            error_template.render(RegisterStatusErrorTemplate {
                error_message: reason,
            }),
        )
            .into_response(),
        imkitchen_user::Status::Processing(_) => template
            .render(RegisterStatusTemplate { id })
            .into_response(),
    }
}
