// Profile management handlers with TwinSpark integration

use axum::{
    body::Bytes,
    extract::{Form, State},
    http::{HeaderMap, StatusCode},
    response::Html,
};
use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};
use imkitchen_user::commands::{
    ChangeDietaryRestrictionsCommand, ProfileCommandHandler, UpdateUserProfileCommand,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use crate::{middleware::auth::extract_user_id_from_token, AppState};

/// Helper function to extract user ID from request headers
fn extract_user_id_from_headers(headers: &HeaderMap) -> Result<uuid::Uuid, StatusCode> {
    use axum::http::header::COOKIE;

    if let Some(cookie_header) = headers.get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(session_value) = cookie.strip_prefix("imkitchen_session=") {
                    if let Some(user_id) = extract_user_id_from_token(session_value) {
                        return Ok(user_id);
                    }
                }
            }
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

// Template data structures for rendering
#[derive(Debug, Clone, Serialize)]
pub struct ProfilePageData {
    pub user_email: String,
    pub profile: UserProfileData,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserProfileData {
    pub family_size: u8,
    pub cooking_skill_level: String,
    pub dietary_restrictions: Vec<String>,
    pub weekday_cooking_minutes: u32,
    pub weekend_cooking_minutes: u32,
}

/// Profile update success response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdateSuccessResponse {
    pub success: bool,
    pub message: String,
    pub updated_profile: UpdatedProfileData,
}

/// Dietary restrictions update success response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DietaryUpdateSuccessResponse {
    pub success: bool,
    pub message: String,
    pub updated_restrictions: Vec<String>,
}

/// Profile validation error response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileValidationErrorResponse {
    pub success: bool,
    pub validation_errors: Vec<String>,
    pub field_errors: HashMap<String, String>,
}

/// Dietary validation error response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DietaryValidationErrorResponse {
    pub success: bool,
    pub validation_errors: Vec<String>,
    pub restriction_conflicts: Option<Vec<String>>,
}

/// Data structure for updated profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatedProfileData {
    pub family_size: FamilySize,
    pub cooking_skill_level: String,
    pub weekday_cooking_minutes: u32,
    pub weekend_cooking_minutes: u32,
}

/// Dietary restriction option for template rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DietaryRestrictionOption {
    pub name: String,
    pub display_name: String,
    pub description: String,
}

impl DietaryRestrictionOption {
    pub fn all_options() -> Vec<Self> {
        vec![
            Self {
                name: "Vegetarian".to_string(),
                display_name: "Vegetarian".to_string(),
                description: "No meat, but includes dairy and eggs".to_string(),
            },
            Self {
                name: "Vegan".to_string(),
                display_name: "Vegan".to_string(),
                description: "No animal products including dairy and eggs".to_string(),
            },
            Self {
                name: "GlutenFree".to_string(),
                display_name: "Gluten-Free".to_string(),
                description: "No wheat, barley, rye, or gluten-containing ingredients".to_string(),
            },
            Self {
                name: "DairyFree".to_string(),
                display_name: "Dairy-Free".to_string(),
                description: "No milk, cheese, yogurt, or dairy products".to_string(),
            },
            Self {
                name: "NutFree".to_string(),
                display_name: "Nut-Free".to_string(),
                description: "No tree nuts or peanuts".to_string(),
            },
            Self {
                name: "SoyFree".to_string(),
                display_name: "Soy-Free".to_string(),
                description: "No soy or soy-derived ingredients".to_string(),
            },
            Self {
                name: "LowSodium".to_string(),
                display_name: "Low Sodium".to_string(),
                description: "Reduced salt and sodium content".to_string(),
            },
            Self {
                name: "LowCarb".to_string(),
                display_name: "Low Carb".to_string(),
                description: "Reduced carbohydrate content".to_string(),
            },
            Self {
                name: "Keto".to_string(),
                display_name: "Keto".to_string(),
                description: "Very low carb, high fat diet".to_string(),
            },
            Self {
                name: "Paleo".to_string(),
                display_name: "Paleo".to_string(),
                description: "Whole foods, no processed foods or grains".to_string(),
            },
        ]
    }
}

