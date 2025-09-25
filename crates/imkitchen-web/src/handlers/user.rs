use crate::SharedState;
use askama::Template;
use axum::{
    body::Bytes,
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, Json},
};
use imkitchen_core::{
    models::user::User,
    repositories::UserRepository,
    services::{UserService, UserServiceError},
};
use imkitchen_shared::auth_types::{
    CookingSkillLevel, ErrorResponse, ProfileResponse, ProfileUpdateRequest, UserPublic,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct ProfileFormData {
    pub name: String,
    #[serde(rename = "familySize")]
    pub family_size: String,
    #[serde(rename = "cookingSkillLevel")]
    pub cooking_skill_level: String,
    #[serde(rename = "cookingTimePreferences.weekdayMaxMinutes")]
    pub weekday_max_minutes: String,
    #[serde(rename = "cookingTimePreferences.weekendMaxMinutes")]
    pub weekend_max_minutes: String,
    #[serde(default, rename = "dietaryRestrictions")]
    pub dietary_restrictions: Vec<String>,
    pub csrf_token: String,
}

#[derive(Template)]
#[template(path = "pages/profile.html")]
struct ProfileTemplate {
    user: UserPublic,
    dietary_options: Vec<String>,
    csrf_token: String,
}

#[derive(Template)]
#[template(path = "fragments/profile-form.html")]
#[allow(dead_code)]
struct ProfileFormTemplate {
    user: UserPublic,
    dietary_options: Vec<String>,
    csrf_token: String,
    success_message: String,
    error_message: String,
}

#[derive(Template)]
#[template(path = "fragments/profile-success.html")]
struct ProfileSuccessTemplate {
    user: UserPublic,
    dietary_options: Vec<String>,
    csrf_token: String,
}

impl ProfileSuccessTemplate {
    fn skill_level_str(&self) -> &'static str {
        match self.user.cooking_skill_level {
            CookingSkillLevel::Beginner => "beginner",
            CookingSkillLevel::Intermediate => "intermediate",
            CookingSkillLevel::Advanced => "advanced",
        }
    }

    fn has_dietary_restriction(&self, restriction: &str) -> bool {
        self.user
            .dietary_restrictions
            .contains(&restriction.to_string())
    }

    fn format_restriction(&self, restriction: &str) -> String {
        restriction.replace("-", " ").to_string()
    }
}

#[derive(Template)]
#[template(path = "fragments/profile-error.html")]
struct ProfileErrorTemplate {
    name: String,
    email: String,
    family_size: String,
    cooking_skill_level: String,
    weekday_max_minutes: String,
    weekend_max_minutes: String,
    dietary_restrictions: Vec<String>,
    dietary_options: Vec<String>,
    csrf_token: String,
    name_error: String,
    family_size_error: String,
    cooking_skill_level_error: String,
    weekday_time_error: String,
    weekend_time_error: String,
    dietary_restrictions_error: String,
    error_message: String,
}

impl ProfileTemplate {
    fn skill_level_str(&self) -> &'static str {
        match self.user.cooking_skill_level {
            CookingSkillLevel::Beginner => "beginner",
            CookingSkillLevel::Intermediate => "intermediate",
            CookingSkillLevel::Advanced => "advanced",
        }
    }

    fn has_dietary_restriction(&self, restriction: &str) -> bool {
        self.user
            .dietary_restrictions
            .contains(&restriction.to_string())
    }

    fn format_restriction(&self, restriction: &str) -> String {
        restriction.replace("-", " ")
    }
}

impl ProfileFormTemplate {
    #[allow(dead_code)]
    fn skill_level_str(&self) -> &'static str {
        match self.user.cooking_skill_level {
            CookingSkillLevel::Beginner => "beginner",
            CookingSkillLevel::Intermediate => "intermediate",
            CookingSkillLevel::Advanced => "advanced",
        }
    }

    #[allow(dead_code)]
    fn has_dietary_restriction(&self, restriction: &str) -> bool {
        self.user
            .dietary_restrictions
            .contains(&restriction.to_string())
    }

    #[allow(dead_code)]
    fn format_restriction(&self, restriction: &str) -> String {
        restriction.replace("-", " ")
    }
}

