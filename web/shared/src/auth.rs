use std::{
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{
    TypedHeader,
    extract::{
        CookieJar,
        cookie::{Cookie, Expiration, SameSite},
    },
    headers::UserAgent,
};
use imkitchen_identity::types::user::State;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    config::JwtConfig,
    template::{ForbiddenTemplate, Template},
};

const AUTH_COOKIE_NAME: &str = "auth_token";

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    aud: String,
    exp: u64,
    iat: u64,
    iss: String,
    pub sub: String,
    pub acc: String,
}

/// Signs the `auth_token` JWT and returns it with its expiry. Framework-agnostic
/// so both stacks issue the same token; see [`build_cookie`] for the axum cookie.
pub fn encode_token(
    config: &JwtConfig,
    sub: String,
    acc: String,
) -> anyhow::Result<(String, OffsetDateTime)> {
    let now = OffsetDateTime::now_utc();
    let expires_at = now + time::Duration::days(config.expiration_days.into());
    let claims = Claims {
        aud: config.audience.to_owned(),
        exp: expires_at.unix_timestamp().try_into()?,
        iat: now.unix_timestamp().try_into()?,
        iss: config.issuer.to_owned(),
        sub,
        acc,
    };

    let auth_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )?;

    Ok((auth_token, expires_at))
}

pub fn build_cookie<'a>(config: JwtConfig, sub: String, acc: String) -> anyhow::Result<Cookie<'a>> {
    let (auth_token, expires_at) = encode_token(&config, sub, acc)?;

    Ok(Cookie::build((AUTH_COOKIE_NAME, auth_token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .expires(Expiration::from(expires_at))
        .build())
}

pub fn auth_cookie<'a>() -> Cookie<'a> {
    Cookie::from(AUTH_COOKIE_NAME)
}

/// Cookie name of the retired opt-in ad-consent flow. Nothing sets or reads it
/// anymore; only the removal helper below remains so stale cookies get purged.
pub const AD_CONSENT_COOKIE_NAME: &str = "ad_consent";

pub fn ad_consent_cookie<'a>() -> Cookie<'a> {
    Cookie::build(AD_CONSENT_COOKIE_NAME).path("/").build()
}

/// Decodes and validates the `auth_token` JWT. Framework-agnostic so the axum
/// extractors and the topcoat `cx` functions share one implementation.
pub fn decode_claims(config: &JwtConfig, token: &str) -> Option<Claims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(std::slice::from_ref(&config.issuer));
    validation.set_audience(std::slice::from_ref(&config.audience));

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims)
}

/// Resolves the login a set of claims points at: the access must exist for the
/// exact `User-Agent` it was created with, and the account must not be
/// suspended. Framework-agnostic, see [`decode_claims`].
pub async fn resolve_login(
    state: &crate::AppState,
    claims: &Claims,
    user_agent: &str,
) -> Option<imkitchen_identity::login::Login> {
    let user = match state.identity.find_login(&claims.sub).await {
        Ok(user) => user?,
        Err(e) => {
            tracing::error!("{e}");
            return None;
        }
    };

    let mut login = user
        .logins
        .iter()
        .find(|l| l.id == claims.acc && l.user_agent == user_agent)?
        .clone();

    login.id = user.id;

    if login.state == State::Suspended {
        return None;
    }

    if state.config.premium.is_none() || login.is_admin() {
        login.subscription_expire_at = (SystemTime::now() + time::Duration::weeks(10 * 52))
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
    }

    Some(login)
}

pub const AUTH_COOKIE: &str = AUTH_COOKIE_NAME;

#[derive(Clone, Default)]
pub struct AuthToken(Claims);

impl Deref for AuthToken {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::AppState> for AuthToken {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(AuthToken(claims.clone()));
        }

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        let token = jar
            .get(AUTH_COOKIE_NAME)
            .map(|cookie| cookie.value())
            .ok_or(Redirect::to("/login"))?;

        let claims = decode_claims(&state.config.jwt, token).ok_or(Redirect::to("/login"))?;

        parts.extensions.insert(claims.clone());

        Ok(AuthToken(claims))
    }
}

impl FromRequestParts<crate::AppState> for Option<AuthToken> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(AuthToken::from_request_parts(parts, state).await.ok())
    }
}

#[derive(Clone, Default)]
pub struct AuthUser(pub imkitchen_identity::login::Login);

impl AuthUser {
    /// Wraps a `Login` into an `AuthUser` without going through the request
    /// extractor. Used by demo mode to render authenticated pages with
    /// synthetic data.
    pub fn new(login: imkitchen_identity::login::Login) -> Self {
        Self(login)
    }

    /// A synthetic premium "guest" account used to render pages in demo mode —
    /// both the `/demo/*` tour and anonymous views of public recipes. Premium
    /// so feature-gated UI shows without upsell; the subscription is set far in
    /// the future so `is_premium()` holds.
    pub fn demo() -> Self {
        let subscription_expire_at = (SystemTime::now() + time::Duration::weeks(10 * 52))
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());

        Self(imkitchen_identity::login::Login {
            id: "demo".to_owned(),
            email: "chef@imkitchen.app".to_owned(),
            username: Some("demo_chef".to_owned()),
            subscription_expire_at,
            tz: "UTC".to_owned(),
            ..Default::default()
        })
    }
}

impl Deref for AuthUser {
    type Target = imkitchen_identity::login::Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::AppState> for AuthUser {
    type Rejection = Redirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let user_agent = TypedHeader::<UserAgent>::from_request_parts(parts, state)
            .await
            .map_err(|_| Redirect::to("/login"))?;

        let claims = AuthToken::from_request_parts(parts, state).await?;

        let login = resolve_login(state, &claims, &user_agent.to_string())
            .await
            .ok_or(Redirect::to("/login"))?;

        parts.extensions.insert(login.clone());

        Ok(AuthUser(login))
    }
}

impl FromRequestParts<crate::AppState> for Option<AuthUser> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(AuthUser::from_request_parts(parts, state).await.ok())
    }
}

pub struct RequireChef(pub imkitchen_identity::login::Login);

impl Deref for RequireChef {
    type Target = imkitchen_identity::login::Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::AppState> for RequireChef {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        if user.is_chef() {
            return Ok(RequireChef(user));
        }

        let template = Template::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        Err(template.render(ForbiddenTemplate).into_response())
    }
}

pub struct AuthAdmin(imkitchen_identity::login::Login);

impl Deref for AuthAdmin {
    type Target = imkitchen_identity::login::Login;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequestParts<crate::AppState> for AuthAdmin {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &crate::AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        if user.is_admin() {
            return Ok(AuthAdmin(user));
        }

        let template = Template::from_request_parts(parts, state)
            .await
            .expect("Infallible");

        Err(template.render(ForbiddenTemplate).into_response())
    }
}
