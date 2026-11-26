//! `/v1/auth/*` and `/v1/account/*`: creating an account, signing in and out, and
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

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use timada_core::unix_now;
use timada_identity::{
    ChallengePurpose, ChangePassword, IdentityError, PASSWORD_MAX_LEN, PASSWORD_MIN_LEN,
    RegisterWithPassword, ResetPassword, SignInRoot, SignInWithPassword, hash_password,
    normalize_email,
};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use validator::{Validate, ValidationError};

use crate::auth::{Account, Session, SignedIn, login_context, sign_in, token_hash};
use crate::error::{ApiError, ApiResult, Body, INVALID_REQUEST, trimmed};
use crate::openapi::{ACCOUNT, AUTH};
use crate::server::AppState;
use crate::suspension;

/// The refusal when an address tried too many passwords.
pub const TOO_MANY_SIGN_INS: &str = "too_many_sign_ins";

pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(register))
        .routes(routes!(sign_in_with_password))
        .routes(routes!(reset_password))
        .routes(routes!(sign_out))
        .routes(routes!(account))
        .routes(routes!(update_profile))
        .routes(routes!(change_password))
        .routes(routes!(sign_out_everywhere))
}

/// The code is on its way: the same request with it finishes.
#[derive(Serialize, ToSchema)]
struct CodeSent {
    #[schema(example = "code_sent")]
    status: &'static str,
}

fn code_sent() -> Response {
    (
        StatusCode::ACCEPTED,
        Json(CodeSent {
            status: "code_sent",
        }),
    )
        .into_response()
}

fn created<T: Serialize>(body: T) -> Response {
    (StatusCode::CREATED, Json(body)).into_response()
}

pub(crate) fn valid_email(email: &str) -> Result<(), ValidationError> {
    normalize_email(email)
        .map(drop)
        .map_err(|_| ValidationError::new("invalid_email"))
}

/// Whether the password fits the policy, said before a code is mailed for
/// nothing; hashing it checks again.
fn valid_password(password: &str) -> Result<(), ValidationError> {
    if (PASSWORD_MIN_LEN..=PASSWORD_MAX_LEN).contains(&password.chars().count()) {
        return Ok(());
    }
    let mut error = ValidationError::new("weak_password");
    error.add_param("min".into(), &PASSWORD_MIN_LEN);
    error.add_param("max".into(), &PASSWORD_MAX_LEN);
    Err(error)
}

#[derive(Deserialize, Validate, ToSchema)]
struct Registration {
    #[validate(custom(function = "valid_email"))]
    email: String,
    #[validate(custom(function = "valid_password"))]
    password: String,
    /// The mailed code: left out to ask for it, given to finish.
    code: Option<String>,
}

/// Creates an account from an address and a password
///
/// Asked without `code`, mails one to the address and answers 202. Asked
/// again with it, creates the account and signs the new user in.
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = AUTH,
    request_body = Registration,
    responses(
        (status = ACCEPTED, description = "The code is on its way", body = CodeSent),
        (status = CREATED, description = "The account is created and signed in", body = SignedIn),
    )
)]
async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Body(form): Body<Registration>,
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

/// What signs in. Nothing of it is checked as a field: a wrong address is
/// refused the way a wrong password is.
#[derive(Deserialize, Validate, ToSchema)]
struct Credentials {
    email: String,
    password: String,
    /// The mailed code: left out to ask for it, given to finish.
    code: Option<String>,
}

/// Signs in with an address and its password
///
/// The password is checked at both steps, the mailed code finishes the
/// second: without `code` the answer is 202, with it 201 and the session.
/// The root account needs no code. An address is counted before its
/// password is looked at, so a password cannot be guessed by trying. A user
/// an administrator suspended is refused (`account_suspended`).
#[utoipa::path(
    post,
    path = "/auth/sign-in",
    tag = AUTH,
    request_body = Credentials,
    responses(
        (status = ACCEPTED, description = "The code is on its way", body = CodeSent),
        (status = CREATED, description = "Signed in", body = SignedIn),
    )
)]
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
    // Said to who knows the password, before a code is mailed for nothing.
    suspension::refuse_suspended(&state.executor, &owner.user_id).await?;
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

#[derive(Deserialize, Validate, ToSchema)]
struct Reset {
    #[validate(custom(function = "valid_email"))]
    email: String,
    /// The new password, with the code.
    #[validate(custom(function = "valid_password"))]
    password: Option<String>,
    /// The mailed code: left out to ask for it, given to finish.
    code: Option<String>,
}

/// Sets a new password for whoever reads the address
///
/// The first step answers the same for an address nobody registered, so it
/// tells nothing about who has an account. The second, with `code` and
/// `password`, ends every session of the user and signs them in.
#[utoipa::path(
    post,
    path = "/auth/password-reset",
    tag = AUTH,
    request_body = Reset,
    responses(
        (status = ACCEPTED, description = "The code is on its way, if the address has an account", body = CodeSent),
        (status = CREATED, description = "The password is changed and the user signed in", body = SignedIn),
    )
)]
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

/// Ends the session the request's token opened
#[utoipa::path(
    post,
    path = "/auth/sign-out",
    tag = AUTH,
    security(("session" = [])),
    responses((status = NO_CONTENT, description = "Signed out"))
)]
async fn sign_out(State(state): State<AppState>, session: Session) -> ApiResult<StatusCode> {
    state.identity().end_session(&session.token_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Who a token signs in, and until when (unix seconds) if it is not used
/// again.
#[derive(Serialize, ToSchema)]
struct Me {
    expires_at: u64,
    user: Account,
}

/// Who the request's token signs in
#[utoipa::path(
    get,
    path = "/account",
    tag = ACCOUNT,
    security(("session" = [])),
    responses((status = OK, body = Me))
)]
async fn account(session: Session) -> Json<Me> {
    Json(Me {
        expires_at: session.expires_at,
        user: Account::from(&session.user),
    })
}

#[derive(Deserialize, Validate, ToSchema)]
struct Profile {
    /// The name the account shows.
    #[serde(deserialize_with = "trimmed")]
    #[validate(length(min = 1, max = 255))]
    name: String,
}

/// Sets the name the account shows
#[utoipa::path(
    put,
    path = "/account/profile",
    tag = ACCOUNT,
    security(("session" = [])),
    request_body = Profile,
    responses((status = NO_CONTENT, description = "The profile is updated"))
)]
async fn update_profile(
    State(state): State<AppState>,
    session: Session,
    Body(form): Body<Profile>,
) -> ApiResult<StatusCode> {
    state
        .identity()
        .update_profile(&session.user.id, &session.user.account, &form.name)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize, Validate, ToSchema)]
struct NewPassword {
    current_password: String,
    #[validate(custom(function = "valid_password"))]
    password: String,
}

/// Sets a new password for who knows the current one
///
/// Every other session of the user ends; this one stays signed in.
#[utoipa::path(
    put,
    path = "/account/password",
    tag = ACCOUNT,
    security(("session" = [])),
    request_body = NewPassword,
    responses((status = NO_CONTENT, description = "The password is changed"))
)]
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

/// Ends every other session of the user
///
/// This one stays signed in.
#[utoipa::path(
    post,
    path = "/account/sign-out-everywhere",
    tag = ACCOUNT,
    security(("session" = [])),
    responses((status = NO_CONTENT, description = "The other sessions are ended"))
)]
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
        assert!(valid_password("correct horse").is_ok());
        assert!(valid_password("short").is_err());
        assert!(valid_password(&"x".repeat(PASSWORD_MAX_LEN + 1)).is_err());
    }
}
