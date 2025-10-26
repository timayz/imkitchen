//! Story 7.4: Single Week Generation - Comprehensive Unit Tests
//!
//! Tests the generate_single_week function which generates a complete week's meal plan
//! with 21 assignments (7 days × 3 courses) respecting all constraints and rotation rules.

use chrono::{Datelike, NaiveDate, Weekday};
use meal_planning::algorithm::{generate_single_week, RecipeForPlanning, UserPreferences};
use meal_planning::rotation::RotationState;
use meal_planning::WeekMealPlan;
use recipe::{AccompanimentCategory, Cuisine};

/// Helper to create test recipe with specific properties
fn create_test_recipe(
    id: &str,
    recipe_type: &str,
    ingredients: usize,
    steps: usize,
    prep_time: u32,
    cook_time: u32,
    cuisine: Cuisine,
) -> RecipeForPlanning {
    RecipeForPlanning {
        id: id.to_string(),
        title: format!("Test Recipe {}", id),
        recipe_type: recipe_type.to_string(),
        ingredients_count: ingredients,
        instructions_count: steps,
        prep_time_min: Some(prep_time),
        cook_time_min: Some(cook_time),
        advance_prep_hours: None,
        complexity: None, // Will be calculated
        dietary_tags: vec![],
        cuisine,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
    }
}

/// Helper to create quick main course (fits weeknight 30min constraint)
fn create_quick_main_course(id: &str, cuisine: Cuisine) -> RecipeForPlanning {
    create_test_recipe(id, "main_course", 8, 6, 10, 15, cuisine) // 25 min total
}

