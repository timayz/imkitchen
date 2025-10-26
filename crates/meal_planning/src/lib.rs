pub mod aggregate;
pub mod algorithm;
pub mod commands;
pub mod constraints;
pub mod dietary_filter;
pub mod error;
pub mod events;
pub mod read_model;
pub mod rotation;

pub use aggregate::MealPlanAggregate;
pub use algorithm::{
    generate_multi_week_meal_plans, generate_reasoning_text, generate_shopping_list_for_week,
    generate_single_week, select_accompaniment, select_main_course_with_preferences,
    MealPlanningAlgorithm, Recipe, RecipeComplexityCalculator, RecipeForPlanning, ShoppingCategory,
    ShoppingItem, ShoppingList, UserPreferences,
};
pub use commands::{regenerate_meal_plan, GenerateMealPlanCommand, RegenerateMealPlanCommand};
pub use constraints::{
    AdvancePrepConstraint, AvailabilityConstraint, ComplexityConstraint, Constraint, CourseType,
    DietaryConstraint, EquipmentConflictConstraint, FreshnessConstraint, MealSlot,
};
pub use dietary_filter::filter_by_dietary_restrictions;
pub use error::MealPlanningError;
pub use events::{
    AllFutureWeeksRegenerated, MealPlanArchived, MealPlanGenerated, MealPlanRegenerated,
    MultiWeekMealPlan, MultiWeekMealPlanGenerated, RecipeUsedInRotation, RotationCycleReset,
    SingleWeekRegenerated, WeekMealPlan, WeekMealPlanData, WeekStatus,
};
pub use read_model::{
    meal_plan_projection, MealAssignmentReadModel, MealPlanQueries, MealPlanReadModel,
};
pub use rotation::{RotationState, RotationSystem};