/// Profile update form data
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ProfileUpdateForm {
    #[validate(range(
        min = 1,
        max = 8,
        message = "Family size must be between 1 and 8 people"
    ))]
    pub family_size: u8,

    pub cooking_skill_level: String, // Will be validated as SkillLevel enum

    #[validate(range(
        min = 5,
        max = 480,
        message = "Weekday cooking time must be between 5 and 480 minutes"
    ))]
    pub weekday_cooking_minutes: u32,

    #[validate(range(
        min = 5,
        max = 480,
        message = "Weekend cooking time must be between 5 and 480 minutes"
    ))]
    pub weekend_cooking_minutes: u32,
}

/// Dietary restrictions update form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DietaryRestrictionsForm {
    pub dietary_restrictions: Vec<String>, // Will be validated as DietaryRestriction enums
}

impl DietaryRestrictionsForm {
    pub fn get_restrictions(&self) -> Vec<String> {
        self.dietary_restrictions.clone()
    }
}

impl ProfileUpdateForm {
    /// Validate and convert to domain objects
    pub fn to_domain(&self) -> Result<(FamilySize, SkillLevel), String> {
        // Validate family size
        let family_size =
            FamilySize::new(self.family_size).map_err(|_| "Invalid family size".to_string())?;

        // Validate skill level
        let skill_level = match self.cooking_skill_level.as_str() {
            "Beginner" => SkillLevel::Beginner,
            "Intermediate" => SkillLevel::Intermediate,
            "Advanced" => SkillLevel::Advanced,
            _ => return Err("Invalid skill level".to_string()),
        };

        Ok((family_size, skill_level))
    }
}

impl DietaryRestrictionsForm {
    /// Validate and convert to domain objects
    pub fn to_domain(&self) -> Result<Vec<DietaryRestriction>, String> {
        let mut restrictions = Vec::new();
        let restriction_strings = self.get_restrictions();

        for restriction_str in &restriction_strings {
            let restriction = match restriction_str.as_str() {
                "Vegetarian" => DietaryRestriction::Vegetarian,
                "Vegan" => DietaryRestriction::Vegan,
                "GlutenFree" => DietaryRestriction::GlutenFree,
                "DairyFree" => DietaryRestriction::DairyFree,
                "NutFree" => DietaryRestriction::NutFree,
                "SoyFree" => DietaryRestriction::SoyFree,
                "LowSodium" => DietaryRestriction::LowSodium,
                "LowCarb" => DietaryRestriction::LowCarb,
                "Keto" => DietaryRestriction::Keto,
                "Paleo" => DietaryRestriction::Paleo,
                _ => return Err(format!("Invalid dietary restriction: {}", restriction_str)),
            };
            restrictions.push(restriction);
        }

        // Business validation rules
        if restrictions.len() > 5 {
            return Err("Too many dietary restrictions selected (maximum 5)".to_string());
        }

        // Check for conflicting restrictions
        if restrictions.contains(&DietaryRestriction::Vegetarian)
            && restrictions.contains(&DietaryRestriction::Vegan)
        {
            return Err(
                "Cannot be both Vegetarian and Vegan - Vegan includes Vegetarian".to_string(),
            );
        }

        Ok(restrictions)
    }

    /// Validate for conflicts and return conflict descriptions
    pub fn validate_conflicts(&self) -> Option<Vec<String>> {
        let restrictions = self.get_restrictions();
        let mut conflicts = Vec::new();

        if restrictions.contains(&"Vegetarian".to_string())
            && restrictions.contains(&"Vegan".to_string())
        {
            conflicts.push("Vegan diet already includes vegetarian restrictions".to_string());
        }

        if restrictions.len() > 5 {
            conflicts.push(format!(
                "You have selected {} restrictions, but maximum is 5",
                restrictions.len()
            ));
        }

        if conflicts.is_empty() {
            None
        } else {
            Some(conflicts)
        }
    }
}

