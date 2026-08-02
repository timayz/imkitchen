use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::CookieJar;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::{self, AuthUser, build_ad_consent_cookie};
use imkitchen_web_shared::template::Template;

fn back_url(headers: &HeaderMap) -> &str {
    headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("/")
}

pub async fn consent(
    jar: CookieJar,
    user: Option<AuthUser>,
    template: Template,
    State(app): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Some(user) = &user {
        imkitchen_web_shared::try_response!(app.identity.grant_ad_consent(&user.id), template);
    }

    let jar = jar.add(build_ad_consent_cookie());

    (jar, Redirect::to(back_url(&headers))).into_response()
}

pub async fn revoke(
    jar: CookieJar,
    user: AuthUser,
    template: Template,
    State(app): State<AppState>,
) -> impl IntoResponse {
    imkitchen_web_shared::try_response!(app.identity.revoke_ad_consent(&user.id), template);

    let jar = jar.remove(auth::ad_consent_cookie());

    (jar, Redirect::to("/settings/billing")).into_response()
}
