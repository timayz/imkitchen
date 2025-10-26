use chrono::NaiveDate;
use meal_planning::algorithm::{RecipeForPlanning, UserConstraints};
use meal_planning::constraints::CourseType;
use meal_planning::constraints::*;

fn create_test_recipe(
    id: &str,
    ingredients: usize,
    steps: usize,
    advance_prep: Option<u32>,
    prep_time: u32,
    cook_time: u32,
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
    }
}

#[test]
fn test_availability_constraint_weeknight_matches() {
    let recipe = create_test_recipe("1", 7, 5, None, 15, 25); // 40 min total
    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: Vec::new(),
    };

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(); // Monday
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = AvailabilityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Recipe fits weeknight availability, should score high
    assert!(
        score > 0.7,
        "Expected high score for fitting recipe, got {}",
        score
    );
}

#[test]
fn test_availability_constraint_weeknight_too_long() {
    let recipe = create_test_recipe("1", 10, 8, None, 30, 45); // 75 min total
    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: Vec::new(),
    };

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(); // Monday
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = AvailabilityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Recipe doesn't fit weeknight availability, should score low
    assert!(
        score < 0.3,
        "Expected low score for long recipe, got {}",
        score
    );
}

#[test]
fn test_availability_constraint_weekend_allows_all() {
    let recipe = create_test_recipe("1", 15, 12, Some(2), 45, 60); // Complex, long recipe
    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: Vec::new(),
    };

    let saturday = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
    let slot = MealSlot {
        date: saturday,
        course_type: CourseType::Dessert,
    };

    let constraint = AvailabilityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Weekend allows all recipes, should score high
    assert!(
        score > 0.8,
        "Expected high score for weekend slot, got {}",
        score
    );
}

#[test]
fn test_complexity_constraint_simple_weeknight() {
    let recipe = create_test_recipe("1", 6, 4, None, 10, 20); // Simple recipe
    let user_constraints = UserConstraints::default();

    let tuesday = NaiveDate::from_ymd_opt(2025, 10, 21).unwrap(); // Tuesday
    let slot = MealSlot {
        date: tuesday,
        course_type: CourseType::Dessert,
    };

    let constraint = ComplexityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Simple recipe on weeknight should score high
    assert!(
        score > 0.7,
        "Expected high score for simple weeknight recipe, got {}",
        score
    );
}

#[test]
fn test_complexity_constraint_complex_weekend() {
    // Use same complex recipe as weeknight test for consistency
    let recipe = create_test_recipe("1", 40, 50, Some(4), 30, 60); // Complex recipe
    let user_constraints = UserConstraints::default();

    let saturday = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
    let slot = MealSlot {
        date: saturday,
        course_type: CourseType::Dessert,
    };

    let constraint = ComplexityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Complex recipe on weekend should score high
    assert!(
        score > 0.8,
        "Expected high score for complex weekend recipe, got {}",
        score
    );
}

#[test]
fn test_complexity_constraint_complex_weeknight() {
    // Score = (25 * 0.3) + (20 * 0.4) + (100 * 0.3) = 7.5 + 8.0 + 30.0 = 45.5 (Moderate)
    // For Complex, need > 60: (30 * 0.3) + (25 * 0.4) + (100 * 0.3) = 9 + 10 + 30 = 49 (still moderate)
    // Let's use 40 ingredients, 50 steps: (40 * 0.3) + (50 * 0.4) + (100 * 0.3) = 12 + 20 + 30 = 62 (Complex!)
    let recipe = create_test_recipe("1", 40, 50, Some(4), 30, 60); // Complex recipe
    let user_constraints = UserConstraints::default();

    let tuesday = NaiveDate::from_ymd_opt(2025, 10, 21).unwrap(); // Tuesday
    let slot = MealSlot {
        date: tuesday,
        course_type: CourseType::Dessert,
    };

    let constraint = ComplexityConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Complex recipe on weeknight should score low
    assert!(
        score < 0.4,
        "Expected low score for complex weeknight recipe, got {}",
        score
    );
}

