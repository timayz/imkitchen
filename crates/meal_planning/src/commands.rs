use crate::events::MealAssignment;

/// GenerateMealPlanCommand is the command to generate a new weekly meal plan
///
/// This command triggers the meal planning algorithm to assign recipes to 21 meal slots
/// (7 days × 3 meals) based on user constraints and recipe rotation state.
#[derive(Debug, Clone)]
pub struct GenerateMealPlanCommand {
    pub user_id: String,
    pub start_date: String, // ISO 8601 date (Monday of the week)
    pub meal_assignments: Vec<MealAssignment>, // Pre-calculated assignments from algorithm
    pub rotation_state_json: String, // JSON serialized RotationState
}

/// ReplaceMealCommand is the command to replace a single meal slot
///
/// This command allows users to swap out one meal while keeping the rest of the plan intact.
/// Used in Story 3.2 "Replace Individual Meal" feature.
#[derive(Debug, Clone)]
pub struct ReplaceMealCommand {
    pub meal_plan_id: String,
    pub date: String,          // ISO 8601 date
    pub meal_type: String,     // "breakfast", "lunch", or "dinner"
    pub new_recipe_id: String, // Replacement recipe
}

/// ArchiveMealPlanCommand is the command to archive a meal plan
///
/// This command marks a meal plan as archived when it's no longer active.
/// Only one meal plan can be active per user at a time.
#[derive(Debug, Clone)]
pub struct ArchiveMealPlanCommand {
    pub meal_plan_id: String,
}

/// RegenerateMealPlanCommand is the command to regenerate entire meal plan (Story 3.7)
///
/// This command triggers full regeneration of all 21 meal assignments using the
/// same algorithm as initial generation, while preserving rotation state.
#[derive(Debug, Clone)]
pub struct RegenerateMealPlanCommand {
    pub meal_plan_id: String,
    pub user_id: String,
    pub regeneration_reason: Option<String>, // Optional reason for regeneration
}