/// Profile update handler with TwinSpark response
pub async fn update_profile_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<ProfileUpdateForm>,
) -> Result<Html<String>, StatusCode> {
    // Extract user ID from session cookie
    let user_id = extract_user_id_from_headers(&headers)?;
    tracing::info!("Profile update request for user: {} - Family Size: {}, Skill: {}, Weekday: {}min, Weekend: {}min", 
        user_id, form.family_size, form.cooking_skill_level, form.weekday_cooking_minutes, form.weekend_cooking_minutes);

    // Validate form data
    if let Err(validation_errors) = form.validate() {
        let errors: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error.message.as_deref().unwrap_or("Invalid value")
                    )
                })
            })
            .collect();

        let _field_errors: HashMap<String, String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_msg = errors
                    .first()
                    .and_then(|e| e.message.as_deref())
                    .unwrap_or("Invalid value");
                (field.to_string(), error_msg.to_string())
            })
            .collect();

        let error_html = format!(
            r#"<div id="profile-form-container">
                <div class="mb-6 rounded-md bg-red-50 p-4">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            <h3 class="text-sm font-medium text-red-800">Validation errors:</h3>
                            <div class="mt-2 text-sm text-red-700">
                                <ul class="list-disc pl-5">
                                    {}
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>"#,
            errors
                .iter()
                .map(|e| format!("<li>{}</li>", e))
                .collect::<Vec<_>>()
                .join("")
        );

        return Ok(Html(error_html));
    }

    // Convert to domain objects
    let (family_size, skill_level) = match form.to_domain() {
        Ok(result) => result,
        Err(error) => {
            let error_html = format!(
                r#"<div id="profile-form-container">
                    <div class="mb-6 rounded-md bg-red-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">Domain validation error:</h3>
                                <div class="mt-2 text-sm text-red-700">
                                    <p>{}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                error
            );
            return Ok(Html(error_html));
        }
    };

    // Execute profile update command
    let command = UpdateUserProfileCommand::new(
        user_id,
        family_size,
        skill_level,
        form.weekday_cooking_minutes,
        form.weekend_cooking_minutes,
    );

    let db_pool = app_state
        .health_state
        .db_pool
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let command_handler = ProfileCommandHandler::new(db_pool.clone());

    match command_handler.handle_update_profile(command).await {
        Ok(_response) => {
            tracing::info!("Profile update successful for user: {} - Family Size: {}, Skill: {}, Weekday: {}min, Weekend: {}min", 
                user_id, family_size.value, skill_level, form.weekday_cooking_minutes, form.weekend_cooking_minutes);
            let success_html = format!(
                r#"<div id="profile-form-container">
                    <div class="mb-6 rounded-md bg-green-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-green-800">Profile updated successfully!</h3>
                                <div class="mt-2 text-sm text-green-700">
                                    <p>Your cooking preferences have been saved. Family: {} people, Skill: {}, Weekdays: {}min, Weekends: {}min</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                family_size.value,
                skill_level,
                form.weekday_cooking_minutes,
                form.weekend_cooking_minutes
            );

            Ok(Html(success_html))
        }
        Err(error) => {
            let error_html = format!(
                r#"<div id="profile-form-container">
                    <div class="mb-6 rounded-md bg-red-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">Failed to update profile</h3>
                                <div class="mt-2 text-sm text-red-700">
                                    <p>Error: {}</p>
                                    <p>Please try again or contact support if the problem persists.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                error
            );
            Ok(Html(error_html))
        }
    }
}