#[test]
fn test_advance_prep_constraint_sufficient_lead_time() {
    let recipe = create_test_recipe("1", 10, 8, Some(4), 20, 30); // 4-hour marinade
    let user_constraints = UserConstraints::default();

    let wednesday = NaiveDate::from_ymd_opt(2025, 10, 22).unwrap(); // Wednesday
    let slot = MealSlot {
        date: wednesday,
        course_type: CourseType::Dessert,
    };

    let constraint = AdvancePrepConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Recipe with advance prep has sufficient lead time (can prep Tuesday evening)
    assert!(
        score > 0.5,
        "Expected positive score for sufficient lead time, got {}",
        score
    );
}

#[test]
fn test_advance_prep_constraint_no_prep_required() {
    let recipe = create_test_recipe("1", 8, 6, None, 15, 25); // No advance prep
    let user_constraints = UserConstraints::default();

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(); // Monday
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = AdvancePrepConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // No advance prep requirement is neutral/positive
    assert!(
        score >= 0.5,
        "Expected neutral/positive score for no prep, got {}",
        score
    );
}

#[test]
fn test_dietary_constraint_vegetarian_restriction() {
    // Recipe without vegetarian tag
    let recipe = create_test_recipe("1", 10, 8, None, 20, 30);

    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: vec!["vegetarian".to_string()],
    };

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = DietaryConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Recipe doesn't have vegetarian tag, so should be disqualified
    assert_eq!(
        score, 0.0,
        "Expected recipe without vegetarian tag to be disqualified"
    );
}

#[test]
fn test_dietary_constraint_no_restrictions() {
    // Recipe without any dietary tags
    let recipe = create_test_recipe("1", 10, 8, None, 20, 30);

    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: vec![],
    };

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = DietaryConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // No dietary restrictions, all recipes should be acceptable
    assert_eq!(
        score, 1.0,
        "Expected neutral score when user has no dietary restrictions"
    );
}

#[test]
fn test_freshness_constraint_early_week() {
    let recipe = create_test_recipe("1", 10, 8, None, 20, 30);
    let user_constraints = UserConstraints::default();

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap(); // Monday (day 1)
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    let constraint = FreshnessConstraint;
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // Early week slot should score well for freshness-sensitive recipes
    assert!(
        score >= 0.5,
        "Expected positive score for early week, got {}",
        score
    );
}

#[test]
fn test_equipment_conflict_constraint_no_conflict() {
    let recipe = create_test_recipe("1", 8, 6, None, 15, 25);
    let user_constraints = UserConstraints::default();

    let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
    let slot = MealSlot {
        date: monday,
        course_type: CourseType::Dessert,
    };

    // Empty day assignments (no conflicts)
    let day_assignments: Vec<DayAssignment> = Vec::new();

    let constraint = EquipmentConflictConstraint::new(day_assignments);
    let score = constraint.evaluate(&recipe, &slot, &user_constraints);

    // No conflicts, should score high
    assert!(
        score >= 0.9,
        "Expected high score with no conflicts, got {}",
        score
    );
}

#[test]
fn test_all_constraints_together() {
    // Integration test: simple recipe on weeknight should score well across all constraints
    let recipe = create_test_recipe("1", 6, 4, None, 10, 20);
    let user_constraints = UserConstraints {
        weeknight_availability_minutes: Some(45),
        dietary_restrictions: Vec::new(),
    };

    let tuesday = NaiveDate::from_ymd_opt(2025, 10, 21).unwrap();
    let slot = MealSlot {
        date: tuesday,
        course_type: CourseType::Dessert,
    };

    // Test all constraints
    let availability = AvailabilityConstraint;
    let complexity = ComplexityConstraint;
    let advance_prep = AdvancePrepConstraint;
    let dietary = DietaryConstraint;
    let freshness = FreshnessConstraint;
    let equipment = EquipmentConflictConstraint::new(Vec::new());

    let scores = [
        availability.evaluate(&recipe, &slot, &user_constraints),
        complexity.evaluate(&recipe, &slot, &user_constraints),
        advance_prep.evaluate(&recipe, &slot, &user_constraints),
        dietary.evaluate(&recipe, &slot, &user_constraints),
        freshness.evaluate(&recipe, &slot, &user_constraints),
        equipment.evaluate(&recipe, &slot, &user_constraints),
    ];

    // All scores should be positive for a well-matched recipe
    for (i, score) in scores.iter().enumerate() {
        assert!(
            *score >= 0.5,
            "Constraint {} scored {}, expected >= 0.5",
            i,
            score
        );
    }
}
