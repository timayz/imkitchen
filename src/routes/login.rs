use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use imkitchen_user::{LoginInput, Metadata};
use serde::Deserialize;

use crate::extract::template::Template;
use crate::filters;
use crate::server::AppState;

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub error_message: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn page(template: Template<LoginTemplate>) -> impl IntoResponse {
    template.render(LoginTemplate {
        error_message: None,
        email: None,
        password: None,
    })
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
    pub password: String,
}

pub async fn action(
    template: Template<LoginTemplate>,
    State(state): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    match state
        .user_command
        .login(
            LoginInput {
                email: input.email.to_owned(),
                password: input.password.to_owned(),
                lang: "en".to_owned(),
            },
            Metadata::default(),
        )
        .await
    {
        Ok(_id) => ([("ts-location", "/")], "").into_response(),
        Err(e) => template
            .render(LoginTemplate {
                email: Some(input.email),
                password: Some(input.password),
                error_message: Some(e.to_string()),
            })
            .into_response(),
    }
}