/// Dietary restrictions update handler
pub async fn update_dietary_restrictions_handler(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Html<String>, StatusCode> {
    // Extract user ID from session cookie
    let user_id = extract_user_id_from_headers(&headers)?;

    // Parse form data manually to handle multiple values with same name
    let body_str = String::from_utf8_lossy(&body);
    let mut dietary_restrictions = Vec::new();

    for pair in body_str.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = urlencoding::decode(key).map_err(|_| StatusCode::BAD_REQUEST)?;
            let decoded_value = urlencoding::decode(value).map_err(|_| StatusCode::BAD_REQUEST)?;

            if decoded_key == "dietary_restrictions[]" || decoded_key == "dietary_restrictions" {
                dietary_restrictions.push(decoded_value.to_string());
            }
        }
    }

    let form = DietaryRestrictionsForm {
        dietary_restrictions,
    };

    // Check for conflicts first
    if let Some(conflicts) = form.validate_conflicts() {
        let error_html = format!(
            r#"<div id="dietary-form-container">
                <div class="mb-6 rounded-md bg-red-50 p-4">
                    <div class="flex">
                        <div class="flex-shrink-0">
                            <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                            </svg>
                        </div>
                        <div class="ml-3">
                            <h3 class="text-sm font-medium text-red-800">Dietary restriction conflicts:</h3>
                            <div class="mt-2 text-sm text-red-700">
                                <ul class="list-disc pl-5">
                                    {}
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>
            </div>"#,
            conflicts
                .iter()
                .map(|c| format!("<li>{}</li>", c))
                .collect::<Vec<_>>()
                .join("")
        );
        return Ok(Html(error_html));
    }

    // Validate and convert dietary restrictions
    let restrictions = match form.to_domain() {
        Ok(restrictions) => restrictions,
        Err(error) => {
            let error_html = format!(
                r#"<div id="dietary-form-container">
                    <div class="mb-6 rounded-md bg-red-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">Dietary restriction error:</h3>
                                <div class="mt-2 text-sm text-red-700">
                                    <p>{}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                error
            );
            return Ok(Html(error_html));
        }
    };

    // Execute dietary restrictions update command
    let command = ChangeDietaryRestrictionsCommand::new(user_id, restrictions.clone());
    let db_pool = app_state
        .health_state
        .db_pool
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let command_handler = ProfileCommandHandler::new(db_pool.clone());

    match command_handler
        .handle_dietary_restrictions_change(command)
        .await
    {
        Ok(_response) => {
            let updated_restrictions: Vec<String> =
                restrictions.iter().map(|r| format!("{:?}", r)).collect();

            let success_html = format!(
                r#"<div id="dietary-form-container">
                    <div class="mb-6 rounded-md bg-green-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-green-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-green-800">Dietary preferences updated!</h3>
                                <div class="mt-2 text-sm text-green-700">
                                    <p>Your dietary restrictions have been saved: {}</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                if updated_restrictions.is_empty() {
                    "No restrictions".to_string()
                } else {
                    updated_restrictions.join(", ")
                }
            );

            Ok(Html(success_html))
        }
        Err(error) => {
            let error_html = format!(
                r#"<div id="dietary-form-container">
                    <div class="mb-6 rounded-md bg-red-50 p-4">
                        <div class="flex">
                            <div class="flex-shrink-0">
                                <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                                </svg>
                            </div>
                            <div class="ml-3">
                                <h3 class="text-sm font-medium text-red-800">Failed to update dietary restrictions</h3>
                                <div class="mt-2 text-sm text-red-700">
                                    <p>Error: {}</p>
                                    <p>Please try again or contact support if the problem persists.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#,
                error
            );
            Ok(Html(error_html))
        }
    }
}

/// Profile validation endpoint (for TwinSpark sync validation)
pub async fn validate_profile_handler(
    Form(form): Form<ProfileUpdateForm>,
) -> Result<Html<String>, StatusCode> {
    match form.validate() {
        Ok(_) => {
            // Additional domain validation
            if let Err(e) = form.to_domain() {
                let error_html = format!(
                    r#"<div id="validation-errors" class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                        <p class="text-sm">{}</p>
                    </div>"#,
                    e
                );
                return Ok(Html(error_html));
            }

            let success_html = r#"<div id="validation-errors" class="bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded">
                <p class="text-sm">✓ Profile data is valid</p>
            </div>"#;
            Ok(Html(success_html.to_string()))
        }
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .field_errors()
                .iter()
                .flat_map(|(field, errors)| {
                    let field_name = field.to_string();
                    errors.iter().map(move |error| {
                        format!(
                            "{}: {}",
                            field_name,
                            error.message.as_deref().unwrap_or("Invalid value")
                        )
                    })
                })
                .collect();

            let error_html = format!(
                r#"<div id="validation-errors" class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                    {}
                </div>"#,
                error_messages
                    .iter()
                    .map(|msg| format!("<p class=\"text-sm\">{}</p>", msg))
                    .collect::<Vec<_>>()
                    .join("")
            );

            Ok(Html(error_html))
        }
    }
}

