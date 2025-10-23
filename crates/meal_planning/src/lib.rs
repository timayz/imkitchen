pub mod aggregate;
pub mod algorithm;
pub mod commands;
pub mod constraints;
pub mod error;
pub mod events;
pub mod read_model;
pub mod rotation;

pub use aggregate::MealPlanAggregate;
pub use algorithm::{generate_reasoning_text, MealPlanningAlgorithm, RecipeComplexityCalculator};
pub use commands::{
    regenerate_meal_plan, replace_meal, GenerateMealPlanCommand, RegenerateMealPlanCommand,
    ReplaceMealCommand,
};
pub use constraints::{
    AdvancePrepConstraint, AvailabilityConstraint, ComplexityConstraint, Constraint, CourseType,
    DietaryConstraint, EquipmentConflictConstraint, FreshnessConstraint, MealSlot,
};
pub use error::MealPlanningError;
pub use events::{
    MealPlanArchived, MealPlanGenerated, MealPlanRegenerated, MealReplaced, RecipeUsedInRotation,
    RotationCycleReset,
};
pub use read_model::{
    meal_plan_projection, MealAssignmentReadModel, MealPlanQueries, MealPlanReadModel,
};
pub use rotation::{RotationState, RotationSystem};

#[cfg(test)]
mod tests {
    use super::*;
    use algorithm::{RecipeForPlanning, UserConstraints};
    use commands::RegenerateMealPlanCommand;
    use evento::prelude::{Migrate, Plan};
    use events::MealPlanGenerated;
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
        // Extract numeric part from id (e.g., "recipe_1" -> 1)
        let num = id
            .split('_')
            .next_back()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        // Distribute types evenly to ensure variety for regeneration:
        // Use modulo 3 to distribute evenly across all types
        let recipe_type = match num % 3 {
            0 => "dessert",     // Every 3rd recipe
            1 => "appetizer",   // Recipes 1, 4, 7, 10, 13...
            _ => "main_course", // Recipes 2, 3, 5, 6, 8, 9...
        };

        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Test Recipe {}", id),
            recipe_type: recipe_type.to_string(),
            ingredients_count: 5,
            instructions_count: 4,
            prep_time_min: Some(15),
            cook_time_min: Some(30),
            advance_prep_hours: None,
            complexity: Some("simple".to_string()),
            dietary_tags: Vec::new(),
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
        rotation_state.total_favorite_count = 30;

        // Create 30 test recipes (enough for variety and rotation)
        let mut favorites = Vec::new();
        for i in 1..=30 {
            favorites.push(create_test_recipe(&format!("recipe_{}", i)));
        }

        // Generate initial meal plan using algorithm
        let constraints = UserConstraints::default();
        let (initial_assignments, initial_rotation_state) = MealPlanningAlgorithm::generate(
            start_date,
            favorites.clone(),
            constraints.clone(),
            rotation_state,
            Some(42),
        )
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

        let result = regenerate_meal_plan(
            regenerate_cmd,
            &executor,
            favorites.clone(),
            constraints.clone(),
        )
        .await;
        assert!(result.is_ok(), "Regeneration should succeed");

        // Load aggregate and verify state
        let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
            .await
            .unwrap();
        let aggregate = loaded.item;

        // AC-6: Verify all 21 slots filled
        assert_eq!(
            aggregate.meal_assignments.len(),
            21,
            "Should have 21 meal assignments"
        );

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

        let result = regenerate_meal_plan(
            regenerate_cmd,
            &executor,
            favorites,
            UserConstraints::default(),
        )
        .await;
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

        // Create 15 initial recipes for successful generation
        let mut favorites = Vec::new();
        for i in 1..=15 {
            favorites.push(create_test_recipe(&format!("recipe_{}", i)));
        }

        let constraints = UserConstraints::default();
        let (assignments, rotation_state) = MealPlanningAlgorithm::generate(
            start_date,
            favorites.clone(),
            constraints.clone(),
            rotation_state,
            Some(42),
        )
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

        let result = regenerate_meal_plan(
            regenerate_cmd,
            &executor,
            insufficient_favorites,
            constraints,
        )
        .await;
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

        let favorites = (1..=20)
            .map(|i| create_test_recipe(&format!("{}", i)))
            .collect::<Vec<_>>();

        let constraints = UserConstraints::default();
        let (assignments, rotation_state) = MealPlanningAlgorithm::generate(
            start_date,
            favorites.clone(),
            constraints.clone(),
            rotation_state,
            Some(42),
        )
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
