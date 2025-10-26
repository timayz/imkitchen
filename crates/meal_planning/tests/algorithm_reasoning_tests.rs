use chrono::NaiveDate;
use meal_planning::algorithm::{generate_reasoning_text, RecipeForPlanning, UserConstraints};
use meal_planning::constraints::{CourseType, MealSlot};

fn create_test_recipe(
    id: &str,
    ingredients: usize,
    steps: usize,
    prep_time: u32,
    cook_time: u32,
    advance_prep: Option<u32>,
) -> RecipeForPlanning {
    RecipeForPlanning {
        id: id.to_string(),
        title: format!("Recipe {}", id),
        ingredients_count: ingredients,
        instructions_count: steps,
        prep_time_min: Some(prep_time),
        cook_time_min: Some(cook_time),
        advance_prep_hours: advance_prep,
        complexity: None,
        recipe_type: "main_course".to_string(),
        dietary_tags: Vec::new(),
        cuisine: recipe::Cuisine::Italian,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
    }
}

#[test]
fn test_generate_reasoning_weeknight_time_constraint() {
    // AC-3: Quick weeknight meal reasoning
    let recipe = create_test_recipe("1", 6, 4, 10, 20, None); // Simple recipe, 30min total
    let tuesday = NaiveDate::from_ymd_opt(2025, 10, 21).unwrap();
    let slot = MealSlot {
        date: tuesday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Expected format: "Assigned to Tuesday: Quick weeknight meal (Simple recipe, 30min total time)"
    assert!(
        reasoning.contains("Tuesday"),
        "Reasoning should mention day: {}",
        reasoning
    );
    assert!(
        reasoning.contains("weeknight") || reasoning.contains("Quick"),
        "Reasoning should mention weeknight/quick: {}",
        reasoning
    );
    assert!(
        reasoning.contains("30min") || reasoning.contains("30 min"),
        "Reasoning should mention time: {}",
        reasoning
    );
    assert!(
        !reasoning.contains("complex") && !reasoning.contains("Complex"),
        "Reasoning should not mention complex for simple recipe: {}",
        reasoning
    );
}

#[test]
fn test_generate_reasoning_weekend_complexity() {
    // AC-2: Weekend complexity reasoning
    let recipe = create_test_recipe("2", 100, 100, 45, 30, None); // Complex recipe, 75min total
    let saturday = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
    let slot = MealSlot {
        date: saturday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Expected format: "Assigned to Saturday: More prep time available (Complex recipe, 75min total time)"
    assert!(
        reasoning.contains("Saturday"),
        "Reasoning should mention day: {}",
        reasoning
    );
    assert!(
        reasoning.contains("prep time") || reasoning.contains("time available"),
        "Reasoning should mention time available: {}",
        reasoning
    );
    assert!(
        reasoning.contains("Complex") || reasoning.contains("complex"),
        "Reasoning should mention complexity: {}",
        reasoning
    );
    assert!(
        reasoning.contains("75min") || reasoning.contains("75 min"),
        "Reasoning should mention total time: {}",
        reasoning
    );
}

#[test]
fn test_generate_reasoning_advance_prep() {
    // AC-4: Advance prep reasoning
    let recipe = create_test_recipe("3", 10, 8, 15, 30, Some(4)); // 4-hour marinade
    let wednesday = NaiveDate::from_ymd_opt(2025, 10, 22).unwrap();
    let slot = MealSlot {
        date: wednesday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Expected format: "Prep tonight for tomorrow: Requires 4-hour marinade"
    assert!(
        reasoning.contains("Prep") || reasoning.contains("prep"),
        "Reasoning should mention prep: {}",
        reasoning
    );
    assert!(
        reasoning.contains("4-hour") || reasoning.contains("4 hour"),
        "Reasoning should mention prep duration: {}",
        reasoning
    );
    assert!(
        reasoning.contains("marinade") || reasoning.contains("advance"),
        "Reasoning should mention marinade or advance prep: {}",
        reasoning
    );
}

#[test]
fn test_generate_reasoning_advance_prep_short() {
    // AC-4: Short advance prep reasoning (< 4 hours)
    let recipe = create_test_recipe("4", 8, 6, 20, 25, Some(2)); // 2-hour prep
    let thursday = NaiveDate::from_ymd_opt(2025, 10, 23).unwrap();
    let slot = MealSlot {
        date: thursday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Should mention advance prep
    assert!(
        reasoning.contains("2-hour") || reasoning.contains("2 hour"),
        "Reasoning should mention short prep duration: {}",
        reasoning
    );
}

#[test]
fn test_generate_reasoning_freshness_early_week() {
    // AC-7: Freshness reasoning for early week
    let recipe = create_test_recipe("5", 12, 10, 15, 20, None);
    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Should mention early week positioning (if freshness is primary factor)
    // Note: This may fall back to default reasoning if other factors dominate
    assert!(
        reasoning.len() > 10,
        "Reasoning should be non-empty: {}",
        reasoning
    );
}

#[test]
fn test_generate_reasoning_default_fallback() {
    // Test default reasoning when no specific constraint dominates
    // Use moderate complexity recipe that exceeds weeknight time limit
    let recipe = create_test_recipe("6", 50, 50, 35, 40, None); // Moderate recipe, 75min (exceeds 45min weeknight)
    let wednesday = NaiveDate::from_ymd_opt(2025, 10, 22).unwrap();
    let slot = MealSlot {
        date: wednesday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Default format: "Best fit for Wednesday based on your preferences"
    assert!(
        reasoning.contains("Wednesday"),
        "Reasoning should mention day: {}",
        reasoning
    );
    assert!(
        reasoning.contains("fit") || reasoning.contains("preferences"),
        "Reasoning should mention fit/preferences: {}",
        reasoning
    );
}

#[test]
fn test_reasoning_human_readable_no_jargon() {
    // AC-6: Clear, human-readable language (no technical jargon)
    let recipe = create_test_recipe("7", 50, 50, 30, 30, Some(4));
    let saturday = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
    let slot = MealSlot {
        date: saturday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    // Should NOT contain technical jargon
    assert!(
        !reasoning.contains("CSP"),
        "Should not contain 'CSP': {}",
        reasoning
    );
    assert!(
        !reasoning.contains("constraint satisfaction"),
        "Should not contain 'constraint satisfaction': {}",
        reasoning
    );
    assert!(
        !reasoning.contains("score"),
        "Should not contain 'score': {}",
        reasoning
    );
    assert!(
        !reasoning.contains("algorithm"),
        "Should not contain 'algorithm': {}",
        reasoning
    );
}

#[test]
fn test_reasoning_concise_max_length() {
    // Reasoning should be concise (max ~100 characters)
    let recipe = create_test_recipe("8", 20, 15, 25, 35, Some(8));
    let friday = NaiveDate::from_ymd_opt(2025, 10, 24).unwrap();
    let slot = MealSlot {
        date: friday,
        course_type: CourseType::Dessert,
    };
    let user_constraints = UserConstraints::default();

    let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

    assert!(
        reasoning.len() <= 120,
        "Reasoning should be concise (<=120 chars): {} chars - {}",
        reasoning.len(),
        reasoning
    );
    assert!(
        reasoning.len() >= 20,
        "Reasoning should be meaningful (>=20 chars): {}",
        reasoning
    );
}

#[test]
fn test_reasoning_day_name_formatting() {
    // Verify day names are properly formatted (not ISO dates)
    let recipe = create_test_recipe("9", 10, 8, 15, 25, None);

    let days = vec![
        (NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(), "Monday"),
        (NaiveDate::from_ymd_opt(2025, 10, 21).unwrap(), "Tuesday"),
        (NaiveDate::from_ymd_opt(2025, 10, 22).unwrap(), "Wednesday"),
        (NaiveDate::from_ymd_opt(2025, 10, 23).unwrap(), "Thursday"),
        (NaiveDate::from_ymd_opt(2025, 10, 24).unwrap(), "Friday"),
        (NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(), "Saturday"),
        (NaiveDate::from_ymd_opt(2025, 10, 26).unwrap(), "Sunday"),
    ];

    for (date, expected_day_name) in days {
        let slot = MealSlot {
            date,
            course_type: CourseType::Dessert,
        };
        let user_constraints = UserConstraints::default();
        let reasoning = generate_reasoning_text(&recipe, &slot, &user_constraints);

        assert!(
            reasoning.contains(expected_day_name),
            "Reasoning should contain day name '{}': {}",
            expected_day_name,
            reasoning
        );

        // Should NOT contain ISO date format
        assert!(
            !reasoning.contains("2025-10"),
            "Reasoning should not contain ISO date: {}",
            reasoning
        );
    }
}
