use axum::Form;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum_extra::extract::CookieJar;
use imkitchen_shared::Metadata;
use imkitchen_user::LoginInput;
use serde::Deserialize;

use crate::auth::build_cookie;
use crate::filters;
use crate::server::AppState;
use crate::template::{SERVER_ERROR_MESSAGE, Template};

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
    jar: CookieJar,
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
        Ok(id) => {
            let auth_cookie = match build_cookie(state.config.jwt, id) {
                Ok(cookie) => cookie,
                Err(e) => {
                    tracing::error!("{e}");

                    return template
                        .render(LoginTemplate {
                            email: Some(input.email),
                            password: Some(input.password),
                            error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                        })
                        .into_response();
                }
            };

            let jar = jar.add(auth_cookie);

            let mut resp = Html("").into_response();
            resp.headers_mut()
                .insert("ts-location", "/".parse().unwrap());

            (jar, resp).into_response()
        }
        Err(imkitchen_shared::Error::Unknown(e)) => {
            tracing::error!("{e}");

            template
                .render(LoginTemplate {
                    email: Some(input.email),
                    password: Some(input.password),
                    error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                })
                .into_response()
        }
        Err(e) => template
            .render(LoginTemplate {
                email: Some(input.email),
                password: Some(input.password),
                error_message: Some(e.to_string()),
            })
            .into_response(),
    }
}
