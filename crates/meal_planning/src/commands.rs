use crate::aggregate::MealPlanAggregate;
use crate::algorithm::{MealPlanningAlgorithm, RecipeForPlanning, UserConstraints};
use crate::error::MealPlanningError;
use crate::events::{MealAssignment, MealPlanRegenerated};
use crate::rotation::RotationState;
use chrono::Utc;

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

/// Regenerate an entire meal plan (Story 3.7)
///
/// This function implements the domain logic for regenerating all 21 meal assignments
/// while preserving rotation state and applying the same constraints as initial generation.
///
/// **Flow:**
/// 1. Load MealPlan aggregate from evento event stream
/// 2. Validate meal plan is active
/// 3. Load user profile and favorite recipes
/// 4. Load current rotation state (preserved, NOT reset)
/// 5. Run MealPlanningAlgorithm with same constraints
/// 6. Generate new assignments (different from current)
/// 7. Emit MealPlanRegenerated event
/// 8. Projection handler updates read model
///
/// **Rotation Integrity:**
/// - Rotation state preserved (cycle_number unchanged)
/// - Previously used recipes in current cycle NOT reassigned
/// - New recipe usage tracked via rotation state update
pub async fn regenerate_meal_plan<E: evento::Executor>(
    cmd: RegenerateMealPlanCommand,
    executor: &E,
    favorite_recipes: Vec<RecipeForPlanning>,
    user_constraints: UserConstraints,
) -> Result<(), MealPlanningError> {
    // Load the MealPlan aggregate from event stream
    let loaded = evento::load::<MealPlanAggregate, _>(executor, &cmd.meal_plan_id)
        .await
        .map_err(|e| {
            MealPlanningError::EventoError(format!(
                "Failed to load meal plan {}: {}",
                cmd.meal_plan_id, e
            ))
        })?;

    let aggregate = loaded.item;

    // Validate: Check that meal plan exists
    if aggregate.meal_plan_id.is_empty() {
        return Err(MealPlanningError::MealPlanNotFound(
            cmd.meal_plan_id.clone(),
        ));
    }

    // Validate: Check that meal plan is active
    if aggregate.status != "active" {
        return Err(MealPlanningError::MealPlanNotActive(
            cmd.meal_plan_id.clone(),
        ));
    }

    // Validate: Check that user_id matches (authorization)
    if aggregate.user_id != cmd.user_id {
        return Err(MealPlanningError::UnauthorizedAccess(
            cmd.user_id.clone(),
            cmd.meal_plan_id.clone(),
        ));
    }

    // Validate: Minimum 7 favorite recipes required
    if favorite_recipes.len() < 7 {
        return Err(MealPlanningError::InsufficientRecipes {
            minimum: 7,
            current: favorite_recipes.len(),
        });
    }

    // Load current rotation state (PRESERVED - do NOT reset cycle)
    let rotation_state = RotationState::from_json(&aggregate.rotation_state_json).map_err(|e| {
        MealPlanningError::RotationStateError(format!("Failed to parse rotation state: {}", e))
    })?;

    // Calculate start date (next Monday) - Story 3.13: Next-week-only meal planning
    // Business rule: Regeneration always creates plans for next week, not current week
    let start_date = crate::calculate_next_week_start()
        .format("%Y-%m-%d")
        .to_string();

    // Generate new meal plan using algorithm (same constraints as initial generation)
    // Use different seed to ensure variety (timestamp-based)
    let (new_assignments, updated_rotation_state) = MealPlanningAlgorithm::generate(
        &start_date,
        favorite_recipes,
        user_constraints,
        rotation_state,
        None, // No seed = timestamp-based randomization for variety
    )?;

    // Emit MealPlanRegenerated event
    let regenerated_at = Utc::now().to_rfc3339();
    let event_data = MealPlanRegenerated {
        new_assignments,
        rotation_state_json: updated_rotation_state.to_json().map_err(|e| {
            MealPlanningError::RotationStateError(format!(
                "Failed to serialize rotation state: {}",
                e
            ))
        })?,
        regeneration_reason: cmd.regeneration_reason,
        regenerated_at,
    };

    evento::save::<MealPlanAggregate>(&cmd.meal_plan_id)
        .data(&event_data)
        .map_err(|e| {
            MealPlanningError::EventoError(format!(
                "Failed to encode MealPlanRegenerated event: {}",
                e
            ))
        })?
        .metadata(&true)
        .map_err(|e| MealPlanningError::EventoError(format!("Failed to encode metadata: {}", e)))?
        .commit(executor)
        .await
        .map_err(|e| {
            MealPlanningError::EventoError(format!(
                "Failed to commit MealPlanRegenerated event: {}",
                e
            ))
        })?;

    Ok(())
}
