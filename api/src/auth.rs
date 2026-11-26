//! Who a request is.
//!
//! Signing in answers a session token: 32 random bytes, handed out once. The
//! client keeps it (www and admin in a private cookie of their visitor's
//! browser, the app in its own storage) and sends it back as
//! `Authorization: Bearer <token>`. The store only ever holds the token's
//! SHA-256, so nothing read from it signs anyone in. Identity itself, the
//! users, their passwords and their sessions, is `timada-identity`'s; this
//! is the HTTP side of it.
//!
//! A handler says who may call it by what it takes: [`Session`] for anyone
//! signed in, [`Admin`] for an administrator.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngExt;
use serde::Serialize;
use sha2::{Digest, Sha256};
use timada_core::unix_now;
use timada_identity::{
    CodeVerifier, Command, HmacCodeVerifier, LoginContext, RootAccount, UserState,
};
use utoipa::ToSchema;

use crate::cluster::Executor;
use crate::config::AuthConfig;
use crate::error::{ApiError, ApiResult, NOT_ALLOWED, SIGN_IN_REQUIRED, SIGNED_OUT};
use crate::server::AppState;

/// What the one-time codes are derived under, from the configured secret.
const CODE_PURPOSE: &str = "email-code";

/// The most of a `User-Agent` the sign-in history keeps.
const USER_AGENT_MAX_LEN: usize = 512;

/// The command gateway of identity.
pub type Identity = Command<Executor>;

/// What signing in needs besides the store.
pub struct Auth {
    /// Derives the mailed codes and checks them.
    pub codes: Arc<dyn CodeVerifier>,
    pub root: Option<RootAccount>,
    pub session_lifetime_secs: u64,
}

impl Auth {
    /// From the configuration. Without a secret the codes derive from a key
    /// drawn now: fine for a node alone, whose codes then last until it
    /// stops ([`crate::config::Config::load`] refuses a cluster without
    /// one).
    pub fn new(config: &AuthConfig) -> anyhow::Result<Self> {
        let codes = match &config.secret {
            Some(secret) => HmacCodeVerifier::derive(CODE_PURPOSE, secret.expose()),
            None => {
                tracing::warn!(
                    "auth.secret is not set: the codes mailed before a restart stop working"
                );
                HmacCodeVerifier::new(rand::rng().random())
            }
        };
        Ok(Self {
            codes: Arc::new(codes),
            root: config.root_account()?,
            session_lifetime_secs: config.session_lifetime_secs,
        })
    }
}

impl AppState {
    pub fn identity(&self) -> Identity {
        Command::new(self.executor.clone(), self.auth.codes.clone())
    }
}

pub type TokenHash = [u8; 32];

pub fn token_hash(token: &str) -> TokenHash {
    Sha256::digest(token.as_bytes()).into()
}

/// A session just opened, as the client is told of it: the only time the
/// token is said.
#[derive(Serialize, ToSchema)]
pub struct SignedIn {
    pub token: String,
    /// Unix seconds.
    pub expires_at: u64,
    pub user: Account,
}

/// A user as they are told of themself.
#[derive(Serialize, ToSchema)]
pub struct Account {
    pub id: String,
    /// The address they sign in with.
    pub email: String,
    pub name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Admin,
}

impl From<&UserState> for Account {
    fn from(user: &UserState) -> Self {
        Self {
            id: user.id.clone(),
            email: user.account.clone(),
            name: user.label.clone(),
            role: if user.is_admin {
                Role::Admin
            } else {
                Role::User
            },
        }
    }
}

/// Where a sign-in comes from, for the account's history. The address is
/// left out: the api sits behind www, admin and a proxy, and takes nobody's
/// word for it yet.
pub fn login_context(headers: &HeaderMap) -> LoginContext {
    LoginContext {
        user_agent: headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|agent| agent.chars().take(USER_AGENT_MAX_LEN).collect()),
        ip: None,
    }
}

/// Opens a session for `user_id`, who just proved who they are.
pub async fn sign_in(
    state: &AppState,
    user_id: &str,
    context: LoginContext,
) -> ApiResult<SignedIn> {
    let identity = state.identity();
    let token = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let expires_at = unix_now() + state.auth.session_lifetime_secs;
    identity
        .start_session(&token_hash(&token), user_id, expires_at, context)
        .await?;
    let user = identity
        .load_user(user_id)
        .await?
        .ok_or(ApiError::refused(StatusCode::UNAUTHORIZED, SIGNED_OUT))?;
    Ok(SignedIn {
        token,
        expires_at,
        user: Account::from(&user),
    })
}

/// The token a request carries, if any.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

/// The session a request's token opens and the user behind it. Taking it is
/// what makes a route refuse whoever is not signed in.
pub struct Session {
    pub user: UserState,
    pub token_hash: TokenHash,
    /// Unix seconds.
    pub expires_at: u64,
}

impl FromRequestParts<AppState> for Session {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ApiResult<Self> {
        let Some(token) = bearer(&parts.headers) else {
            return Err(ApiError::refused(
                StatusCode::UNAUTHORIZED,
                SIGN_IN_REQUIRED,
            ));
        };
        let token_hash = token_hash(token);
        let identity = state.identity();
        let now = unix_now();
        // Unknown, ended, expired, revoked by a "sign out everywhere", or
        // its user gone: all the same to the client, which forgets it.
        let Some(resolved) = identity.resolve_session(&token_hash, now).await? else {
            return Err(ApiError::refused(StatusCode::UNAUTHORIZED, SIGNED_OUT));
        };

        // Sliding expiration: past half its lifetime, the session is
        // extended for a full one. Best effort: the request goes on with
        // the session as it is whatever happens here.
        let lifetime = state.auth.session_lifetime_secs;
        let mut expires_at = resolved.session.expires_at;
        if expires_at.saturating_sub(now) < lifetime / 2 {
            match identity.refresh_session(&token_hash, now + lifetime).await {
                Ok(true) => expires_at = now + lifetime,
                Ok(false) => {}
                Err(err) => tracing::warn!(%err, "cannot extend the session"),
            }
        }

        Ok(Self {
            user: resolved.user,
            token_hash,
            expires_at,
        })
    }
}

/// The session of an administrator. Taking it is what makes a route refuse
/// everyone else.
pub struct Admin(pub Session);

impl FromRequestParts<AppState> for Admin {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> ApiResult<Self> {
        let session = Session::from_request_parts(parts, state).await?;
        if !session.user.is_admin {
            return Err(ApiError::refused(StatusCode::FORBIDDEN, NOT_ALLOWED));
        }
        Ok(Self(session))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_is_read_from_a_bearer_authorization() {
        let headers = |value: &str| {
            let mut headers = HeaderMap::new();
            headers.insert(AUTHORIZATION, value.parse().unwrap());
            headers
        };
        assert_eq!(bearer(&headers("Bearer abc")), Some("abc"));
        assert_eq!(bearer(&headers("Bearer ")), None);
        assert_eq!(bearer(&headers("Basic abc")), None);
        assert_eq!(bearer(&HeaderMap::new()), None);
    }
}