/// Profile page handler (displays current profile)
pub async fn profile_page(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<Html<String>, StatusCode> {
    // Extract user ID from session cookie
    let user_id = extract_user_id_from_headers(&headers)?;
    tracing::info!("Loading profile page for user: {}", user_id);

    // Get actual user profile from database
    let profile_data = if let Some(ref query_handler) = app_state.user_query_handler {
        // Try to get real user profile from database
        use imkitchen_user::queries::UserByIdQuery;

        let query = UserByIdQuery::new(user_id);
        match query_handler.handle_user_by_id(query).await {
            Ok(response) if response.found => {
                tracing::info!("User found in database: {}", user_id);
                if let Some(user_view) = response.user {
                    tracing::info!("User profile data from DB - Family Size: {}, Skill: {:?}, Email: {}, Weekday: {}min, Weekend: {}min",
                        user_view.profile.family_size.value,
                        user_view.profile.cooking_skill_level,
                        user_view.email.value,
                        user_view.profile.weekday_cooking_minutes,
                        user_view.profile.weekend_cooking_minutes);
                    ProfilePageData {
                        user_email: user_view.email.value,
                        profile: UserProfileData {
                            family_size: user_view.profile.family_size.value,
                            cooking_skill_level: format!(
                                "{:?}",
                                user_view.profile.cooking_skill_level
                            ),
                            dietary_restrictions: user_view
                                .profile
                                .dietary_restrictions
                                .iter()
                                .map(|r| format!("{:?}", r))
                                .collect(),
                            weekday_cooking_minutes: user_view.profile.weekday_cooking_minutes,
                            weekend_cooking_minutes: user_view.profile.weekend_cooking_minutes,
                        },
                    }
                } else {
                    // If user not found in database, create mock data
                    create_mock_profile_data(user_id, "user@example.com")
                }
            }
            _ => {
                // If user not found in database or error, create mock data
                tracing::warn!("User not found in database, using mock data: {}", user_id);
                create_mock_profile_data(user_id, "user@example.com")
            }
        }
    } else {
        // No query handler available, use mock data
        create_mock_profile_data(user_id, "user@example.com")
    };

    // Simple profile view HTML (in a real app, this would use Askama templates)
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Profile - IMKitchen</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-50">
    <div class="min-h-screen">
        <nav class="bg-white shadow">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between h-16">
                    <div class="flex items-center">
                        <h1 class="text-xl font-bold text-gray-900">IMKitchen</h1>
                    </div>
                    <div class="flex items-center space-x-4">
                        <a href="/dashboard" class="text-gray-600 hover:text-gray-900">Dashboard</a>
                        <a href="/profile/edit" class="text-blue-600 hover:text-blue-800">Edit Profile</a>
                    </div>
                </div>
            </div>
        </nav>
        
        <div class="max-w-3xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
            <div class="bg-white shadow rounded-lg">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold text-gray-900">Profile Information</h2>
                </div>
                <div class="px-6 py-4">
                    <dl class="grid grid-cols-1 gap-x-4 gap-y-4 sm:grid-cols-2">
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Email</dt>
                            <dd class="text-sm text-gray-900">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Family Size</dt>
                            <dd class="text-sm text-gray-900">{} people</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Cooking Skill Level</dt>
                            <dd class="text-sm text-gray-900">{}</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Weekday Cooking Time</dt>
                            <dd class="text-sm text-gray-900">{} minutes</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Weekend Cooking Time</dt>
                            <dd class="text-sm text-gray-900">{} minutes</dd>
                        </div>
                        <div>
                            <dt class="text-sm font-medium text-gray-500">Dietary Restrictions</dt>
                            <dd class="text-sm text-gray-900">{}</dd>
                        </div>
                    </dl>
                </div>
                <div class="px-6 py-4 bg-gray-50 border-t border-gray-200">
                    <a href="/profile/edit" class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700">
                        Edit Profile
                    </a>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"#,
        profile_data.user_email,
        profile_data.profile.family_size,
        profile_data.profile.cooking_skill_level,
        profile_data.profile.weekday_cooking_minutes,
        profile_data.profile.weekend_cooking_minutes,
        if profile_data.profile.dietary_restrictions.is_empty() {
            "None".to_string()
        } else {
            profile_data.profile.dietary_restrictions.join(", ")
        }
    );

    Ok(Html(html))
}

