use axum::Form;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum_extra::TypedHeader;
use axum_extra::extract::CookieJar;
use axum_extra::headers::UserAgent;
use imkitchen_identity::LoginInput;
use serde::Deserialize;

use imkitchen_web_shared::auth::{self, AuthToken, AuthUser, build_cookie};
use imkitchen_web_shared::AppState;
use imkitchen_web_shared::template::{SERVER_ERROR_MESSAGE, Template};
use imkitchen_web_shared::template::{ToastErrorTemplate, filters};

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
    State(app): State<AppState>,
    jar: CookieJar,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    let (user_id, access_id) = imkitchen_web_shared::try_response!(
        app.identity.login(LoginInput {
            email: input.email,
            password: input.password,
            lang: template.preferred_language_iso.to_owned(),
            timezone: template.timezone.to_owned(),
            user_agent: user_agent.to_string(),
        },),
        template
    );

    let auth_cookie = match build_cookie(app.config.jwt, user_id, access_id) {
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

pub async fn logout(
    jar: CookieJar,
    token: AuthToken,
    user: AuthUser,
    template: Template,
    State(app): State<AppState>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(
        app.identity.logout(&user.id, token.sub.to_owned()),
        template
    );

    let jar = jar.remove(auth::auth_cookie());

    (jar, Redirect::to("/")).into_response()
}
