use imkitchen_identity::login::Login;
use imkitchen_web_shared::AppState;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct Token {
    pub token: String,
    #[serde(serialize_with = "super::rfc3339")]
    pub expires_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct SessionResponse {
    #[serde(flatten)]
    pub token: Token,
    pub user: Me,
}

#[derive(Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// The signed-in user as the app needs it. `premium_enabled` says whether
/// monetization exists at all (badges), never whether anything is for sale:
/// purchase flows do not exist in the API by design.
#[derive(Serialize)]
pub struct Me {
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    pub role: String,
    pub is_chef: bool,
    pub is_admin: bool,
    pub is_premium: bool,
    pub premium_enabled: bool,
    pub tz: String,
}

impl Me {
    pub fn new(app: &AppState, login: &Login) -> Self {
        Self {
            id: login.id.to_owned(),
            email: login.email.to_owned(),
            username: login.username.to_owned(),
            role: login.role.to_string(),
            is_chef: login.is_chef(),
            is_admin: login.is_admin(),
            is_premium: login.is_premium(),
            premium_enabled: app.config.premium.is_some(),
            tz: login.tz.to_owned(),
        }
    }
}
