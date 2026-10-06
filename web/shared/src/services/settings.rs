//! Account settings: meal preferences, public profile, username and the
//! signed-in devices.

use imkitchen_identity::login::Login;
use imkitchen_identity::meal_preferences::MealPreferences;

use crate::AppState;

pub struct General {
    pub email: String,
    pub description: String,
    pub preferences: MealPreferences,
}

/// Everything the general settings page shows.
pub async fn general(app: &AppState, user_id: &str) -> anyhow::Result<General> {
    let preferences = app.identity.meal_preferences.load(user_id).await?;
    let profile = app.identity.user_profile.load(user_id).await?;
    let email = app.identity.find_email(user_id).await?;
    Ok(General {
        email: email.unwrap_or_default(),
        description: profile.description,
        preferences,
    })
}

/// A username can be chosen once.
pub async fn set_username(
    app: &AppState,
    user: &Login,
    username: String,
) -> imkitchen_core::Result<()> {
    if user.username.is_some() {
        imkitchen_core::user!("Username has already been set.");
    }
    app.identity.set_username(&user.id, username).await
}

/// The devices signed in to the account (login records).
pub async fn sessions(app: &AppState, user_id: &str) -> anyhow::Result<Vec<Login>> {
    Ok(app
        .identity
        .find_login(user_id)
        .await?
        .map(|view| view.logins.0.clone())
        .unwrap_or_default())
}
