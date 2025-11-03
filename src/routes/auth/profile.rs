//! Profile route handlers

use super::{render_template, AppState};
use crate::auth::AuthUser;
use crate::queries::user::get_user_profile;
use askama::Template;
use axum::{extract::State, response::Response};
use axum_extra::extract::Form;
use imkitchen_user::command::{Command, UpdateProfileInput};
use imkitchen_user::event::EventMetadata;
use serde::Deserialize;
use tracing::{error, info};
use ulid::Ulid;

/// Profile page template
#[derive(Template)]
#[template(path = "pages/auth/profile.html")]
struct ProfilePageTemplate {
    profile: ProfileData,
    available_restrictions: Vec<String>,
    success: Option<String>,
    error: Option<String>,
}

/// Profile data for template
#[derive(Clone)]
struct ProfileData {
    dietary_restrictions: Vec<String>,
    cuisine_variety_weight: f32,
    household_size: Option<i32>,
}

/// Profile form data
#[derive(Deserialize)]
pub struct ProfileForm {
    #[serde(default)]
    dietary_restrictions: Vec<String>,
    cuisine_variety_weight: f32,
    household_size: Option<String>,
}

/// List of available dietary restrictions
fn get_available_restrictions() -> Vec<String> {
    vec![
        "Vegetarian".to_string(),
        "Vegan".to_string(),
        "Gluten-free".to_string(),
        "Dairy-free".to_string(),
        "Nut-free".to_string(),
        "Shellfish-free".to_string(),
        "Kosher".to_string(),
        "Halal".to_string(),
        "Low-carb".to_string(),
        "Keto".to_string(),
        "Paleo".to_string(),
    ]
}

/// GET /auth/profile - Show profile form
pub async fn get_profile(State(state): State<AppState>, auth_user: AuthUser) -> Response {
    info!(user_id = %auth_user.user_id, "Loading profile page");

    // Load user profile from query database
    let profile = match get_user_profile(&state.query_pool, &auth_user.user_id).await {
        Ok(p) => ProfileData {
            dietary_restrictions: p.dietary_restrictions,
            cuisine_variety_weight: p.cuisine_variety_weight,
            household_size: p.household_size,
        },
        Err(e) => {
            error!(error = %e, user_id = %auth_user.user_id, "Failed to load profile");
            // Return defaults on error
            ProfileData {
                dietary_restrictions: Vec::new(),
                cuisine_variety_weight: 0.7,
                household_size: None,
            }
        }
    };

    render_template(ProfilePageTemplate {
        profile,
        available_restrictions: get_available_restrictions(),
        success: None,
        error: None,
    })
}

/// POST /auth/profile - Handle profile update submission
pub async fn post_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Form(form): Form<ProfileForm>,
) -> Response {
    info!(
        user_id = %auth_user.user_id,
        restrictions_count = form.dietary_restrictions.len(),
        "Processing profile update"
    );

    // Parse household_size from string
    let household_size = form
        .household_size
        .as_ref()
        .and_then(|s| s.parse::<i32>().ok());

    let input = UpdateProfileInput {
        dietary_restrictions: form.dietary_restrictions.clone(),
        cuisine_variety_weight: form.cuisine_variety_weight,
        household_size,
    };

    let metadata = EventMetadata {
        user_id: Some(auth_user.user_id.clone()),
        request_id: Ulid::new().to_string(),
    };

    let command = Command::new(state.evento.clone());

    match command
        .update_profile(auth_user.user_id.clone(), input, metadata)
        .await
    {
        Ok(_) => {
            info!(user_id = %auth_user.user_id, "Profile updated successfully");

            // Return success template
            render_template(ProfilePageTemplate {
                profile: ProfileData {
                    dietary_restrictions: form.dietary_restrictions,
                    cuisine_variety_weight: form.cuisine_variety_weight,
                    household_size,
                },
                available_restrictions: get_available_restrictions(),
                success: Some("Profile updated successfully!".to_string()),
                error: None,
            })
        }
        Err(e) => {
            error!(error = %e, user_id = %auth_user.user_id, "Profile update failed");

            // Return error template with current form values
            render_template(ProfilePageTemplate {
                profile: ProfileData {
                    dietary_restrictions: form.dietary_restrictions,
                    cuisine_variety_weight: form.cuisine_variety_weight,
                    household_size,
                },
                available_restrictions: get_available_restrictions(),
                success: None,
                error: Some(
                    "Failed to update profile. Please check your input and try again.".to_string(),
                ),
            })
        }
    }
}
