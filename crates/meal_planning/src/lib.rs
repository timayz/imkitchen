pub mod aggregate;
pub mod algorithm;
pub mod commands;
pub mod constraints;
pub mod error;
pub mod events;
pub mod read_model;
pub mod rotation;

pub use aggregate::MealPlanAggregate;
pub use algorithm::{MealPlanningAlgorithm, RecipeComplexityCalculator};
pub use commands::{GenerateMealPlanCommand, ReplaceMealCommand};
pub use constraints::{
    AdvancePrepConstraint, AvailabilityConstraint, ComplexityConstraint, Constraint,
    DietaryConstraint, EquipmentConflictConstraint, FreshnessConstraint, MealSlot, MealType,
};
pub use error::MealPlanningError;
pub use events::{MealPlanGenerated, MealReplaced, RecipeUsedInRotation};
pub use read_model::{
    meal_plan_projection, MealAssignmentReadModel, MealPlanQueries, MealPlanReadModel,
};
pub use rotation::{RotationState, RotationSystem};

use chrono::Utc;

/// Replace a meal slot in an existing meal plan (Story 3.6)
///
/// This function implements the domain logic for replacing a single meal slot while
/// respecting rotation constraints and maintaining data integrity.
///
/// **Flow:**
/// 1. Load MealPlan aggregate from evento event stream
/// 2. Validate new recipe is not already used in current rotation
/// 3. Emit MealReplaced event
/// 4. Projection handler updates read model and rotation state
///
/// **Rotation Integrity:**
/// - Old recipe is unmarked (returned to pool) via aggregate event handler
/// - New recipe is marked as used via aggregate event handler
/// - Rotation state updated atomically in MealPlan aggregate
pub async fn replace_meal<E: evento::Executor>(
    cmd: ReplaceMealCommand,
    executor: &E,
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

    // Validate: Check that the assignment exists for given date/meal_type
    let assignment_exists = aggregate
        .meal_assignments
        .iter()
        .any(|a| a.date == cmd.date && a.meal_type == cmd.meal_type);

    if !assignment_exists {
        return Err(MealPlanningError::MealAssignmentNotFound(
            cmd.date.clone(),
            cmd.meal_type.clone(),
        ));
    }

    // Get the old recipe_id for the event
    let old_recipe_id = aggregate
        .meal_assignments
        .iter()
        .find(|a| a.date == cmd.date && a.meal_type == cmd.meal_type)
        .map(|a| a.recipe_id.clone())
        .ok_or_else(|| {
            MealPlanningError::MealAssignmentNotFound(cmd.date.clone(), cmd.meal_type.clone())
        })?;

    // Validate: Check that new recipe is not already the current recipe
    if old_recipe_id == cmd.new_recipe_id {
        return Err(MealPlanningError::RecipeAlreadyAssigned(
            cmd.new_recipe_id.clone(),
        ));
    }

    // Validate rotation constraint: new recipe must not be already used in current cycle
    let rotation_state = RotationState::from_json(&aggregate.rotation_state_json).map_err(|e| {
        MealPlanningError::RotationStateError(format!("Failed to parse rotation state: {}", e))
    })?;

    if rotation_state.is_recipe_used(&cmd.new_recipe_id) {
        return Err(MealPlanningError::RecipeAlreadyUsedInRotation(
            cmd.new_recipe_id.clone(),
            rotation_state.cycle_number,
        ));
    }

    // Emit MealReplaced event
    let replaced_at = Utc::now().to_rfc3339();
    let event_data = MealReplaced {
        date: cmd.date,
        meal_type: cmd.meal_type,
        old_recipe_id,
        new_recipe_id: cmd.new_recipe_id,
        replaced_at,
    };

    evento::save::<MealPlanAggregate>(&cmd.meal_plan_id)
        .data(&event_data)
        .map_err(|e| {
            MealPlanningError::EventoError(format!("Failed to encode MealReplaced event: {}", e))
        })?
        .metadata(&true)
        .map_err(|e| MealPlanningError::EventoError(format!("Failed to encode metadata: {}", e)))?
        .commit(executor)
        .await
        .map_err(|e| {
            MealPlanningError::EventoError(format!("Failed to commit MealReplaced event: {}", e))
        })?;

    Ok(())
}