/// Helper function to create mock profile data when database lookup fails
fn create_mock_profile_data(_user_id: uuid::Uuid, email: &str) -> ProfilePageData {
    ProfilePageData {
        user_email: email.to_string(),
        profile: UserProfileData {
            family_size: 4,
            cooking_skill_level: "Intermediate".to_string(),
            dietary_restrictions: vec!["Vegetarian".to_string()],
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        },
    }
}

/// Profile edit page handler
pub async fn profile_edit_page(
    State(_app_state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    // Simple profile edit form HTML (in a real app, this would use the Askama templates we created)
    let html = r####"<!DOCTYPE html>
<html>
<head>
    <title>Edit Profile - IMKitchen</title>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="/static/js/twinspark.js" defer></script>
</head>
<body class="bg-gray-50">
    <div class="min-h-screen">
        <nav class="bg-white shadow">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex justify-between h-16">
                    <div class="flex items-center">
                        <h1 class="text-xl font-bold text-gray-900">IMKitchen</h1>
                    </div>
                    <div class="flex items-center space-x-4">
                        <a href="/dashboard" class="text-gray-600 hover:text-gray-900">Dashboard</a>
                        <a href="/profile" class="text-gray-600 hover:text-gray-900">View Profile</a>
                    </div>
                </div>
            </div>
        </nav>
        
        <div class="max-w-3xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
            <div class="bg-white shadow rounded-lg">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold text-gray-900">Edit Profile</h2>
                    <p class="text-sm text-gray-600">Update your cooking preferences and family details.</p>
                </div>
                
                <div id="profile-update-result" class="px-6 py-4"></div>
                
                <form ts-req="/profile/update" ts-target="#profile-update-result" class="px-6 py-4 space-y-6">
                    <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
                        <div>
                            <label for="family_size" class="block text-sm font-medium text-gray-700">Family Size</label>
                            <select name="family_size" id="family_size" class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500">
                                <option value="1">1 person</option>
                                <option value="2">2 people</option>
                                <option value="3">3 people</option>
                                <option value="4" selected>4 people</option>
                                <option value="5">5 people</option>
                                <option value="6">6 people</option>
                                <option value="7">7 people</option>
                                <option value="8">8 people</option>
                            </select>
                        </div>
                        
                        <div>
                            <label for="cooking_skill_level" class="block text-sm font-medium text-gray-700">Cooking Skill Level</label>
                            <select name="cooking_skill_level" id="cooking_skill_level" class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500">
                                <option value="Beginner">Beginner</option>
                                <option value="Intermediate" selected>Intermediate</option>
                                <option value="Advanced">Advanced</option>
                            </select>
                        </div>
                        
                        <div>
                            <label for="weekday_cooking_minutes" class="block text-sm font-medium text-gray-700">Weekday Cooking Time (minutes)</label>
                            <input type="number" name="weekday_cooking_minutes" id="weekday_cooking_minutes" min="5" max="480" value="30" 
                                   class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500">
                        </div>
                        
                        <div>
                            <label for="weekend_cooking_minutes" class="block text-sm font-medium text-gray-700">Weekend Cooking Time (minutes)</label>
                            <input type="number" name="weekend_cooking_minutes" id="weekend_cooking_minutes" min="5" max="480" value="60" 
                                   class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500">
                        </div>
                    </div>
                    
                    <div class="pt-4 border-t border-gray-200">
                        <button type="submit" class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2">
                            Update Profile
                        </button>
                        <a href="/profile" class="ml-3 bg-gray-300 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-400">
                            Cancel
                        </a>
                    </div>
                </form>
                
                <div class="px-6 py-4 border-t border-gray-200">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">Dietary Restrictions</h3>
                    <div id="dietary-update-result" class="mb-4"></div>
                    
                    <form ts-req="/profile/dietary" ts-target="#dietary-update-result" class="space-y-3">
                        <div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="Vegetarian" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Vegetarian</span>
                            </label>
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="Vegan" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Vegan</span>
                            </label>
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="GlutenFree" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Gluten Free</span>
                            </label>
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="DairyFree" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Dairy Free</span>
                            </label>
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="NutFree" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Nut Free</span>
                            </label>
                            <label class="flex items-center">
                                <input type="checkbox" name="dietary_restrictions[]" value="LowSodium" class="rounded text-blue-600">
                                <span class="ml-2 text-sm text-gray-700">Low Sodium</span>
                            </label>
                        </div>
                        
                        <button type="submit" class="bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-green-500 focus:ring-offset-2">
                            Update Dietary Restrictions
                        </button>
                    </form>
                </div>
            </div>
        </div>
    </div>
</body>
</html>"####;

    Ok(Html(html.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::{DietaryRestriction, SkillLevel};

    #[test]
    fn test_profile_update_form_validation() {
        // Valid form
        let valid_form = ProfileUpdateForm {
            family_size: 4,
            cooking_skill_level: "Intermediate".to_string(),
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        };
        assert!(valid_form.validate().is_ok());

        // Invalid family size (too small)
        let invalid_form = ProfileUpdateForm {
            family_size: 0,
            cooking_skill_level: "Beginner".to_string(),
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        };
        assert!(invalid_form.validate().is_err());

        // Invalid cooking time (too short)
        let invalid_time_form = ProfileUpdateForm {
            family_size: 2,
            cooking_skill_level: "Beginner".to_string(),
            weekday_cooking_minutes: 2, // Too short
            weekend_cooking_minutes: 60,
        };
        assert!(invalid_time_form.validate().is_err());
    }

    #[test]
    fn test_profile_form_to_domain_conversion() {
        let form = ProfileUpdateForm {
            family_size: 6,
            cooking_skill_level: "Advanced".to_string(),
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 120,
        };

        let (family_size, skill_level) = form.to_domain().unwrap();
        assert_eq!(family_size.value, 6);
        assert_eq!(skill_level, SkillLevel::Advanced);
    }

    #[test]
    fn test_dietary_restrictions_form_validation() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: vec!["Vegetarian".to_string(), "GlutenFree".to_string()],
        };

        let restrictions = form.to_domain().unwrap();
        assert_eq!(restrictions.len(), 2);
        assert!(restrictions.contains(&DietaryRestriction::Vegetarian));
        assert!(restrictions.contains(&DietaryRestriction::GlutenFree));
    }

    #[test]
    fn test_invalid_dietary_restriction() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: vec!["InvalidRestriction".to_string()],
        };

        assert!(form.to_domain().is_err());
    }

    #[test]
    fn test_dietary_restrictions_conflicts() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: vec!["Vegetarian".to_string(), "Vegan".to_string()],
        };

        assert!(form.validate_conflicts().is_some());
        assert!(form.to_domain().is_err());
    }

    #[test]
    fn test_too_many_dietary_restrictions() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: vec![
                "Vegetarian".to_string(),
                "GlutenFree".to_string(),
                "DairyFree".to_string(),
                "NutFree".to_string(),
                "SoyFree".to_string(),
                "LowSodium".to_string(), // 6 restrictions - too many
            ],
        };

        assert!(form.validate_conflicts().is_some());
        assert!(form.to_domain().is_err());
    }

    #[test]
    fn test_invalid_skill_level() {
        let form = ProfileUpdateForm {
            family_size: 4,
            cooking_skill_level: "Expert".to_string(), // Invalid
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        };

        assert!(form.to_domain().is_err());
    }
}