/// Helper to create balanced recipe set (appetizers, mains, desserts)
fn create_balanced_recipes(count_per_type: usize) -> Vec<RecipeForPlanning> {
    let mut recipes = Vec::new();

    // Appetizers
    for i in 0..count_per_type {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    // Main courses (mix of weeknight and weekend recipes)
    for i in 0..count_per_type {
        // Alternate between quick (weeknight-friendly) and longer (weekend) recipes
        let (prep, cook) = if i % 2 == 0 {
            (10, 15) // 25 minutes total - fits weeknight 30min limit
        } else {
            (20, 30) // 50 minutes total - fits weekend 90min limit
        };

        recipes.push(create_test_recipe(
            &format!("main_{}", i),
            "main_course",
            8,
            6,
            prep,
            cook,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    // Desserts
    for i in 0..count_per_type {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    recipes
}

/// AC-1: Test that generate_single_week function is implemented with correct signature
#[test]
fn test_function_exists_and_has_correct_signature() {
    // This test verifies the function compiles with the expected signature
    let recipes = create_balanced_recipes(7);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday

    let result: Result<WeekMealPlan, _> =
        generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);

    // Function should exist and return the correct type
    assert!(result.is_ok() || result.is_err()); // Either outcome is valid for existence test
}

/// AC-2: Test that generate_single_week generates exactly 21 assignments (7 days × 3 courses)
#[test]
fn test_generates_21_assignments() {
    let recipes = create_balanced_recipes(10); // 10 of each type (30 total)
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);

    assert!(
        result.is_ok(),
        "Should successfully generate week, but got error: {:?}",
        result.as_ref().err()
    );
    let week_plan = result.unwrap();

    // AC-2: Must have exactly 21 assignments
    assert_eq!(
        week_plan.meal_assignments.len(),
        21,
        "Should have 21 meal assignments (7 days × 3 courses)"
    );
}

/// AC-3: Test that each day has exactly 3 assignments: appetizer, main_course, dessert
#[test]
fn test_each_day_has_three_courses() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // Group assignments by date
    use std::collections::HashMap;
    let mut assignments_by_date: HashMap<String, Vec<String>> = HashMap::new();

    for assignment in &week_plan.meal_assignments {
        assignments_by_date
            .entry(assignment.date.clone())
            .or_default()
            .push(assignment.course_type.clone());
    }

    // AC-3: Each day should have exactly 3 courses
    assert_eq!(assignments_by_date.len(), 7, "Should have 7 days");

    for (date, courses) in &assignments_by_date {
        assert_eq!(
            courses.len(),
            3,
            "Date {} should have exactly 3 courses",
            date
        );

        // Verify all three course types present
        assert!(
            courses.contains(&"appetizer".to_string()),
            "Date {} missing appetizer",
            date
        );
        assert!(
            courses.contains(&"main_course".to_string()),
            "Date {} missing main_course",
            date
        );
        assert!(
            courses.contains(&"dessert".to_string()),
            "Date {} missing dessert",
            date
        );
    }
}

/// AC-3: Test that dates span Monday-Sunday (7 consecutive days)
#[test]
fn test_dates_span_monday_to_sunday() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // Extract unique dates
    use std::collections::HashSet;
    let unique_dates: HashSet<&String> =
        week_plan.meal_assignments.iter().map(|a| &a.date).collect();

    assert_eq!(unique_dates.len(), 7, "Should have exactly 7 unique dates");

    // Verify dates are consecutive Monday-Sunday
    let expected_dates: Vec<String> = (0..7)
        .map(|offset| {
            (week_start_date + chrono::Duration::days(offset))
                .format("%Y-%m-%d")
                .to_string()
        })
        .collect();

    for expected_date in &expected_dates {
        assert!(
            unique_dates.contains(expected_date),
            "Missing date: {}",
            expected_date
        );
    }

    // Verify week starts Monday and ends Sunday
    let first_date = NaiveDate::parse_from_str(&expected_dates[0], "%Y-%m-%d").unwrap();
    let last_date = NaiveDate::parse_from_str(&expected_dates[6], "%Y-%m-%d").unwrap();

    assert_eq!(
        first_date.weekday(),
        Weekday::Mon,
        "Week should start on Monday"
    );
    assert_eq!(
        last_date.weekday(),
        Weekday::Sun,
        "Week should end on Sunday"
    );
}

/// AC-4: Test appetizer rotation with reset logic
#[test]
fn test_appetizer_rotation_with_reset() {
    // Create only 3 appetizers but 10 mains and desserts (to force appetizer reuse)
    let mut recipes = Vec::new();

    // 3 appetizers (will be exhausted and reset)
    for i in 0..3 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            Cuisine::Italian,
        ));
    }

    // 10 main courses (quick ones that fit weeknight 30min constraint)
    for i in 0..10 {
        recipes.push(create_quick_main_course(
            &format!("main_{}", i),
            Cuisine::Italian,
        ));
    }

    // 10 desserts
    for i in 0..10 {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            Cuisine::Italian,
        ));
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(
        result.is_ok(),
        "Should successfully generate week despite limited appetizers"
    );
    let week_plan = result.unwrap();

    // Collect appetizer assignments
    let appetizer_ids: Vec<&String> = week_plan
        .meal_assignments
        .iter()
        .filter(|a| a.course_type == "appetizer")
        .map(|a| &a.recipe_id)
        .collect();

    // AC-4: Should have 7 appetizer assignments (one per day)
    assert_eq!(appetizer_ids.len(), 7, "Should have 7 appetizers");

    // AC-4: With only 3 unique appetizers, some must repeat (proving reset logic works)
    use std::collections::HashSet;
    let unique_appetizers: HashSet<&String> = appetizer_ids.iter().cloned().collect();
    assert!(
        unique_appetizers.len() <= 3,
        "Should only use 3 unique appetizers (cycling through available set)"
    );

    // Verify at least one appetizer repeated (since 7 slots but only 3 appetizers)
    assert!(
        appetizer_ids.len() > unique_appetizers.len(),
        "Appetizers should repeat after exhausting available set"
    );
}

/// AC-4: Test dessert rotation with reset logic
#[test]
fn test_dessert_rotation_with_reset() {
    // Create only 4 desserts but 10 mains and appetizers
    let mut recipes = Vec::new();

    // 10 appetizers
    for i in 0..10 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            Cuisine::Italian,
        ));
    }

    // 10 main courses (quick ones that fit weeknight 30min constraint)
    for i in 0..10 {
        recipes.push(create_quick_main_course(
            &format!("main_{}", i),
            Cuisine::Italian,
        ));
    }

    // 4 desserts (will be exhausted and reset)
    for i in 0..4 {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            Cuisine::Italian,
        ));
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // Collect dessert assignments
    let dessert_ids: Vec<&String> = week_plan
        .meal_assignments
        .iter()
        .filter(|a| a.course_type == "dessert")
        .map(|a| &a.recipe_id)
        .collect();

    // AC-4: Should have 7 dessert assignments
    assert_eq!(dessert_ids.len(), 7, "Should have 7 desserts");

    // AC-4: With only 4 unique desserts, some must repeat
    use std::collections::HashSet;
    let unique_desserts: HashSet<&String> = dessert_ids.iter().cloned().collect();
    assert!(
        unique_desserts.len() <= 4,
        "Should only use 4 unique desserts"
    );

    assert!(
        dessert_ids.len() > unique_desserts.len(),
        "Desserts should repeat after exhausting available set"
    );
}