impl ProfileErrorTemplate {
    fn format_restriction(&self, restriction: &str) -> String {
        restriction.replace("-", " ")
    }

    fn has_dietary_restriction_error(&self, restriction: &str) -> bool {
        self.dietary_restrictions.contains(&restriction.to_string())
    }

    fn is_family_size_selected(&self, size: &i32) -> bool {
        self.family_size == size.to_string()
    }
}

fn convert_user_to_public(user: User) -> UserPublic {
    let dietary_restrictions: Vec<String> =
        serde_json::from_str(&user.dietary_restrictions).unwrap_or_default();

    let cooking_skill_level = match user.cooking_skill_level.as_str() {
        "intermediate" => CookingSkillLevel::Intermediate,
        "advanced" => CookingSkillLevel::Advanced,
        _ => CookingSkillLevel::Beginner,
    };

    UserPublic {
        id: user.id,
        email: user.email,
        name: user.name,
        family_size: user.family_size,
        dietary_restrictions,
        cooking_skill_level,
        cooking_time_preferences: serde_json::from_str(&user.cooking_time_preferences)
            .unwrap_or_default(),
        email_verified: user.email_verified,
        created_at: user.created_at.to_rfc3339(),
        last_active: user.last_active.to_rfc3339(),
    }
}

fn convert_core_to_shared_user_public(
    core_user: imkitchen_core::models::user::UserPublic,
) -> imkitchen_shared::auth_types::UserPublic {
    use imkitchen_shared::auth_types::{
        CookingSkillLevel as SharedCookingSkillLevel,
        CookingTimePreferences as SharedCookingTimePreferences,
    };

    let cooking_skill_level = match core_user.cooking_skill_level {
        imkitchen_core::models::user::CookingSkillLevel::Beginner => {
            SharedCookingSkillLevel::Beginner
        }
        imkitchen_core::models::user::CookingSkillLevel::Intermediate => {
            SharedCookingSkillLevel::Intermediate
        }
        imkitchen_core::models::user::CookingSkillLevel::Advanced => {
            SharedCookingSkillLevel::Advanced
        }
    };

    let cooking_time_preferences = SharedCookingTimePreferences {
        weekday_max_minutes: core_user.cooking_time_preferences.weekday_max_minutes,
        weekend_max_minutes: core_user.cooking_time_preferences.weekend_max_minutes,
    };

    imkitchen_shared::auth_types::UserPublic {
        id: core_user.id,
        email: core_user.email,
        name: core_user.name,
        family_size: core_user.family_size,
        dietary_restrictions: core_user.dietary_restrictions,
        cooking_skill_level,
        cooking_time_preferences,
        email_verified: core_user.email_verified,
        created_at: core_user.created_at.to_rfc3339(),
        last_active: core_user.last_active.to_rfc3339(),
    }
}

/// Render profile management page - requires authentication  
pub async fn profile_page(
    Extension(user): Extension<User>,
    State(_shared_state): State<SharedState>,
) -> Result<Html<String>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Profile page requested for user: {} ({})",
        user.name, user.email
    );

    // Convert user to public format
    let user_public = convert_user_to_public(user);

    // Define dietary options
    let dietary_options = vec![
        "vegetarian".to_string(),
        "vegan".to_string(),
        "gluten-free".to_string(),
        "dairy-free".to_string(),
        "nut-allergies".to_string(),
    ];

    // Generate CSRF token (for now, use a placeholder)
    let csrf_token = "placeholder_csrf_token".to_string();

    let template = ProfileTemplate {
        user: user_public,
        dietary_options,
        csrf_token,
    };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(e) => {
            info!("Template rendering error: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(
                    "Failed to render profile page".to_string(),
                )),
            ))
        }
    }
}

