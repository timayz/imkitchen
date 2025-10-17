use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// MealType enum for meal slot classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

impl MealType {
    pub fn as_str(&self) -> &str {
        match self {
            MealType::Breakfast => "breakfast",
            MealType::Lunch => "lunch",
            MealType::Dinner => "dinner",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "breakfast" => Ok(MealType::Breakfast),
            "lunch" => Ok(MealType::Lunch),
            "dinner" => Ok(MealType::Dinner),
            _ => Err(format!("Invalid meal type: {}", s)),
        }
    }
}

/// Meal assignment representing a single recipe assigned to a meal slot
///
/// Note: meal_type stored as String for bincode compatibility (like complexity in RecipeTagged)
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MealAssignment {
    pub date: String,        // ISO 8601 date (YYYY-MM-DD)
    pub meal_type: String,   // "breakfast", "lunch", or "dinner"
    pub recipe_id: String,   // Recipe assigned to this slot
    pub prep_required: bool, // True if recipe has advance_prep_hours > 0
}

/// MealPlanGenerated event emitted when a new meal plan is generated
///
/// This event captures the complete meal plan generation result including all
/// 21 meal assignments (7 days × 3 meals) and rotation state tracking.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanGenerated {
    pub user_id: String,                       // Owner of the meal plan
    pub start_date: String,                    // ISO 8601 date (Monday of the week)
    pub meal_assignments: Vec<MealAssignment>, // 21 assignments (7 days × 3 meals)
    pub rotation_state_json: String,           // JSON serialized RotationState
    pub generated_at: String,                  // RFC3339 formatted timestamp
}

/// RecipeUsedInRotation event emitted when a recipe is used in meal plan generation
///
/// This event tracks recipe usage for the rotation system, ensuring recipes are
/// used exactly once before repeating. Emitted for each recipe assigned during generation.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeUsedInRotation {
    pub recipe_id: String, // Recipe that was used
    pub cycle_number: u32, // Rotation cycle number
    pub used_at: String,   // RFC3339 formatted timestamp
}

/// MealPlanArchived event emitted when a meal plan is archived
///
/// Meal plans are archived when a new plan is generated or when the week ends.
/// Only one meal plan can be active per user at a time.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanArchived {
    pub archived_at: String, // RFC3339 formatted timestamp
}

/// MealReplaced event emitted when a user replaces a specific meal slot
///
/// This event supports the "Replace Meal" feature (Story 3.2) allowing users
/// to swap out a single meal while preserving the rest of the plan.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealReplaced {
    pub date: String,          // ISO 8601 date of the meal slot
    pub meal_type: String,     // "breakfast", "lunch", or "dinner"
    pub old_recipe_id: String, // Recipe being replaced
    pub new_recipe_id: String, // Replacement recipe
    pub replaced_at: String,   // RFC3339 formatted timestamp
}
