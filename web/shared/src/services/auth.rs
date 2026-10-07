use imkitchen_identity::{LoginInput, RegisterInput};
use time::OffsetDateTime;

use crate::AppState;

/// A freshly issued session: the signed JWT plus the ids it carries.
#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub expires_at: OffsetDateTime,
    pub user_id: String,
    /// The device/login record id (`acc` claim). Pass it back to
    /// `identity.logout` to end exactly this session.
    pub access_id: String,
}

/// Creates the account and returns its id. The configured root address is
/// special: it always gets the configured root password and becomes admin,
/// so the first sign-up on a fresh install bootstraps the administrator.
pub async fn register(app: &AppState, mut input: RegisterInput) -> imkitchen_core::Result<String> {
    let is_root = input.email == app.config.root.email;
    if is_root {
        input.password = app.config.root.password.to_owned();
    }

    let id = app.identity.register(input).await?;

    if is_root {
        app.identity.made_admin(&id).await?;
    }

    Ok(id)
}

/// Verifies the credentials, records the login (device) and signs a token.
pub async fn login(app: &AppState, input: LoginInput) -> imkitchen_core::Result<Session> {
    let (user_id, access_id) = app.identity.login(input).await?;
    let (token, expires_at) =
        crate::auth::encode_token(&app.config.jwt, user_id.to_owned(), access_id.to_owned())?;

    Ok(Session {
        token,
        expires_at,
        user_id,
        access_id,
    })
}