/// AC-5: Test that main courses are unique (never repeat within week)
#[test]
fn test_main_courses_never_repeat_within_week() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // Collect main course IDs
    let main_course_ids: Vec<&String> = week_plan
        .meal_assignments
        .iter()
        .filter(|a| a.course_type == "main_course")
        .map(|a| &a.recipe_id)
        .collect();

    // AC-5: Should have 7 main course assignments
    assert_eq!(main_course_ids.len(), 7, "Should have 7 main courses");

    // AC-5: All main courses must be unique (no repeats)
    use std::collections::HashSet;
    let unique_main_courses: HashSet<&String> = main_course_ids.iter().cloned().collect();
    assert_eq!(
        unique_main_courses.len(),
        7,
        "All 7 main courses should be unique (no repeats)"
    );
}

/// AC-6: Test accompaniment assigned when main course accepts_accompaniment=true
#[test]
fn test_accompaniment_assigned_when_accepted() {
    // Create recipes with some main courses accepting accompaniments
    let mut recipes = Vec::new();

    // Appetizers and desserts
    for i in 0..10 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            Cuisine::Italian,
        ));
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            Cuisine::Italian,
        ));
    }

    // Main courses that accept accompaniments - quick ones for weeknights
    for i in 0..7 {
        let mut main = create_quick_main_course(&format!("main_{}", i), Cuisine::Italian);
        main.accepts_accompaniment = true;
        main.preferred_accompaniments = vec![AccompanimentCategory::Rice];
        recipes.push(main);
    }

    // Add accompaniment recipes
    for i in 0..5 {
        let mut accompaniment = create_test_recipe(
            &format!("rice_{}", i),
            "main_course",
            3,
            2,
            5,
            15,
            Cuisine::Italian,
        );
        accompaniment.accompaniment_category = Some(AccompanimentCategory::Rice);
        recipes.push(accompaniment);
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // Check main course assignments have accompaniments
    let main_assignments: Vec<&_> = week_plan
        .meal_assignments
        .iter()
        .filter(|a| a.course_type == "main_course")
        .collect();

    let with_accompaniments = main_assignments
        .iter()
        .filter(|a| a.accompaniment_recipe_id.is_some())
        .count();

    // AC-6: At least some main courses should have accompaniments assigned
    assert!(
        with_accompaniments > 0,
        "Main courses with accepts_accompaniment=true should have accompaniments assigned"
    );
}

/// AC-7: Test RotationState updated after week generation
#[test]
fn test_rotation_state_updated_after_generation() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();

    // Store initial state
    let initial_main_count = rotation_state.used_main_course_ids.len();
    let initial_app_count = rotation_state.used_appetizer_ids.len();
    let initial_dessert_count = rotation_state.used_dessert_ids.len();

    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());

    // AC-7: RotationState should be updated after generation
    assert!(
        rotation_state.used_main_course_ids.len() > initial_main_count,
        "Main course usage should be tracked"
    );
    assert!(
        rotation_state.used_appetizer_ids.len() > initial_app_count,
        "Appetizer usage should be tracked"
    );
    assert!(
        rotation_state.used_dessert_ids.len() > initial_dessert_count,
        "Dessert usage should be tracked"
    );

    // Verify exactly 7 main courses marked as used
    assert_eq!(
        rotation_state.used_main_course_ids.len(),
        7,
        "Should mark 7 main courses as used"
    );
}

/// AC-8: Test WeekMealPlan has status=Future and is_locked=false
#[test]
fn test_week_meal_plan_metadata() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    assert!(result.is_ok());
    let week_plan = result.unwrap();

    // AC-8: status should be Future
    use meal_planning::WeekStatus;
    assert_eq!(
        week_plan.status,
        WeekStatus::Future,
        "Newly generated week should have status=Future"
    );

    // AC-8: is_locked should be false
    assert!(
        !week_plan.is_locked,
        "Newly generated week should have is_locked=false"
    );

    // Verify start_date is Monday
    let start_date = NaiveDate::parse_from_str(&week_plan.start_date, "%Y-%m-%d").unwrap();
    assert_eq!(
        start_date.weekday(),
        Weekday::Mon,
        "start_date should be Monday"
    );

    // Verify end_date is Sunday (6 days later)
    let end_date = NaiveDate::parse_from_str(&week_plan.end_date, "%Y-%m-%d").unwrap();
    assert_eq!(
        end_date.weekday(),
        Weekday::Sun,
        "end_date should be Sunday"
    );
    assert_eq!(
        (end_date - start_date).num_days(),
        6,
        "end_date should be 6 days after start_date"
    );

    // Verify UUIDs are generated
    assert!(!week_plan.id.is_empty(), "id should be generated");
    assert!(
        !week_plan.generation_batch_id.is_empty(),
        "generation_batch_id should be generated"
    );
}

