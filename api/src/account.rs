//! `/auth/*` and `/account/*`: creating an account, signing in and out, and
//! what a user does with their own account.
//!
//! An address is proven with a mailed code before it creates an account,
//! signs in or gets a new password. Each of the three takes the same two
//! steps on one route: asked without a `code`, it mails one and answers 202;
//! asked again with it, it answers 201 and the session ([`SignedIn`]). The
//! root account is the exception: its configured password signs it in at
//! once, so root gets in while no mail leaves.

use std::collections::HashMap;
use std::sync::Mutex;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use timada_core::unix_now;
use timada_identity::{
    ChallengePurpose, ChangePassword, IdentityError, PASSWORD_MAX_LEN, PASSWORD_MIN_LEN,
    RegisterWithPassword, ResetPassword, SignInRoot, SignInWithPassword, hash_password,
    normalize_email,
};

use crate::auth::{Account, Session, login_context, sign_in, token_hash};
use crate::error::{ApiError, ApiResult, Body, INVALID_REQUEST};
use crate::server::AppState;

/// The refusal when an address tried too many passwords.
pub const TOO_MANY_SIGN_INS: &str = "too_many_sign_ins";
pub const INVALID_NAME: &str = "invalid_name";

/// The longest name an account shows.
const NAME_MAX_LEN: usize = 255;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/sign-in", post(sign_in_with_password))
        .route("/auth/password-reset", post(reset_password))
        .route("/auth/sign-out", post(sign_out))
        .route("/account", get(account))
        .route("/account/profile", put(update_profile))
        .route("/account/password", put(change_password))
        .route("/account/sign-out-everywhere", post(sign_out_everywhere))
}

/// 202: the code is on its way, the same request with it finishes.
fn code_sent() -> Response {
    (StatusCode::ACCEPTED, Json(json!({ "status": "code_sent" }))).into_response()
}

fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, Json(body)).into_response()
}

/// Whether the password fits the policy, said before a code is mailed for
/// nothing; hashing it checks again.
fn check_password_policy(password: &str) -> Result<(), IdentityError> {
    if (PASSWORD_MIN_LEN..=PASSWORD_MAX_LEN).contains(&password.chars().count()) {
        Ok(())
    } else {
        Err(IdentityError::WeakPassword)
    }
}

#[derive(Deserialize)]
struct Credentials {
    email: String,
    password: String,
    code: Option<String>,
}

/// `POST /auth/register`: an account from an address and a password. The
/// new user is signed in.
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Body(form): Body<Credentials>,
) -> ApiResult<Response> {
    let identity = state.identity();
    let email = normalize_email(&form.email)?;
    // Root's address is taken, though no account claimed it.
    if state
        .auth
        .root
        .as_ref()
        .is_some_and(|root| root.has_email(&email))
    {
        return Err(IdentityError::AlreadyRegistered.into());
    }

    let Some(code) = form.code else {
        if identity.load_email_account(&email).await?.is_some() {
            return Err(IdentityError::AlreadyRegistered.into());
        }
        check_password_policy(&form.password)?;
        identity
            .issue_email_code(&email, ChallengePurpose::Register, unix_now())
            .await?;
        return Ok(code_sent());
    };

    let context = login_context(&headers);
    let registered = identity
        .register_with_password(
            RegisterWithPassword {
                email,
                code,
                password_hash: hash_password(&form.password).await?,
                context: context.clone(),
            },
            unix_now(),
        )
        .await?;
    Ok(created(
        sign_in(&state, &registered.user_id, context).await?,
    ))
}

/// `POST /auth/sign-in`: the password is checked at both steps, the mailed
/// code finishes the second. An address is counted before its password is
/// looked at, so a password cannot be guessed by trying.
async fn sign_in_with_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Body(form): Body<Credentials>,
) -> ApiResult<Response> {
    if !state.sign_ins.allow(&form.email, unix_now()) {
        tracing::info!("password sign-in refused: too many attempts");
        return Err(ApiError::refused(
            StatusCode::TOO_MANY_REQUESTS,
            TOO_MANY_SIGN_INS,
        ));
    }
    let identity = state.identity();
    let context = login_context(&headers);

    if let Some(root) = &state.auth.root
        && root.has_email(&form.email)
    {
        let done = identity
            .sign_in_root(
                root,
                SignInRoot {
                    email: form.email,
                    password: form.password,
                    session_user_id: None,
                    context: context.clone(),
                },
            )
            .await?;
        return Ok(created(sign_in(&state, &done.user_id, context).await?));
    }

    let owner = identity.check_password(&form.email, &form.password).await?;
    // Valid: `check_password` normalized it already.
    let email = normalize_email(&form.email)?;
    let Some(code) = form.code else {
        identity
            .issue_email_code(&email, ChallengePurpose::SignIn, unix_now())
            .await?;
        return Ok(code_sent());
    };

    let done = identity
        .sign_in_with_password(
            SignInWithPassword {
                email,
                user_id: owner.user_id,
                code,
                session_user_id: None,
                context: context.clone(),
            },
            unix_now(),
        )
        .await?;
    Ok(created(sign_in(&state, &done.user_id, context).await?))
}

#[derive(Deserialize)]
struct Reset {
    email: String,
    /// The new password, with the code.
    password: Option<String>,
    code: Option<String>,
}

