use std::collections::HashSet;

use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use imkitchen_shared::Metadata;
use imkitchen_user::UpdateMealPreferencesInput;
use serde::Deserialize;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::{SERVER_ERROR_MESSAGE, ServerErrorTemplate, Template, filters};

#[derive(askama::Template)]
#[template(path = "profile-meal-preferences.html")]
pub struct MealPreferencesTemplate {
    pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
    pub household_size: u8,
    pub dietary_restrictions: HashSet<String>,
    pub cuisine_variety_weight: f32,
    pub user: imkitchen_user::AuthUser,
}

impl Default for MealPreferencesTemplate {
    fn default() -> Self {
        Self {
            error_message: None,
            current_path: "profile".to_owned(),
            profile_path: "meal-preferences".to_owned(),
            household_size: 2,
            dietary_restrictions: HashSet::default(),
            cuisine_variety_weight: 0.7,
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

pub async fn page(
    template: Template<MealPreferencesTemplate>,
    server_error: Template<ServerErrorTemplate>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let preferences = match state
        .user_command
        .load_meal_preferences_optional(&user.id)
        .await
    {
        Ok(loaded) => loaded.unwrap_or_default().item,
        Err(e) => {
            tracing::error!("{e}");
            return server_error.render(ServerErrorTemplate).into_response();
        }
    };

    let dietary_restrictions = HashSet::from_iter(preferences.dietary_restrictions.iter().cloned());

    template
        .render(MealPreferencesTemplate {
            household_size: preferences.household_size,
            dietary_restrictions,
            cuisine_variety_weight: preferences.cuisine_variety_weight,
            user,
            ..Default::default()
        })
        .into_response()
}

#[derive(Deserialize, Debug)]
pub struct ActionInput {
    pub household_size: u8,
    #[serde(default)]
    pub dietary_restrictions: Vec<String>,
    pub cuisine_variety_weight: f32,
}

pub async fn action(
    template: Template<MealPreferencesTemplate>,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    let dietary_restrictions = HashSet::from_iter(input.dietary_restrictions.iter().cloned());

    match state
        .user_command
        .update_meal_preferences(
            UpdateMealPreferencesInput {
                dietary_restrictions: input.dietary_restrictions,
                cuisine_variety_weight: input.cuisine_variety_weight,
                household_size: input.household_size,
            },
            Metadata::by(user.id),
        )
        .await
    {
        Ok(_) => template.render(MealPreferencesTemplate {
            household_size: input.household_size,
            dietary_restrictions,
            cuisine_variety_weight: input.cuisine_variety_weight,
            ..Default::default()
        }),
        Err(imkitchen_shared::Error::Unknown(e)) => {
            tracing::error!("{e}");

            template.render(MealPreferencesTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                household_size: input.household_size,
                dietary_restrictions,
                cuisine_variety_weight: input.cuisine_variety_weight,
                ..Default::default()
            })
        }
        Err(e) => template.render(MealPreferencesTemplate {
            error_message: Some(e.to_string()),
            household_size: input.household_size,
            dietary_restrictions,
            cuisine_variety_weight: input.cuisine_variety_weight,
            ..Default::default()
        }),
    }
}
