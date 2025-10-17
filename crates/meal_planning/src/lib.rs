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
pub use commands::{GenerateMealPlanCommand, RegenerateMealPlanCommand, ReplaceMealCommand};
pub use constraints::{
    AdvancePrepConstraint, AvailabilityConstraint, ComplexityConstraint, Constraint,
    DietaryConstraint, EquipmentConflictConstraint, FreshnessConstraint, MealSlot, MealType,
};
pub use error::MealPlanningError;
pub use events::{MealPlanGenerated, MealPlanRegenerated, MealReplaced, RecipeUsedInRotation};
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
    favorite_recipes: Vec<algorithm::RecipeForPlanning>,
    user_constraints: algorithm::UserConstraints,
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

    // Generate new meal plan using algorithm (same constraints as initial generation)
    // Use different seed to ensure variety (timestamp-based)
    let (new_assignments, updated_rotation_state) = MealPlanningAlgorithm::generate(
        &aggregate.start_date,
        favorite_recipes,
        user_constraints,
        rotation_state,
        None, // No seed = timestamp-based randomization for variety
    )?;

    // Emit MealPlanRegenerated event
    let regenerated_at = Utc::now().to_rfc3339();
    let event_data = events::MealPlanRegenerated {
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

#[cfg(test)]
mod tests {
    use super::*;
    use algorithm::{RecipeForPlanning, UserConstraints};
    use commands::RegenerateMealPlanCommand;
    use events::MealPlanGenerated;
    use evento::prelude::{Migrate, Plan};
    use rotation::RotationState;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_executor() -> (evento::Sqlite, sqlx::SqlitePool) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(":memory:")
            .await
            .unwrap();

        // Run evento migrations for event store
        let mut conn = pool.acquire().await.unwrap();
        evento::sql_migrator::new_migrator::<sqlx::Sqlite>()
            .unwrap()
            .run(&mut conn, &Plan::apply_all())
            .await
            .unwrap();
        drop(conn);

        let executor: evento::Sqlite = pool.clone().into();
        (executor, pool)
    }

    fn create_test_recipe(id: &str) -> RecipeForPlanning {
        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Test Recipe {}", id),
            ingredients_count: 5,
            instructions_count: 4,
            prep_time_min: Some(15),
            cook_time_min: Some(30),
            advance_prep_hours: None,
            complexity: Some("simple".to_string()),
        }
    }

    /// Test: Regenerate meal plan succeeds with valid input (Story 3.7 AC-3, AC-4)
    #[tokio::test]
    async fn test_regenerate_meal_plan_success() {
        // Setup in-memory executor
        let (executor, _pool) = setup_test_executor().await;

        // Create initial meal plan
        let user_id = "test_user_1";
        let start_date = "2025-10-20"; // Monday
        let mut rotation_state = RotationState::new();
        rotation_state.total_favorite_count = 10;

        // Create 10 test recipes
        let mut favorites = Vec::new();
        for i in 1..=10 {
            favorites.push(create_test_recipe(&format!("recipe_{}", i)));
        }

        // Generate initial meal plan using algorithm
        let constraints = UserConstraints::default();
        let (initial_assignments, initial_rotation_state) =
            MealPlanningAlgorithm::generate(start_date, favorites.clone(), constraints.clone(), rotation_state, Some(42))
                .expect("Initial generation failed");

        // Emit MealPlanGenerated event
        let generated_at = chrono::Utc::now().to_rfc3339();
        let event_data = MealPlanGenerated {
            user_id: user_id.to_string(),
            start_date: start_date.to_string(),
            meal_assignments: initial_assignments.clone(),
            rotation_state_json: initial_rotation_state.to_json().unwrap(),
            generated_at: generated_at.clone(),
        };

        let meal_plan_id = evento::create::<MealPlanAggregate>()
            .data(&event_data)
            .unwrap()
            .metadata(&true)
            .unwrap()
            .commit(&executor)
            .await
            .unwrap();

        // Store old cycle number
        let old_cycle_number = initial_rotation_state.cycle_number;

        // Now regenerate the meal plan
        let regenerate_cmd = RegenerateMealPlanCommand {
            meal_plan_id: meal_plan_id.clone(),
            user_id: user_id.to_string(),
            regeneration_reason: Some("Testing regeneration".to_string()),
        };

        let result = regenerate_meal_plan(regenerate_cmd, &executor, favorites.clone(), constraints.clone()).await;
        assert!(result.is_ok(), "Regeneration should succeed");

        // Load aggregate and verify state
        let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
            .await
            .unwrap();
        let aggregate = loaded.item;

        // AC-6: Verify all 21 slots filled
        assert_eq!(aggregate.meal_assignments.len(), 21, "Should have 21 meal assignments");

        // AC-5: Verify rotation state preserved (cycle number unchanged or incremented if reset)
        let new_rotation_state = RotationState::from_json(&aggregate.rotation_state_json).unwrap();
        assert!(
            new_rotation_state.cycle_number >= old_cycle_number,
            "Cycle number should be preserved or incremented"
        );

        // Verify meal plan is still active
        assert_eq!(aggregate.status, "active", "Meal plan should remain active");
    }

    /// Test: Regenerate fails when meal plan not found (Story 3.7 validation)
    #[tokio::test]
    async fn test_regenerate_meal_plan_not_found() {
        let (executor, _pool) = setup_test_executor().await;

        let favorites = vec![
            create_test_recipe("1"),
            create_test_recipe("2"),
            create_test_recipe("3"),
            create_test_recipe("4"),
            create_test_recipe("5"),
            create_test_recipe("6"),
            create_test_recipe("7"),
        ];

        let regenerate_cmd = RegenerateMealPlanCommand {
            meal_plan_id: "non_existent_plan".to_string(),
            user_id: "test_user".to_string(),
            regeneration_reason: None,
        };

        let result = regenerate_meal_plan(regenerate_cmd, &executor, favorites, UserConstraints::default()).await;
        assert!(result.is_err(), "Should fail when meal plan not found");

        // evento::load returns EventoError("not found") instead of custom error
        match result {
            Err(MealPlanningError::EventoError(msg)) if msg.contains("not found") => {
                // Expected error from evento
            }
            Err(MealPlanningError::MealPlanNotFound(_)) => {
                // Also valid - happens after aggregate load
            }
            Err(e) => panic!("Expected not found error, got: {:?}", e),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }

    /// Test: Regenerate fails with insufficient recipes (Story 3.7 AC-10)
    #[tokio::test]
    async fn test_regenerate_insufficient_recipes() {
        let (executor, _pool) = setup_test_executor().await;

        // Create meal plan first
        let user_id = "test_user_2";
        let start_date = "2025-10-20";
        let rotation_state = RotationState::new();

        let favorites = vec![
            create_test_recipe("1"),
            create_test_recipe("2"),
            create_test_recipe("3"),
            create_test_recipe("4"),
            create_test_recipe("5"),
            create_test_recipe("6"),
            create_test_recipe("7"),
        ];

        let constraints = UserConstraints::default();
        let (assignments, rotation_state) =
            MealPlanningAlgorithm::generate(start_date, favorites.clone(), constraints.clone(), rotation_state, Some(42))
                .unwrap();

        let event_data = MealPlanGenerated {
            user_id: user_id.to_string(),
            start_date: start_date.to_string(),
            meal_assignments: assignments,
            rotation_state_json: rotation_state.to_json().unwrap(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        let meal_plan_id = evento::create::<MealPlanAggregate>()
            .data(&event_data)
            .unwrap()
            .metadata(&true)
            .unwrap()
            .commit(&executor)
            .await
            .unwrap();

        // Try to regenerate with only 3 recipes (insufficient)
        let insufficient_favorites = vec![
            create_test_recipe("1"),
            create_test_recipe("2"),
            create_test_recipe("3"),
        ];

        let regenerate_cmd = RegenerateMealPlanCommand {
            meal_plan_id,
            user_id: user_id.to_string(),
            regeneration_reason: None,
        };

        let result = regenerate_meal_plan(regenerate_cmd, &executor, insufficient_favorites, constraints).await;
        assert!(result.is_err(), "Should fail with insufficient recipes");

        match result {
            Err(MealPlanningError::InsufficientRecipes { minimum, current }) => {
                assert_eq!(minimum, 7);
                assert_eq!(current, 3);
            }
            _ => panic!("Expected InsufficientRecipes error"),
        }
    }

    /// Test: Regenerate fails with unauthorized access (Story 3.7 security)
    #[tokio::test]
    async fn test_regenerate_unauthorized_access() {
        let (executor, _pool) = setup_test_executor().await;

        // Create meal plan for user1
        let user_id_1 = "test_user_1";
        let start_date = "2025-10-20";
        let rotation_state = RotationState::new();

        let favorites = (1..=10).map(|i| create_test_recipe(&format!("{}", i))).collect::<Vec<_>>();

        let constraints = UserConstraints::default();
        let (assignments, rotation_state) =
            MealPlanningAlgorithm::generate(start_date, favorites.clone(), constraints.clone(), rotation_state, Some(42))
                .unwrap();

        let event_data = MealPlanGenerated {
            user_id: user_id_1.to_string(),
            start_date: start_date.to_string(),
            meal_assignments: assignments,
            rotation_state_json: rotation_state.to_json().unwrap(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };

        let meal_plan_id = evento::create::<MealPlanAggregate>()
            .data(&event_data)
            .unwrap()
            .metadata(&true)
            .unwrap()
            .commit(&executor)
            .await
            .unwrap();

        // Try to regenerate as user2 (unauthorized)
        let regenerate_cmd = RegenerateMealPlanCommand {
            meal_plan_id,
            user_id: "test_user_2".to_string(), // Different user!
            regeneration_reason: None,
        };

        let result = regenerate_meal_plan(regenerate_cmd, &executor, favorites, constraints).await;
        assert!(result.is_err(), "Should fail with unauthorized access");

        match result {
            Err(MealPlanningError::UnauthorizedAccess(_, _)) => {
                // Expected error
            }
            _ => panic!("Expected UnauthorizedAccess error"),
        }
    }
}