/// Handle profile form submission - requires authentication
pub async fn profile_form_handler(
    Extension(user): Extension<User>,
    State(shared_state): State<SharedState>,
    body: Bytes,
) -> Result<Html<String>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Profile form submitted for user: {} ({})",
        user.name, user.email
    );

    // Parse form data manually
    let form_str = String::from_utf8_lossy(&body);
    let mut parsed: HashMap<String, Vec<String>> = HashMap::new();

    for pair in form_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = key.replace("%5B%5D", ""); // Remove URL-encoded []
            let decoded_value = value.replace('+', " "); // Basic URL decoding
            parsed
                .entry(decoded_key.to_string())
                .or_default()
                .push(decoded_value.to_string());
        }
    }

    // Extract form data manually to handle checkbox arrays
    let name = parsed
        .get("name")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();
    let family_size_str = parsed
        .get("familySize")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();
    let cooking_skill_level = parsed
        .get("cookingSkillLevel")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();
    let weekday_max_minutes_str = parsed
        .get("cookingTimePreferences.weekdayMaxMinutes")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();
    let weekend_max_minutes_str = parsed
        .get("cookingTimePreferences.weekendMaxMinutes")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();
    let csrf_token = parsed
        .get("csrf_token")
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default();

    // Handle dietary restrictions (checkboxes) - get all values
    let dietary_restrictions = parsed
        .get("dietaryRestrictions")
        .cloned()
        .unwrap_or_default();

    // Create ProfileFormData structure
    let form_data = ProfileFormData {
        name,
        family_size: family_size_str.clone(),
        cooking_skill_level: cooking_skill_level.clone(),
        weekday_max_minutes: weekday_max_minutes_str.clone(),
        weekend_max_minutes: weekend_max_minutes_str.clone(),
        dietary_restrictions,
        csrf_token,
    };

    // Validate and convert form data to ProfileUpdateRequest
    let family_size = match family_size_str.parse::<i32>() {
        Ok(size) if (1..=8).contains(&size) => size,
        _ => {
            let dietary_options = vec![
                "vegetarian".to_string(),
                "vegan".to_string(),
                "gluten-free".to_string(),
                "dairy-free".to_string(),
                "nut-allergies".to_string(),
            ];
            let csrf_token = "placeholder_csrf_token".to_string();
            let template = ProfileErrorTemplate {
                name: form_data.name,
                email: user.email,
                family_size: form_data.family_size,
                cooking_skill_level: form_data.cooking_skill_level,
                weekday_max_minutes: form_data.weekday_max_minutes,
                weekend_max_minutes: form_data.weekend_max_minutes,
                dietary_restrictions: form_data.dietary_restrictions.clone(),
                dietary_options,
                csrf_token,
                name_error: "".to_string(),
                family_size_error: "Family size must be between 1 and 8".to_string(),
                cooking_skill_level_error: "".to_string(),
                weekday_time_error: "".to_string(),
                weekend_time_error: "".to_string(),
                dietary_restrictions_error: "".to_string(),
                error_message: "".to_string(),
            };
            return match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(e) => {
                    info!("Template rendering error: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(
                            "Failed to render profile page".to_string(),
                        )),
                    ))
                }
            };
        }
    };

    let weekday_max_minutes = match form_data.weekday_max_minutes.parse::<i32>() {
        Ok(mins) if (5..=480).contains(&mins) => mins,
        _ => {
            let dietary_options = vec![
                "vegetarian".to_string(),
                "vegan".to_string(),
                "gluten-free".to_string(),
                "dairy-free".to_string(),
                "nut-allergies".to_string(),
            ];
            let csrf_token = "placeholder_csrf_token".to_string();
            let template = ProfileErrorTemplate {
                name: form_data.name,
                email: user.email,
                family_size: form_data.family_size,
                cooking_skill_level: form_data.cooking_skill_level,
                weekday_max_minutes: form_data.weekday_max_minutes,
                weekend_max_minutes: form_data.weekend_max_minutes,
                dietary_restrictions: form_data.dietary_restrictions.clone(),
                dietary_options,
                csrf_token,
                name_error: "".to_string(),
                family_size_error: "".to_string(),
                cooking_skill_level_error: "".to_string(),
                weekday_time_error: "Weekday time must be between 5 and 480 minutes".to_string(),
                weekend_time_error: "".to_string(),
                dietary_restrictions_error: "".to_string(),
                error_message: "".to_string(),
            };
            return match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(e) => {
                    info!("Template rendering error: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(
                            "Failed to render profile page".to_string(),
                        )),
                    ))
                }
            };
        }
    };

    let weekend_max_minutes = match form_data.weekend_max_minutes.parse::<i32>() {
        Ok(mins) if (5..=480).contains(&mins) => mins,
        _ => {
            let dietary_options = vec![
                "vegetarian".to_string(),
                "vegan".to_string(),
                "gluten-free".to_string(),
                "dairy-free".to_string(),
                "nut-allergies".to_string(),
            ];
            let csrf_token = "placeholder_csrf_token".to_string();
            let template = ProfileErrorTemplate {
                name: form_data.name,
                email: user.email,
                family_size: form_data.family_size,
                cooking_skill_level: form_data.cooking_skill_level,
                weekday_max_minutes: form_data.weekday_max_minutes,
                weekend_max_minutes: form_data.weekend_max_minutes,
                dietary_restrictions: form_data.dietary_restrictions.clone(),
                dietary_options,
                csrf_token,
                name_error: "".to_string(),
                family_size_error: "".to_string(),
                cooking_skill_level_error: "".to_string(),
                weekday_time_error: "".to_string(),
                weekend_time_error: "Weekend time must be between 5 and 480 minutes".to_string(),
                dietary_restrictions_error: "".to_string(),
                error_message: "".to_string(),
            };
            return match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(e) => {
                    info!("Template rendering error: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(
                            "Failed to render profile page".to_string(),
                        )),
                    ))
                }
            };
        }
    };

    let cooking_skill_level = match form_data.cooking_skill_level.as_str() {
        "beginner" => CookingSkillLevel::Beginner,
        "intermediate" => CookingSkillLevel::Intermediate,
        "advanced" => CookingSkillLevel::Advanced,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "Invalid cooking skill level".to_string(),
                )),
            ))
        }
    };

    let profile_update = ProfileUpdateRequest {
        name: Some(form_data.name.clone()),
        family_size: Some(family_size),
        dietary_restrictions: Some(form_data.dietary_restrictions.clone()),
        cooking_skill_level: Some(cooking_skill_level),
        cooking_time_preferences: Some(imkitchen_shared::auth_types::CookingTimePreferences {
            weekday_max_minutes,
            weekend_max_minutes,
        }),
    };

    let state = shared_state.read().await;
    let db = state.db.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Database not available".to_string())),
        )
    })?;
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let user_service = UserService::new(user_repository);

    drop(state);

    match user_service.update_profile(&user.id, profile_update).await {
        Ok(updated_profile) => {
            info!("Profile updated successfully for user: {}", user.id);

            // Convert updated profile to UserPublic for template
            let user_public = convert_core_to_shared_user_public(updated_profile);

            // Define dietary options
            let dietary_options = vec![
                "vegetarian".to_string(),
                "vegan".to_string(),
                "gluten-free".to_string(),
                "dairy-free".to_string(),
                "nut-allergies".to_string(),
            ];

            let csrf_token = "placeholder_csrf_token".to_string();

            let template = ProfileSuccessTemplate {
                user: user_public,
                dietary_options,
                csrf_token,
            };

            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(e) => {
                    info!("Template rendering error: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(
                            "Failed to render profile page".to_string(),
                        )),
                    ))
                }
            }
        }
        Err(UserServiceError::ValidationError(msg)) => {
            let dietary_options = vec![
                "vegetarian".to_string(),
                "vegan".to_string(),
                "gluten-free".to_string(),
                "dairy-free".to_string(),
                "nut-allergies".to_string(),
            ];
            let csrf_token = "placeholder_csrf_token".to_string();

            let template = ProfileErrorTemplate {
                name: form_data.name,
                email: user.email,
                family_size: form_data.family_size,
                cooking_skill_level: form_data.cooking_skill_level,
                weekday_max_minutes: form_data.weekday_max_minutes,
                weekend_max_minutes: form_data.weekend_max_minutes,
                dietary_restrictions: form_data.dietary_restrictions,
                dietary_options,
                csrf_token,
                name_error: "".to_string(),
                family_size_error: "".to_string(),
                cooking_skill_level_error: "".to_string(),
                weekday_time_error: "".to_string(),
                weekend_time_error: "".to_string(),
                dietary_restrictions_error: "".to_string(),
                error_message: msg,
            };

            match template.render() {
                Ok(html) => Ok(Html(html)),
                Err(e) => {
                    info!("Template rendering error: {:?}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(
                            "Failed to render profile page".to_string(),
                        )),
                    ))
                }
            }
        }
        Err(UserServiceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("Profile not found".to_string())),
        )),
        Err(e) => {
            info!("Error updating profile: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to update profile".to_string())),
            ))
        }
    }
}

