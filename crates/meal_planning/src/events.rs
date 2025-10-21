use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// CourseType enum for course slot classification (AC-4)
/// Renamed from MealType to reflect new course-based meal planning model
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CourseType {
    Appetizer,
    MainCourse,
    Dessert,
}

impl CourseType {
    pub fn as_str(&self) -> &str {
        match self {
            CourseType::Appetizer => "appetizer",
            CourseType::MainCourse => "main_course",
            CourseType::Dessert => "dessert",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "appetizer" => Ok(CourseType::Appetizer),
            "main_course" => Ok(CourseType::MainCourse),
            "dessert" => Ok(CourseType::Dessert),
            // AC-9: Backward compatibility for old data
            "breakfast" => Ok(CourseType::Appetizer),
            "lunch" => Ok(CourseType::MainCourse),
            "dinner" => Ok(CourseType::Dessert),
            _ => Err(format!("Invalid course type: {}", s)),
        }
    }
}

// Keep old MealType as deprecated alias for backward compatibility
#[deprecated(since = "0.5.0", note = "Use CourseType instead")]
pub type MealType = CourseType;

/// Meal assignment representing a single recipe assigned to a course slot
///
/// AC-4: Renamed meal_type to course_type to reflect course-based model
/// Note: course_type stored as String for bincode compatibility (like complexity in RecipeTagged)
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MealAssignment {
    pub date: String,                         // ISO 8601 date (YYYY-MM-DD)
    pub course_type: String, // AC-4: "appetizer", "main_course", or "dessert" (renamed from meal_type)
    pub recipe_id: String,   // Recipe assigned to this slot
    pub prep_required: bool, // True if recipe has advance_prep_hours > 0
    pub assignment_reasoning: Option<String>, // Human-readable explanation of assignment (Story 3.8)
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

/// RotationCycleReset event emitted when all favorites have been used once
///
/// This event triggers the start of a new rotation cycle, allowing all favorite
/// recipes to be used again in subsequent meal plans.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RotationCycleReset {
    pub user_id: String,       // User whose rotation is resetting
    pub old_cycle_number: u32, // Previous cycle number
    pub new_cycle_number: u32, // New cycle number (old + 1)
    pub favorite_count: usize, // Total number of favorite recipes
    pub reset_at: String,      // RFC3339 formatted timestamp
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

/// MealReplaced event emitted when a user replaces a specific course slot
///
/// AC-5: Updated to use course_type instead of meal_type
/// This event supports the "Replace Meal" feature (Story 3.2) allowing users
/// to swap out a single course while preserving the rest of the plan.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealReplaced {
    pub date: String,          // ISO 8601 date of the course slot
    pub course_type: String, // AC-5: "appetizer", "main_course", or "dessert" (renamed from meal_type)
    pub old_recipe_id: String, // Recipe being replaced
    pub new_recipe_id: String, // Replacement recipe
    pub replaced_at: String, // RFC3339 formatted timestamp
}

/// MealPlanRegenerated event emitted when user regenerates entire meal plan (Story 3.7)
///
/// This event replaces all 21 meal assignments with freshly generated recipes
/// while preserving rotation state (doesn't reset cycle).
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanRegenerated {
    pub new_assignments: Vec<MealAssignment>, // Fresh 21 assignments (7 days × 3 meals)
    pub rotation_state_json: String,          // Updated rotation state (cycle preserved)
    pub regeneration_reason: Option<String>,  // Optional reason for regeneration
    pub regenerated_at: String,               // RFC3339 formatted timestamp
}
