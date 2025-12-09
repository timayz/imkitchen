use axum::Form;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;
use imkitchen_shared::Metadata;
use imkitchen_user::LoginInput;
use serde::Deserialize;

use crate::auth::{self, build_cookie};
use crate::routes::AppState;
use crate::template::{SERVER_ERROR_MESSAGE, Template};
use crate::template::{ToastErrorTemplate, filters};

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub email: Option<String>,
    pub password: Option<String>,
}

pub async fn page(template: Template) -> impl IntoResponse {
    template.render(LoginTemplate {
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
    template: Template,
    State(state): State<AppState>,
    jar: CookieJar,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    let id = crate::try_response!(
        state.user_command.login(
            LoginInput {
                email: input.email,
                password: input.password,
                lang: template.preferred_language_iso.to_owned(),
                timezone: template.timezone.to_owned(),
            },
            &Metadata::default(),
        ),
        template
    );

    let auth_cookie = match build_cookie(state.config.jwt, id) {
        Ok(cookie) => cookie,
        Err(e) => {
            tracing::error!("{e}");

            return template
                .render(ToastErrorTemplate {
                    original: None,
                    message: SERVER_ERROR_MESSAGE,
                    description: None,
                })
                .into_response();
        }
    };

    let jar = jar.add(auth_cookie);

    (jar, Redirect::to("/")).into_response()
}

pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let jar = jar.remove(auth::remove_cookie());

    (jar, Redirect::to("/"))
}
