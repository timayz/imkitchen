use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use imkitchen_identity::RegisterInput;
use serde::Deserialize;

use imkitchen_web_shared::template::{ToastErrorTemplate, filters};
use imkitchen_web_shared::{AppState, template::Template};

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(RegisterTemplate {
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
    template: Template,
    State(app): State<AppState>,
    Form(mut input): Form<ActionInput>,
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

    if input.email == app.config.root.email {
        input.password = app.config.root.password;
    }

    let id = imkitchen_web_shared::try_response!(
        app.identity.register(RegisterInput {
            email: input.email.to_owned(),
            password: input.password.to_owned(),
            lang: template.preferred_language_iso.to_owned(),
            timezone: template.timezone.to_owned(),
        },),
        template
    );

    if input.email != app.config.root.email {
        return Redirect::to("/login").into_response();
    }

    imkitchen_web_shared::try_response!(app.identity.made_admin(&id), template);

    Redirect::to("/login").into_response()
}
