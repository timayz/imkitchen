use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use imkitchen_recipe::DietaryRestriction;
use imkitchen_user::meal_preferences::UpdateInput;
use serde::Deserialize;
use strum::VariantArray;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::{Template, ToastSuccessTemplate, filters};

#[derive(askama::Template)]
#[template(path = "profile-meal-preferences.html")]
pub struct MealPreferencesTemplate {
    pub current_path: String,
    pub profile_path: String,
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
    pub user: AuthUser,
}

impl Default for MealPreferencesTemplate {
    fn default() -> Self {
        Self {
            current_path: "profile".to_owned(),
            profile_path: "meal-preferences".to_owned(),
            household_size: 4,
            dietary_restrictions: Vec::default(),
            cuisine_variety_weight: 1.0,
            user: AuthUser::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let preferences = crate::try_page_response!(
        imkitchen_user::meal_preferences::load(&app.executor, &user.id),
        template
    );

    template.render(MealPreferencesTemplate {
        household_size: preferences.household_size,
        dietary_restrictions: preferences.dietary_restrictions.to_vec(),
        cuisine_variety_weight: preferences.cuisine_variety_weight,
        user,
        ..Default::default()
    })
}

#[derive(Deserialize, Debug)]
pub struct ActionInput {
    pub household_size: u16,
    #[serde(default)]
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn action(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    let preferences = crate::try_response!(anyhow:
        imkitchen_user::meal_preferences::load(&app.executor, &user.id),
        template
    );

    crate::try_response!(
        preferences.update(UpdateInput {
            dietary_restrictions: input.dietary_restrictions.to_vec(),
            cuisine_variety_weight: input.cuisine_variety_weight,
            household_size: input.household_size,
        }),
        template
    );

    template
        .render(ToastSuccessTemplate {
            original: None,
            message: "Meal preferences updated successfully",
            description: None,
        })
        .into_response()
}