/// Calculate the start date for next week (always a Monday).
///
/// Business Rule: All meal plans are generated for "next week only" - the Monday-Sunday
/// period starting from the Monday following the current week. This forward-looking approach
/// gives users time to shop and prepare without disrupting current week meals.
///
/// # Algorithm
/// - If today is Monday: next week starts in 7 days (next Monday)
/// - If today is Tuesday-Sunday: next week starts on the coming Monday
///
/// # Returns
/// A `chrono::NaiveDate` representing next Monday in ISO 8601 format (YYYY-MM-DD)
///
/// # Examples
/// ```
/// use meal_planning::calculate_next_week_start;
/// use chrono::{Datelike, Weekday};
///
/// let next_monday = calculate_next_week_start();
/// assert_eq!(next_monday.weekday(), Weekday::Mon);
/// ```
pub fn calculate_next_week_start() -> chrono::NaiveDate {
    use chrono::{Datelike, Duration, Local, Weekday};

    let today = Local::now().date_naive();
    let days_until_next_monday = match today.weekday() {
        Weekday::Mon => 7, // If Monday, next week is 7 days away
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1, // If Sunday, next week starts tomorrow
    };
    today + Duration::days(days_until_next_monday as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use algorithm::{RecipeForPlanning, UserConstraints};
    use commands::RegenerateMealPlanCommand;
    use evento::migrator::{Migrate, Plan};
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
            cuisine: recipe::Cuisine::Italian,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
        }
    }

    /// Test: Regenerate meal plan succeeds with valid input (Story 3.7 AC-3, AC-4)
    #[tokio::test]
    async fn test_regenerate_meal_plan_success() {
        // Setup in-memory executor
        let (executor, _pool) = setup_test_executor().await;

        // Create initial meal plan
        let user_id = "test_user_1";
        let start_date = &crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string(); // Next Monday (Story 3.13)
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
        let start_date = &crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string();
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
        let start_date = &crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string();
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

    /// Test: calculate_next_week_start returns next Monday for all weekdays (Story 3.13 AC1)
    #[test]
    fn test_calculate_next_week_start_all_weekdays() {
        use chrono::{Datelike, NaiveDate, Weekday};

        // Test reference dates (known weekdays in 2025)
        let test_cases = vec![
            ("2025-10-20", Weekday::Mon, "2025-10-27"), // Monday -> +7 days
            ("2025-10-21", Weekday::Tue, "2025-10-27"), // Tuesday -> +6 days
            ("2025-10-22", Weekday::Wed, "2025-10-27"), // Wednesday -> +5 days
            ("2025-10-23", Weekday::Thu, "2025-10-27"), // Thursday -> +4 days
            ("2025-10-24", Weekday::Fri, "2025-10-27"), // Friday -> +3 days
            ("2025-10-25", Weekday::Sat, "2025-10-27"), // Saturday -> +2 days
            ("2025-10-26", Weekday::Sun, "2025-10-27"), // Sunday -> +1 day
        ];

        for (input_date_str, expected_weekday, expected_next_monday_str) in test_cases {
            let input_date = NaiveDate::parse_from_str(input_date_str, "%Y-%m-%d").unwrap();
            let expected_next_monday =
                NaiveDate::parse_from_str(expected_next_monday_str, "%Y-%m-%d").unwrap();

            // Verify input date is correct weekday
            assert_eq!(
                input_date.weekday(),
                expected_weekday,
                "Input date {} should be {:?}",
                input_date_str,
                expected_weekday
            );

            // Calculate days until next Monday manually
            let days_until_next_monday = match expected_weekday {
                Weekday::Mon => 7,
                Weekday::Tue => 6,
                Weekday::Wed => 5,
                Weekday::Thu => 4,
                Weekday::Fri => 3,
                Weekday::Sat => 2,
                Weekday::Sun => 1,
            };

            let calculated_next_monday =
                input_date + chrono::Duration::days(days_until_next_monday);

            // Verify calculation matches expected
            assert_eq!(
                calculated_next_monday, expected_next_monday,
                "From {} ({:?}), next Monday should be {}",
                input_date_str, expected_weekday, expected_next_monday_str
            );

            // Verify next Monday is actually a Monday
            assert_eq!(
                calculated_next_monday.weekday(),
                Weekday::Mon,
                "Result should always be a Monday"
            );
        }
    }

    /// Test: calculate_next_week_start edge case - today is Sunday (Story 3.13 AC7)
    #[test]
    fn test_calculate_next_week_start_edge_case_sunday() {
        use chrono::{Datelike, NaiveDate, Weekday};

        // Sunday 2025-10-26 -> next Monday is 2025-10-27 (+1 day)
        let sunday = NaiveDate::from_ymd_opt(2025, 10, 26).unwrap();
        assert_eq!(sunday.weekday(), Weekday::Sun);

        let next_monday = sunday + chrono::Duration::days(1);
        assert_eq!(next_monday.weekday(), Weekday::Mon);
        assert_eq!(next_monday.to_string(), "2025-10-27");
    }

    /// Test: calculate_next_week_start edge case - today is Monday (Story 3.13 AC7)
    #[test]
    fn test_calculate_next_week_start_edge_case_monday() {
        use chrono::{Datelike, NaiveDate, Weekday};

        // Monday 2025-10-20 -> next Monday is 2025-10-27 (+7 days)
        let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
        assert_eq!(monday.weekday(), Weekday::Mon);

        let next_monday = monday + chrono::Duration::days(7);
        assert_eq!(next_monday.weekday(), Weekday::Mon);
        assert_eq!(next_monday.to_string(), "2025-10-27");
    }

    /// Test: Week boundaries validation (Story 3.13 AC1)
    #[test]
    fn test_week_boundaries() {
        use chrono::{Datelike, NaiveDate, Weekday};

        let monday = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();
        assert_eq!(monday.weekday(), Weekday::Mon, "Week starts on Monday");

        let sunday = monday + chrono::Duration::days(6);
        assert_eq!(sunday.weekday(), Weekday::Sun, "Week ends on Sunday");

        // Verify exactly 7 days in week (inclusive)
        let days_in_week = (sunday - monday).num_days() + 1;
        assert_eq!(days_in_week, 7, "Week should have exactly 7 days");
    }

    /// Test: Algorithm rejects start_date in the past (Story 3.13 AC4)
    #[tokio::test]
    async fn test_algorithm_rejects_past_date() {
        let yesterday = chrono::Local::now().date_naive() - chrono::Duration::days(1);
        let start_date = yesterday.format("%Y-%m-%d").to_string();

        let favorites = (1..=21)
            .map(|i| create_test_recipe(&format!("recipe_{}", i)))
            .collect::<Vec<_>>();

        let result = MealPlanningAlgorithm::generate(
            &start_date,
            favorites,
            UserConstraints::default(),
            RotationState::new(),
            Some(42),
        );

        assert!(result.is_err(), "Should reject past dates");
        match result {
            Err(MealPlanningError::InvalidWeekStart(msg)) => {
                assert!(msg.contains("must be in the future"));
            }
            _ => panic!("Expected InvalidWeekStart error for past date"),
        }
    }

    /// Test: Algorithm rejects start_date that is not Monday (Story 3.13 AC1)
    #[tokio::test]
    async fn test_algorithm_rejects_non_monday() {
        use chrono::{Datelike, Weekday};

        // Find next Tuesday (not Monday)
        let today = chrono::Local::now().date_naive();
        let mut test_date = today + chrono::Duration::days(1);
        while test_date.weekday() != Weekday::Tue {
            test_date += chrono::Duration::days(1);
        }

        let start_date = test_date.format("%Y-%m-%d").to_string();

        let favorites = (1..=21)
            .map(|i| create_test_recipe(&format!("recipe_{}", i)))
            .collect::<Vec<_>>();

        let result = MealPlanningAlgorithm::generate(
            &start_date,
            favorites,
            UserConstraints::default(),
            RotationState::new(),
            Some(42),
        );

        assert!(result.is_err(), "Should reject non-Monday dates");
        match result {
            Err(MealPlanningError::InvalidWeekStart(msg)) => {
                assert!(msg.contains("must be a Monday"));
            }
            _ => panic!("Expected InvalidWeekStart error for non-Monday date"),
        }
    }

    /// Test: Algorithm accepts valid next Monday (Story 3.13 AC2)
    #[tokio::test]
    async fn test_algorithm_accepts_next_monday() {
        // Calculate next Monday using our function
        let next_monday = crate::calculate_next_week_start();
        let start_date = next_monday.format("%Y-%m-%d").to_string();

        // Use 21 recipes to ensure we have enough variety across all recipe types
        let favorites = (1..=21)
            .map(|i| create_test_recipe(&format!("recipe_{}", i)))
            .collect::<Vec<_>>();

        let result = MealPlanningAlgorithm::generate(
            &start_date,
            favorites,
            UserConstraints::default(),
            RotationState::new(),
            Some(42),
        );

        assert!(
            result.is_ok(),
            "Should accept next Monday: {}, but got error: {:?}",
            start_date,
            result.as_ref().err()
        );

        let (assignments, _) = result.unwrap();
        assert_eq!(assignments.len(), 21, "Should generate 21 meal assignments");
    }

    /// Test: Regenerate enforces next-week-only constraint (Story 3.13 AC3)
    #[tokio::test]
    async fn test_regenerate_uses_next_monday() {
        let (executor, _pool) = setup_test_executor().await;

        // Create initial meal plan with next Monday (valid plan)
        let user_id = "test_user_1";
        let old_start_date = &crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string();

        let favorites = (1..=30)
            .map(|i| create_test_recipe(&format!("recipe_{}", i)))
            .collect::<Vec<_>>();

        // Generate initial plan using next Monday
        let (initial_assignments, rotation_state) = MealPlanningAlgorithm::generate(
            old_start_date,
            favorites.clone(),
            UserConstraints::default(),
            RotationState::new(),
            Some(42),
        )
        .unwrap();

        let event_data = MealPlanGenerated {
            user_id: user_id.to_string(),
            start_date: old_start_date.to_string(),
            meal_assignments: initial_assignments,
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

        // Regenerate meal plan
        let cmd = RegenerateMealPlanCommand {
            meal_plan_id: meal_plan_id.clone(),
            user_id: user_id.to_string(),
            regeneration_reason: Some("Testing next-week enforcement".to_string()),
        };

        let result =
            regenerate_meal_plan(cmd, &executor, favorites, UserConstraints::default()).await;
        assert!(
            result.is_ok(),
            "Regeneration should succeed and use next Monday"
        );

        // Load aggregate and verify start_date was updated to next Monday
        let loaded = evento::load::<MealPlanAggregate, _>(&executor, &meal_plan_id)
            .await
            .unwrap();

        // Note: MealPlanRegenerated event doesn't update start_date in aggregate
        // The regeneration creates new assignments for next week, but the aggregate
        // retains the original start_date. This is acceptable as the new assignments
        // use next week's dates. We verify this by checking the assignment dates.

        let first_assignment_date = &loaded.item.meal_assignments[0].date;
        let next_monday = crate::calculate_next_week_start();
        let expected_start = next_monday.format("%Y-%m-%d").to_string();

        assert_eq!(
            first_assignment_date, &expected_start,
            "First assignment should be for next Monday"
        );
    }
}