/// AC-9 Edge Case: Test insufficient main courses returns error
#[test]
fn test_insufficient_main_courses_returns_error() {
    // Create only 3 main courses (need 7 for a week)
    let mut recipes = Vec::new();

    // 10 appetizers
    for i in 0..10 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            Cuisine::Italian,
        ));
    }

    // Only 3 main courses (insufficient!)
    for i in 0..3 {
        recipes.push(create_test_recipe(
            &format!("main_{}", i),
            "main_course",
            8,
            6,
            15,
            30,
            Cuisine::Italian,
        ));
    }

    // 10 desserts
    for i in 0..10 {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            Cuisine::Italian,
        ));
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);

    // AC-9: Should return error when insufficient main courses
    assert!(
        result.is_err(),
        "Should fail with insufficient main courses"
    );

    // Verify error type (optional - depends on error implementation)
    // This ensures proper error reporting to users
}

/// AC-9 Edge Case: Test week_start_date must be Monday
#[test]
fn test_week_start_date_must_be_monday() {
    let recipes = create_balanced_recipes(10);
    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();

    // Try with Tuesday (not Monday)
    let tuesday = NaiveDate::from_ymd_opt(2025, 10, 28).unwrap();
    assert_eq!(tuesday.weekday(), Weekday::Tue);

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, tuesday);

    // AC-9: Should return error for non-Monday start date
    assert!(
        result.is_err(),
        "Should reject week_start_date that is not Monday"
    );
}

/// AC-9 Edge Case: Test exactly 7 main courses generates week successfully
#[test]
fn test_exactly_seven_main_courses_succeeds() {
    let mut recipes = Vec::new();

    // 7 appetizers
    for i in 0..7 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5,
            3,
            10,
            15,
            Cuisine::Italian,
        ));
    }

    // Exactly 7 main courses (minimum required) - quick ones for weeknights
    for i in 0..7 {
        recipes.push(create_quick_main_course(
            &format!("main_{}", i),
            Cuisine::Italian,
        ));
    }

    // 7 desserts
    for i in 0..7 {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6,
            4,
            20,
            25,
            Cuisine::Italian,
        ));
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);

    // AC-9: Should succeed with exactly 7 main courses
    assert!(
        result.is_ok(),
        "Should succeed with exactly 7 main courses (minimum required)"
    );

    let week_plan = result.unwrap();
    assert_eq!(week_plan.meal_assignments.len(), 21);
}

/// Performance Test: AC-9 - Verify generation completes in <1 second with 50 recipes
#[test]
fn test_single_week_generation_performance() {
    use std::time::Instant;

    // Create 50 recipes (generous set)
    let mut recipes = Vec::new();

    for i in 0..17 {
        recipes.push(create_test_recipe(
            &format!("app_{}", i),
            "appetizer",
            5 + (i % 5),
            3 + (i % 3),
            10,
            15,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    for i in 0..17 {
        // Alternate between quick (weeknight) and longer (weekend) recipes
        let (prep, cook) = if i % 2 == 0 {
            (10, 15) // 25 min - weeknight friendly
        } else {
            (20, 30) // 50 min - weekend friendly
        };

        recipes.push(create_test_recipe(
            &format!("main_{}", i),
            "main_course",
            8 + (i % 8),
            6 + (i % 4),
            prep,
            cook,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    for i in 0..16 {
        recipes.push(create_test_recipe(
            &format!("dessert_{}", i),
            "dessert",
            6 + (i % 6),
            4 + (i % 3),
            20,
            25,
            match i % 3 {
                0 => Cuisine::Italian,
                1 => Cuisine::Mexican,
                _ => Cuisine::Indian,
            },
        ));
    }

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();
    let week_start_date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap();

    let start = Instant::now();
    let result = generate_single_week(recipes, &preferences, &mut rotation_state, week_start_date);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Should successfully generate week");

    // AC-9: Must complete in <1 second (P95 target)
    assert!(
        duration.as_secs() < 1,
        "Single week generation took {:?}, expected <1 second",
        duration
    );

    // Log for monitoring
    println!(
        "Single week generation performance with 50 recipes: {:?}",
        duration
    );
}
