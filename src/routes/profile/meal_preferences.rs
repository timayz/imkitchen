use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use imkitchen_recipe::DietaryRestriction;
use imkitchen_shared::Metadata;
use imkitchen_user::meal_preferences::UpdateInput;
use serde::Deserialize;
use strum::VariantArray;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::{SERVER_ERROR_MESSAGE, Template, filters};

#[derive(askama::Template)]
#[template(path = "profile-meal-preferences.html")]
pub struct MealPreferencesTemplate {
    pub error_message: Option<String>,
    pub current_path: String,
    pub profile_path: String,
    pub household_size: u8,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
    pub user: imkitchen_user::AuthUser,
}

impl Default for MealPreferencesTemplate {
    fn default() -> Self {
        Self {
            error_message: None,
            current_path: "profile".to_owned(),
            profile_path: "meal-preferences".to_owned(),
            household_size: 4,
            dietary_restrictions: Vec::default(),
            cuisine_variety_weight: 1.0,
            user: imkitchen_user::AuthUser::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    let preferences = crate::try_anyhow_response!(
        state.user_meal_preference_command.load_optional(&user.id),
        template
    );
    let preferences = preferences.unwrap_or_default();

    template
        .render(MealPreferencesTemplate {
            household_size: preferences.item.household_size,
            dietary_restrictions: preferences.item.dietary_restrictions,
            cuisine_variety_weight: preferences.item.cuisine_variety_weight,
            user,
            ..Default::default()
        })
        .into_response()
}

#[derive(Deserialize, Debug)]
pub struct ActionInput {
    pub household_size: u8,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}

pub async fn action(
    template: Template,
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    match state
        .user_meal_preference_command
        .update(
            UpdateInput {
                dietary_restrictions: input.dietary_restrictions.to_vec(),
                cuisine_variety_weight: input.cuisine_variety_weight,
                household_size: input.household_size,
            },
            &Metadata::by(user.id),
        )
        .await
    {
        Ok(_) => template.render(MealPreferencesTemplate {
            household_size: input.household_size,
            dietary_restrictions: input.dietary_restrictions,
            cuisine_variety_weight: input.cuisine_variety_weight,
            ..Default::default()
        }),
        Err(imkitchen_shared::Error::Unknown(e)) => {
            tracing::error!("{e}");

            template.render(MealPreferencesTemplate {
                error_message: Some(SERVER_ERROR_MESSAGE.to_owned()),
                household_size: input.household_size,
                dietary_restrictions: input.dietary_restrictions,
                cuisine_variety_weight: input.cuisine_variety_weight,
                ..Default::default()
            })
        }
        Err(e) => template.render(MealPreferencesTemplate {
            error_message: Some(e.to_string()),
            household_size: input.household_size,
            dietary_restrictions: input.dietary_restrictions,
            cuisine_variety_weight: input.cuisine_variety_weight,
            ..Default::default()
        }),
    }
}
