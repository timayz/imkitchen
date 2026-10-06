use imkitchen_identity::LoginInput;
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
