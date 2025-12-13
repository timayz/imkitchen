use axum::{
    Form,
    extract::{Path, State},
    response::IntoResponse,
};
use imkitchen_shared::Metadata;
use imkitchen_user::reset_password::{RequestInput, ResetInput};
use serde::Deserialize;

use crate::{
    routes::AppState,
    template::{Template, ToastErrorTemplate, filters},
};

#[derive(askama::Template)]
#[template(path = "partials/reset-password-check.html")]
pub struct ResetPasswordCheckTemplate {
    pub email: String,
}

#[derive(askama::Template)]
#[template(path = "reset-password.html")]
pub struct ResetPasswordTemplate;

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(ResetPasswordTemplate)
}

#[derive(Deserialize)]
pub struct ActionInput {
    pub email: String,
}

pub async fn action(
    template: Template,
    State(app): State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    if let Some(user) =
        crate::try_response!(anyhow: app.user_command.find_by_email(&input.email), template)
    {
        crate::try_response!(
            app.user_reset_password_command.request(
                RequestInput {
                    email: input.email.to_owned(),
                    lang: template.preferred_language_iso.to_owned(),
                    host: app.config.server.url,
                },
                &Metadata::by(user.id),
            ),
            template
        );
    };

    template.render(ResetPasswordCheckTemplate { email: input.email })
}

#[derive(askama::Template)]
#[template(path = "partials/reset-password-success.html")]
pub struct ResetPasswordSucessTemplate;

#[derive(askama::Template)]
#[template(path = "reset-password-new.html")]
pub struct ResetPasswordNewTemplate {
    pub id: String,
}

pub async fn new_page(template: Template, Path((id,)): Path<(String,)>) -> impl IntoResponse {
    template.render(ResetPasswordNewTemplate { id })
}

#[derive(Deserialize)]
pub struct NewActionInput {
    pub password: String,
    pub confirm_password: String,
}

pub async fn new_action(
    template: Template,
    State(app): State<AppState>,
    Path((id,)): Path<(String,)>,
    Form(input): Form<NewActionInput>,
) -> impl IntoResponse {
    if input.password != input.confirm_password {
        return (
            [("ts-swap", "skip")],
            template.render(ToastErrorTemplate {
                original: None,
                message: "Passwords don't match. Please make sure both fields are identical.",
                description: None,
            }),
        )
            .into_response();
    }

    crate::try_response!(opt: app.user_reset_password_command.reset(ResetInput{id, password: input.password}), template);

    template.render(ResetPasswordSucessTemplate).into_response()
}