/// Get current user profile JSON - requires authentication
pub async fn get_profile_api(
    Extension(user): Extension<User>,
    State(shared_state): State<SharedState>,
) -> Result<Json<UserPublic>, Json<ErrorResponse>> {
    info!("Getting profile for user: {}", user.id);

    let state = shared_state.read().await;
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| Json(ErrorResponse::new("Database not available".to_string())))?;
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let user_service = UserService::new(user_repository);

    drop(state); // Release the read lock

    match user_service.get_profile(&user.id).await {
        Ok(profile) => {
            // Convert from core UserPublic to shared UserPublic
            let shared_profile = convert_core_to_shared_user_public(profile);
            Ok(Json(shared_profile))
        }
        Err(UserServiceError::UserNotFound) => {
            Err(Json(ErrorResponse::new("Profile not found".to_string())))
        }
        Err(e) => {
            info!("Error getting profile: {:?}", e);
            Err(Json(ErrorResponse::new(
                "Failed to get profile".to_string(),
            )))
        }
    }
}

/// Get current user profile (legacy) - requires authentication
pub async fn get_profile(
    Extension(user): Extension<User>,
) -> Result<Json<UserPublic>, (StatusCode, Json<ErrorResponse>)> {
    info!("Profile requested for user: {} ({})", user.name, user.email);

    let user_public = convert_user_to_public(user);
    Ok(Json(user_public))
}

