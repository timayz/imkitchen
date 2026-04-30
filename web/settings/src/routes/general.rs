use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use imkitchen_identity::meal_preferences::UpdateInput;
use imkitchen_types::recipe::DietaryRestriction;
use serde::Deserialize;
use strum::VariantArray;

use imkitchen_web_shared::AppState;
use imkitchen_web_shared::auth::AuthUser;
use imkitchen_web_shared::template::{Template, ToastErrorTemplate, ToastSuccessTemplate, filters};

#[derive(askama::Template)]
#[template(path = "settings-general.html")]
pub struct MealPreferencesTemplate {
    pub current_path: String,
    pub settings_path: String,
    pub household_size: u16,
    pub dietary_restrictions: Vec<DietaryRestriction>,
    pub cuisine_variety_weight: f32,
    pub user: AuthUser,
}

impl Default for MealPreferencesTemplate {
    fn default() -> Self {
        Self {
            current_path: "settings".to_owned(),
            settings_path: "general".to_owned(),
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
    let preferences = imkitchen_web_shared::try_page_response!(
        app.identity.meal_preferences.load(&user.id),
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
    imkitchen_web_shared::try_response!(
        app.identity.meal_preferences.update(
            &user.id,
            UpdateInput {
                dietary_restrictions: input.dietary_restrictions.to_vec(),
                cuisine_variety_weight: input.cuisine_variety_weight,
                household_size: input.household_size,
            }
        ),
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

#[derive(Deserialize)]
pub struct SetUsernameActionInput {
    pub username: String,
}

pub async fn set_username_action(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
    Form(input): Form<SetUsernameActionInput>,
) -> impl IntoResponse {
    if user.username.is_some() {
        return (
            [("ts-swap", "skip")],
            template.render(ToastErrorTemplate {
                original: None,
                message: "Username has already been set.",
                description: None,
            }),
        )
            .into_response();
    }

    imkitchen_web_shared::try_response!(
        app.identity.set_username(&user.id, input.username),
        template
    );

    "<div ts-trigger=\"load\" ts-action=\"remove\"></div>".into_response()
}
