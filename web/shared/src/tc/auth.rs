use imkitchen_identity::login::Login;
use topcoat::{
    Result,
    context::{Cx, memoize},
    cookie::{Cookie, Cookies, SameSite, cookies},
    router::{
        error::{forbidden, see_other},
        request::headers,
    },
};

use super::context::{config, state};
use crate::auth::{AUTH_COOKIE, decode_claims, encode_token, resolve_login};

/// The logged-in user, resolved from the same `auth_token` cookie and exact
/// `User-Agent` match as the axum [`AuthUser`](crate::auth::AuthUser)
/// extractor, so a session issued by either stack works on both.
#[memoize(as_ref)]
pub async fn current_user(cx: &Cx) -> Option<Login> {
    let token = cookies(cx).get(AUTH_COOKIE)?;
    let claims = decode_claims(&config(cx).jwt, token.value())?;
    let user_agent = headers(cx).get("user-agent")?.to_str().ok()?;

    resolve_login(state(cx), &claims, user_agent).await
}

/// 303 to `/login` when anonymous. Pages, shards and procedures all start with
/// this (or a stricter guard): page and layout guards do not run for shard and
/// procedure endpoints.
pub async fn require_user(cx: &Cx) -> Result<&Login> {
    current_user(cx)
        .await
        .ok_or_else(|| see_other("/login").into())
}

pub async fn require_chef(cx: &Cx) -> Result<&Login> {
    let user = require_user(cx).await?;
    if !user.is_chef() {
        return Err(forbidden().into());
    }
    Ok(user)
}

pub async fn require_admin(cx: &Cx) -> Result<&Login> {
    let user = require_user(cx).await?;
    if !user.is_admin() {
        return Err(forbidden().into());
    }
    Ok(user)
}

/// Queues the `auth_token` cookie with the same attributes as
/// [`build_cookie`](crate::auth::build_cookie).
pub fn set_auth_cookie(cx: &Cx, sub: String, acc: String) -> anyhow::Result<()> {
    let (token, expires_at) = encode_token(&config(cx).jwt, sub, acc)?;

    cookies(cx).add(
        Cookie::build((AUTH_COOKIE, token))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .expires(expires_at)
            .build(),
    );

    Ok(())
}