/// Update user profile - requires authentication
pub async fn update_profile(
    Extension(user): Extension<User>,
    Json(_update_data): Json<serde_json::Value>,
) -> Result<Json<UserPublic>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Profile update requested for user: {} ({})",
        user.name, user.email
    );

    // TODO: Implement profile update logic
    // For now, just return current user profile

    let user_public = convert_user_to_public(user);
    Ok(Json(user_public))
}

/// Update user profile - requires authentication (new implementation)
pub async fn update_profile_api(
    Extension(user): Extension<User>,
    State(shared_state): State<SharedState>,
    Json(request): Json<ProfileUpdateRequest>,
) -> Result<Json<ProfileResponse>, Json<ErrorResponse>> {
    info!("Updating profile for user: {}", user.id);

    let state = shared_state.read().await;
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| Json(ErrorResponse::new("Database not available".to_string())))?;
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let user_service = UserService::new(user_repository);

    drop(state); // Release the read lock

    match user_service.update_profile(&user.id, request).await {
        Ok(updated_profile) => {
            let shared_profile = convert_core_to_shared_user_public(updated_profile);
            Ok(Json(ProfileResponse {
                success: true,
                message: "Profile updated successfully".to_string(),
                user: Some(shared_profile),
            }))
        }
        Err(UserServiceError::ValidationError(msg)) => Err(Json(ErrorResponse::new(msg))),
        Err(UserServiceError::UserNotFound) => {
            Err(Json(ErrorResponse::new("Profile not found".to_string())))
        }
        Err(e) => {
            info!("Error updating profile: {:?}", e);
            Err(Json(ErrorResponse::new(
                "Failed to update profile".to_string(),
            )))
        }
    }
}