/// `POST /auth/password-reset`: a new password for whoever reads the
/// address. The first step answers the same for an address nobody
/// registered, so it tells nothing about who has an account. The second
/// ends every session of the user and signs them in.
async fn reset_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Body(form): Body<Reset>,
) -> ApiResult<Response> {
    let identity = state.identity();
    let email = normalize_email(&form.email)?;

    let Some(code) = form.code else {
        match identity
            .issue_email_code(&email, ChallengePurpose::ResetPassword, unix_now())
            .await
        {
            Ok(_) | Err(IdentityError::NotRegistered) => return Ok(code_sent()),
            Err(err) => return Err(err.into()),
        }
    };
    let Some(password) = form.password else {
        return Err(ApiError::refused(
            StatusCode::UNPROCESSABLE_ENTITY,
            INVALID_REQUEST,
        ));
    };

    let context = login_context(&headers);
    let done = identity
        .reset_password(
            ResetPassword {
                email,
                code,
                password_hash: hash_password(&password).await?,
                context: context.clone(),
            },
            unix_now(),
        )
        .await?;
    let signed_in = sign_in(&state, &done.user_id, context).await?;
    // The reset itself ended every session of the user. The new one is
    // spared by name, in case it started within the reset's millisecond;
    // should that fail, the client signs in again.
    if let Err(err) = identity
        .revoke_sessions(
            &done.user_id,
            &done.user_id,
            Some(&token_hash(&signed_in.token)),
        )
        .await
    {
        tracing::warn!(%err, "cannot spare the session a password reset started");
    }
    Ok(created(signed_in))
}

/// `POST /auth/sign-out`: ends the session the request's token opened.
async fn sign_out(State(state): State<AppState>, session: Session) -> ApiResult<StatusCode> {
    state.identity().end_session(&session.token_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Who a token signs in, and until when (unix seconds) if it is not used
/// again.
#[derive(Serialize)]
struct Me {
    expires_at: u64,
    user: Account,
}

/// `GET /account`
async fn account(session: Session) -> Json<Me> {
    Json(Me {
        expires_at: session.expires_at,
        user: Account::from(&session.user),
    })
}

#[derive(Deserialize)]
struct Profile {
    name: String,
}

/// `PUT /account/profile`: the name the account shows.
async fn update_profile(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<Profile>,
) -> ApiResult<StatusCode> {
    let name = form.name.trim();
    if name.is_empty() || name.len() > NAME_MAX_LEN {
        return Err(ApiError::refused(
            StatusCode::UNPROCESSABLE_ENTITY,
            INVALID_NAME,
        ));
    }
    state
        .identity()
        .update_profile(&session.user.id, &session.user.account, name)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct NewPassword {
    current_password: String,
    password: String,
}

/// `PUT /account/password`: a new password for who knows the current one.
/// Every other session of the user ends; this one stays signed in.
async fn change_password(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<NewPassword>,
) -> ApiResult<StatusCode> {
    state
        .identity()
        .change_password(ChangePassword {
            user_id: session.user.id,
            current_password: form.current_password,
            new_password_hash: hash_password(&form.password).await?,
            session_token_hash: Some(session.token_hash),
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /account/sign-out-everywhere`: ends every other session of the
/// user; this one stays signed in.
async fn sign_out_everywhere(
    State(state): State<AppState>,
    session: Session,
) -> ApiResult<StatusCode> {
    state
        .identity()
        .revoke_sessions(
            &session.user.id,
            &session.user.id,
            Some(&session.token_hash),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// How many passwords an address may try within [`SIGN_IN_WINDOW_SECS`].
const SIGN_IN_ATTEMPTS: u32 = 10;
const SIGN_IN_WINDOW_SECS: u64 = 15 * 60;
/// Past this many addresses, the windows that ended are forgotten.
const SIGN_IN_PRUNE_AT: usize = 10_000;

/// Counts the password sign-ins tried per address, on this node. Every try
/// counts, right or wrong: the code a right password is followed by is what
/// a person waits on, not this.
#[derive(Default)]
pub struct SignInAttempts {
    /// By address as typed, trimmed and lowercased: the tries, and when
    /// (unix seconds) their window started.
    windows: Mutex<HashMap<String, (u32, u64)>>,
}

impl SignInAttempts {
    /// Counts a try for `email` at `now` (unix seconds) and says whether it
    /// may be made.
    pub fn allow(&self, email: &str, now: u64) -> bool {
        let mut windows = self.windows.lock().unwrap_or_else(|err| err.into_inner());
        if windows.len() >= SIGN_IN_PRUNE_AT {
            windows.retain(|_, (_, started)| now < *started + SIGN_IN_WINDOW_SECS);
        }
        let (tries, started) = windows
            .entry(email.trim().to_lowercase())
            .or_insert((0, now));
        if now >= *started + SIGN_IN_WINDOW_SECS {
            (*tries, *started) = (0, now);
        }
        *tries += 1;
        *tries <= SIGN_IN_ATTEMPTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_tries_so_many_passwords_a_window() {
        let attempts = SignInAttempts::default();
        for _ in 0..SIGN_IN_ATTEMPTS {
            assert!(attempts.allow("alice@example.com", 1_000));
        }
        assert!(!attempts.allow(" Alice@Example.com ", 1_000));
        assert!(attempts.allow("bob@example.com", 1_000), "counted apart");
        assert!(!attempts.allow("alice@example.com", 1_000 + SIGN_IN_WINDOW_SECS - 1));
        assert!(attempts.allow("alice@example.com", 1_000 + SIGN_IN_WINDOW_SECS));
    }

    #[test]
    fn the_password_policy_is_said_before_a_code_is_mailed() {
        assert!(check_password_policy("correct horse").is_ok());
        assert!(check_password_policy("short").is_err());
        assert!(check_password_policy(&"x".repeat(PASSWORD_MAX_LEN + 1)).is_err());
    }
}
