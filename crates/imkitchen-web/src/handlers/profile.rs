// Profile management handlers with TwinSpark integration

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::Html,
};
use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};
use imkitchen_user::{
    commands::{UpdateUserProfileCommand, ChangeDietaryRestrictionsCommand, ProfileCommandHandler},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use crate::AppState;

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
    #[validate(range(min = 1, max = 8, message = "Family size must be between 1 and 8 people"))]
    pub family_size: u8,
    
    pub cooking_skill_level: String, // Will be validated as SkillLevel enum
    
    #[validate(range(min = 5, max = 480, message = "Weekday cooking time must be between 5 and 480 minutes"))]
    pub weekday_cooking_minutes: u32,
    
    #[validate(range(min = 5, max = 480, message = "Weekend cooking time must be between 5 and 480 minutes"))]
    pub weekend_cooking_minutes: u32,
}

/// Dietary restrictions update form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DietaryRestrictionsForm {
    pub dietary_restrictions: Option<Vec<String>>, // Will be validated as DietaryRestriction enums
}

impl DietaryRestrictionsForm {
    pub fn get_restrictions(&self) -> Vec<String> {
        self.dietary_restrictions.clone().unwrap_or_default()
    }
}

impl ProfileUpdateForm {
    /// Validate and convert to domain objects
    pub fn to_domain(&self) -> Result<(FamilySize, SkillLevel), String> {
        // Validate family size
        let family_size = FamilySize::new(self.family_size)
            .map_err(|_| "Invalid family size".to_string())?;

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
            && restrictions.contains(&DietaryRestriction::Vegan) {
            return Err("Cannot be both Vegetarian and Vegan - Vegan includes Vegetarian".to_string());
        }
        
        Ok(restrictions)
    }
    
    /// Validate for conflicts and return conflict descriptions
    pub fn validate_conflicts(&self) -> Option<Vec<String>> {
        let restrictions = self.get_restrictions();
        let mut conflicts = Vec::new();
        
        if restrictions.contains(&"Vegetarian".to_string()) 
            && restrictions.contains(&"Vegan".to_string()) {
            conflicts.push("Vegan diet already includes vegetarian restrictions".to_string());
        }
        
        if restrictions.len() > 5 {
            conflicts.push(format!("You have selected {} restrictions, but maximum is 5", restrictions.len()));
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
    Form(form): Form<ProfileUpdateForm>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Get actual user ID from session/auth
    let user_id = Uuid::new_v4(); // Placeholder
    
    // Validate form data
    if let Err(validation_errors) = form.validate() {
        let errors: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_deref().unwrap_or("Invalid value"))
                })
            })
            .collect();
        
        let _field_errors: HashMap<String, String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_msg = errors.first()
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
            errors.iter()
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
    
    let db_pool = app_state.health_state.db_pool.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let command_handler = ProfileCommandHandler::new(db_pool.clone());
    
    match command_handler.handle_update_profile(command).await {
        Ok(_response) => {
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
        Err(_error) => {
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
                                    <p>Please try again or contact support if the problem persists.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#
            );
            Ok(Html(error_html))
        }
    }
}

/// Dietary restrictions update handler
pub async fn update_dietary_restrictions_handler(
    State(app_state): State<AppState>,
    Form(form): Form<DietaryRestrictionsForm>,
) -> Result<Html<String>, StatusCode> {
    // TODO: Get actual user ID from session/auth
    let user_id = Uuid::new_v4(); // Placeholder
    
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
            conflicts.iter()
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
    let db_pool = app_state.health_state.db_pool.as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let command_handler = ProfileCommandHandler::new(db_pool.clone());
    
    match command_handler.handle_dietary_restrictions_change(command).await {
        Ok(_response) => {
            let updated_restrictions: Vec<String> = restrictions.iter()
                .map(|r| format!("{:?}", r))
                .collect();
            
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
        Err(_error) => {
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
                                    <p>Please try again or contact support if the problem persists.</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>"#
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
                        format!("{}: {}", field_name, error.message.as_deref().unwrap_or("Invalid value"))
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
            dietary_restrictions: Some(vec![
                "Vegetarian".to_string(),
                "GlutenFree".to_string(),
            ]),
        };

        let restrictions = form.to_domain().unwrap();
        assert_eq!(restrictions.len(), 2);
        assert!(restrictions.contains(&DietaryRestriction::Vegetarian));
        assert!(restrictions.contains(&DietaryRestriction::GlutenFree));
    }

    #[test]
    fn test_invalid_dietary_restriction() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: Some(vec!["InvalidRestriction".to_string()]),
        };

        assert!(form.to_domain().is_err());
    }
    
    #[test]
    fn test_dietary_restrictions_conflicts() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: Some(vec![
                "Vegetarian".to_string(),
                "Vegan".to_string(),
            ]),
        };

        assert!(form.validate_conflicts().is_some());
        assert!(form.to_domain().is_err());
    }
    
    #[test]
    fn test_too_many_dietary_restrictions() {
        let form = DietaryRestrictionsForm {
            dietary_restrictions: Some(vec![
                "Vegetarian".to_string(),
                "GlutenFree".to_string(),
                "DairyFree".to_string(),
                "NutFree".to_string(),
                "SoyFree".to_string(),
                "LowSodium".to_string(), // 6 restrictions - too many
            ]),
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