/// Delete user account - requires authentication
pub async fn delete_account(
    Extension(user): Extension<User>,
    State(shared_state): State<SharedState>,
) -> Result<Json<ProfileResponse>, Json<ErrorResponse>> {
    info!("Deleting account for user: {}", user.id);

    let state = shared_state.read().await;
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| Json(ErrorResponse::new("Database not available".to_string())))?;
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let user_service = UserService::new(user_repository);

    drop(state); // Release the read lock

    match user_service.delete_account(&user.id).await {
        Ok(()) => Ok(Json(ProfileResponse {
            success: true,
            message: "Account deleted successfully".to_string(),
            user: None,
        })),
        Err(UserServiceError::UserNotFound) => {
            Err(Json(ErrorResponse::new("Account not found".to_string())))
        }
        Err(e) => {
            info!("Error deleting account: {:?}", e);
            Err(Json(ErrorResponse::new(
                "Failed to delete account".to_string(),
            )))
        }
    }
}

/// Handle account deletion - requires authentication and confirmation
pub async fn delete_account_handler(
    Extension(user): Extension<User>,
    State(shared_state): State<SharedState>,
) -> Result<Html<String>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        "Account deletion requested for user: {} ({})",
        user.name, user.email
    );

    let state = shared_state.read().await;
    let db = state.db.as_ref().ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("Database not available".to_string())),
        )
    })?;
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let user_service = UserService::new(user_repository);

    drop(state);

    match user_service.delete_account(&user.id).await {
        Ok(_) => {
            info!("Account deleted successfully for user: {}", user.id);

            // Return a simple confirmation message or redirect
            let html = r#"
                <!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Account Deleted - imkitchen</title>
                    <script src="https://cdn.tailwindcss.com"></script>
                </head>
                <body class="bg-gray-50 min-h-screen flex items-center justify-center">
                    <div class="max-w-md mx-auto text-center p-8 bg-white rounded-lg shadow-lg">
                        <div class="mb-6">
                            <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100">
                                <svg class="h-6 w-6 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                </svg>
                            </div>
                        </div>
                        <h1 class="text-2xl font-bold text-gray-900 mb-4">Account Deleted</h1>
                        <p class="text-gray-600 mb-6">Your account has been successfully deleted. We're sorry to see you go!</p>
                        <a href="/login" class="inline-block bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-md">
                            Back to Login
                        </a>
                    </div>
                </body>
                </html>
            "#;

            Ok(Html(html.to_string()))
        }
        Err(UserServiceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("User not found".to_string())),
        )),
        Err(e) => {
            info!("Error deleting account: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to delete account".to_string())),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use imkitchen_core::models::user::User;

    fn create_test_user() -> User {
        User {
            id: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            name: "Test User".to_string(),
            family_size: 2,
            dietary_restrictions: "[]".to_string(),
            cooking_skill_level: "beginner".to_string(),
            email_verified: true,
            email_verification_token: None,
            email_verification_expires_at: None,
            password_reset_token: None,
            password_reset_expires_at: None,
            cooking_time_preferences: r#"{"weekdayMaxMinutes": 30, "weekendMaxMinutes": 60}"#
                .to_string(),
            created_at: Utc::now(),
            last_active: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_get_profile() {
        let user = create_test_user();
        let result = get_profile(Extension(user.clone())).await;

        assert!(result.is_ok());
        let user_public = result.unwrap().0;
        assert_eq!(user_public.email, user.email);
        assert_eq!(user_public.name, user.name);
    }

    #[tokio::test]
    async fn test_update_profile() {
        let user = create_test_user();
        let update_data = serde_json::json!({"name": "Updated Name"});
        let result = update_profile(Extension(user.clone()), Json(update_data)).await;

        assert!(result.is_ok());
        let user_public = result.unwrap().0;
        assert_eq!(user_public.email, user.email);
    }
}
