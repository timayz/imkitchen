//! Account settings: meal preferences, public profile, username, the
//! signed-in devices and account deletion.

use evento::cursor::Args;
use imkitchen_core::recipe::query::user::{RecipesQuery, SortBy};
use imkitchen_identity::login::Login;
use imkitchen_identity::meal_preferences::MealPreferences;
use imkitchen_identity::user_profile;

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

/// Deletes the account after re-checking the password. The user's own data
/// goes first (subscription cancelled, recipes deleted, profile blanked) so
/// a failure half-way leaves an account that can retry; the identity
/// deletion comes last and ends every session.
pub async fn delete_account(
    app: &AppState,
    user: &Login,
    password: &str,
) -> imkitchen_core::Result<()> {
    app.identity.check_password(&user.id, password).await?;

    app.billing.subscription.cancel(&user.id).await?;

    for id in owned_recipe_ids(app, &user.id).await? {
        app.core.recipe.delete(&id, &user.id).await?;
    }

    app.identity
        .user_profile
        .update(
            &user.id,
            user_profile::UpdateInput {
                description: String::new(),
            },
        )
        .await?;

    app.identity.delete_account(&user.id).await
}

/// Every recipe the user owns, collected before deleting any: the read model
/// drops rows asynchronously, so paging while deleting would revisit them.
async fn owned_recipe_ids(app: &AppState, user_id: &str) -> anyhow::Result<Vec<String>> {
    let mut ids = vec![];
    let mut after = None;

    loop {
        let page = app
            .core
            .recipe
            .filter_user(RecipesQuery {
                exclude_ids: None,
                user_id: Some(user_id.to_owned()),
                recipe_type: None,
                is_shared: None,
                has_thumbnail: None,
                dietary_restrictions: vec![],
                dietary_where_any: false,
                in_meal_plan: None,
                sort_by: SortBy::RecentlyAdded,
                search: None,
                args: Args::forward(100, after),
            })
            .await?;

        ids.extend(page.edges.iter().map(|edge| edge.node.id.to_owned()));

        if !page.page_info.has_next_page {
            return Ok(ids);
        }
        after = page.page_info.end_cursor;
    }
}
