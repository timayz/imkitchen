/// Story 8.5: User Preferences Update Route (API)
///
/// RESTful JSON API endpoint for updating user meal planning preferences.
/// Integrates with evento event sourcing for UserMealPlanningPreferencesUpdated events.
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Extension,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

use crate::middleware::auth::Auth;
use crate::routes::AppState;

/// Request payload for meal planning preferences update (Story 8.5 AC-2)
///
/// All fields are validated using the `validator` crate with declarative rules.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MealPlanningPreferences {
    #[validate(range(min = 1, message = "Must be greater than 0"))]
    pub max_prep_time_weeknight: u32,

    #[validate(range(min = 1, message = "Must be greater than 0"))]
    pub max_prep_time_weekend: u32,

    pub avoid_consecutive_complex: bool,

    #[validate(range(min = 0.0, max = 1.0, message = "Must be between 0.0 and 1.0"))]
    pub cuisine_variety_weight: f32,
}

/// JSON response for preferences update (Story 8.5 AC-4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesResponse {
    pub preferences: MealPlanningPreferences,
    pub message: String,
}

/// API-specific error types with JSON responses (Story 8.5 AC-5)
#[derive(Debug)]
pub enum PreferencesApiError {
    ValidationFailed(HashMap<String, String>),
    InternalServerError(String),
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for PreferencesApiError {
    fn from(e: sqlx::Error) -> Self {
        PreferencesApiError::DatabaseError(e)
    }
}

impl IntoResponse for PreferencesApiError {
    fn into_response(self) -> Response {
        match self {
            PreferencesApiError::ValidationFailed(field_errors) => {
                let error_response = serde_json::json!({
                    "error": "ValidationFailed",
                    "message": "Invalid preferences provided.",
                    "details": field_errors,
                });

                (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
            }
            PreferencesApiError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                let error_response = serde_json::json!({
                    "error": "DatabaseError",
                    "message": "Database error occurred. Please try again later.",
                });

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
            PreferencesApiError::InternalServerError(msg) => {
                tracing::error!("Internal server error: {}", msg);
                let error_response = serde_json::json!({
                    "error": "InternalServerError",
                    "message": msg,
                });

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        }
    }
}

/// PUT /profile/meal-planning-preferences route handler (Story 8.5 AC-1)
///
/// Updates user's meal planning preferences by validating input and emitting
/// a UserMealPlanningPreferencesUpdated evento event.
///
/// # Authentication
/// Protected by JWT cookie middleware (AC-1)
///
/// # Validation
/// - max_prep_time_weeknight: Must be > 0 (AC-2)
/// - max_prep_time_weekend: Must be > 0 (AC-2)
/// - cuisine_variety_weight: Must be 0.0 <= value <= 1.0 (AC-2)
///
/// # Returns
/// - 200 OK: JSON with updated preferences + message (AC-4)
/// - 400 Bad Request: ValidationFailed error with field-specific messages (AC-5)
/// - 500 Internal Server Error: Database or event emission errors
#[tracing::instrument(skip(state), fields(user_id = %auth.user_id))]
pub async fn update_meal_planning_preferences(
    State(state): State<AppState>,
    Extension(auth): Extension<Auth>, // AC-1: Extract user_id from JWT
    Json(payload): Json<MealPlanningPreferences>, // AC-2: Extract JSON payload
) -> Result<Json<PreferencesResponse>, PreferencesApiError> {
    let user_id = &auth.user_id;

    tracing::info!(user_id = %user_id, "Meal planning preferences update requested");

    // AC-2: Validate input using validator crate
    tracing::debug!(user_id = %user_id, "Validating preferences payload");

    if let Err(validation_errors) = payload.validate() {
        // AC-5: Build field-specific error messages
        let field_errors: HashMap<String, String> = validation_errors
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_message = errors
                    .first()
                    .and_then(|e| e.message.as_ref())
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Validation failed".to_string());
                (field.to_string(), error_message)
            })
            .collect();

        tracing::warn!(
            user_id = %user_id,
            errors = ?field_errors,
            "Preferences validation failed"
        );

        return Err(PreferencesApiError::ValidationFailed(field_errors));
    }

    tracing::debug!(user_id = %user_id, "Preferences validation passed");

    // AC-3: Emit UserMealPlanningPreferencesUpdated evento event
    // Only update the 4 fields specified in Story 8.5 (partial update)
    let event = user::events::UserMealPlanningPreferencesUpdated {
        dietary_restrictions: None,   // Not updated in Story 8.5
        household_size: None,         // Not updated in Story 8.5
        skill_level: None,            // Not updated in Story 8.5
        weeknight_availability: None, // Not updated in Story 8.5
        max_prep_time_weeknight: payload.max_prep_time_weeknight,
        max_prep_time_weekend: payload.max_prep_time_weekend,
        avoid_consecutive_complex: payload.avoid_consecutive_complex,
        cuisine_variety_weight: payload.cuisine_variety_weight,
        updated_at: Utc::now().to_rfc3339(),
    };

    // Commit event to evento using the UserAggregate
    evento::save::<user::UserAggregate>(user_id.to_string())
        .data(&event)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to encode UserMealPlanningPreferencesUpdated event"
            );
            PreferencesApiError::InternalServerError(format!("Failed to encode event: {}", e))
        })?
        .metadata(&true)
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to encode event metadata"
            );
            PreferencesApiError::InternalServerError(format!("Failed to encode metadata: {}", e))
        })?
        .commit(&state.evento_executor)
        .await
        .map_err(|e| {
            tracing::error!(
                user_id = %user_id,
                error = %e,
                "Failed to commit UserMealPlanningPreferencesUpdated event to evento"
            );
            PreferencesApiError::InternalServerError(format!("Failed to commit event: {}", e))
        })?;

    tracing::info!(
        user_id = %user_id,
        "UserMealPlanningPreferencesUpdated event emitted successfully"
    );

    // AC-4: Build JSON response
    let response = PreferencesResponse {
        preferences: payload,
        message: "Meal planning preferences updated. Changes will apply to your next meal plan generation.".to_string(),
    };

    tracing::info!(user_id = %user_id, "Preferences updated successfully");

    Ok(Json(response))
}
