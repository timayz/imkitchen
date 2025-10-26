use chrono::{NaiveDate, Weekday};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meal_planning::{
    algorithm::{
        generate_multi_week_meal_plans, select_main_course_with_preferences, RecipeForPlanning,
        UserPreferences,
    },
    rotation::RotationState,
};
use recipe::Cuisine;

/// Create a test recipe with specific properties for benchmarking
fn create_bench_recipe(
    id: usize,
    prep_time: u32,
    cook_time: u32,
    ingredients: usize,
    steps: usize,
) -> RecipeForPlanning {
    // Distribute cuisines for variety
    let cuisine = match id % 5 {
        0 => Cuisine::Italian,
        1 => Cuisine::Mexican,
        2 => Cuisine::Indian,
        3 => Cuisine::Chinese,
        _ => Cuisine::Japanese,
    };

    RecipeForPlanning {
        id: format!("recipe_{}", id),
        title: format!("Test Recipe {}", id),
        recipe_type: "main_course".to_string(),
        ingredients_count: ingredients,
        instructions_count: steps,
        prep_time_min: Some(prep_time),
        cook_time_min: Some(cook_time),
        advance_prep_hours: None,
        complexity: None,
        dietary_tags: Vec::new(),
        cuisine,
        accepts_accompaniment: false,
        preferred_accompaniments: vec![],
        accompaniment_category: None,
    }
}

/// Create balanced recipe set for multi-week benchmarking
/// All main courses are weeknight-friendly (≤30min total) to avoid filtering issues
fn create_balanced_bench_recipes(count: usize) -> Vec<RecipeForPlanning> {
    let mut recipes = Vec::new();

    for i in 0..count {
        let recipe_type = match i % 3 {
            0 => "appetizer",
            1 => "main_course",
            _ => "dessert",
        };

        let cuisine = match i % 5 {
            0 => Cuisine::Italian,
            1 => Cuisine::Mexican,
            2 => Cuisine::Indian,
            3 => Cuisine::Chinese,
            _ => Cuisine::Japanese,
        };

        // For main courses, ensure they fit weeknight constraints (≤30min total)
        let (prep_time, cook_time) = if recipe_type == "main_course" {
            (10 + (i as u32 % 6), 8 + (i as u32 % 6)) // Total: 18-30 min
        } else {
            (5 + (i as u32 % 25), 10 + (i as u32 % 40)) // Appetizers/desserts: flexible
        };

        recipes.push(RecipeForPlanning {
            id: format!("recipe_{}", i),
            title: format!("Recipe {}", i),
            recipe_type: recipe_type.to_string(),
            ingredients_count: 5 + (i % 10), // 5-15 ingredients (Simple complexity)
            instructions_count: 3 + (i % 8),  // 3-11 steps (Simple complexity)
            prep_time_min: Some(prep_time),
            cook_time_min: Some(cook_time),
            advance_prep_hours: None,
            complexity: Some("simple".to_string()),
            dietary_tags: vec![],
            cuisine,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
        });
    }

    recipes
}

/// Benchmark select_main_course_with_preferences with 100 recipes (Story 7.2 AC-9)
///
/// Target: <10ms execution time (P95)
fn bench_select_main_course_100_recipes(c: &mut Criterion) {
    // Create 100 main course recipes with varied properties
    let recipes: Vec<RecipeForPlanning> = (0..100)
        .map(|i| {
            create_bench_recipe(
                i,
                5 + (i as u32 % 25),  // prep_time: 5-30 minutes
                10 + (i as u32 % 40), // cook_time: 10-50 minutes
                5 + (i % 15),         // ingredients: 5-20
                4 + (i % 12),         // steps: 4-16
            )
        })
        .collect();

    let preferences = UserPreferences::default();
    let mut rotation_state = RotationState::new();

    // Pre-populate some cuisine usage to test scoring
    rotation_state.increment_cuisine_usage(&Cuisine::Italian);
    rotation_state.increment_cuisine_usage(&Cuisine::Italian);
    rotation_state.increment_cuisine_usage(&Cuisine::Mexican);

    let date = NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(); // Tuesday (weeknight)
    let day_of_week = Weekday::Tue;

    c.bench_function("select_main_course_100_recipes", |b| {
        b.iter(|| {
            select_main_course_with_preferences(
                black_box(&recipes),
                black_box(&preferences),
                black_box(&rotation_state),
                black_box(date),
                black_box(day_of_week),
            )
        })
    });
}

/// Benchmark generate_multi_week_meal_plans with 5 weeks (Story 7.5 AC-9)
///
/// Target: <5 seconds execution time (P95) for 5 weeks with 50 recipes
fn bench_generate_multi_week_5_weeks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create 105 recipes (35 of each type) to ensure 5 weeks can be generated
    let recipes = create_balanced_bench_recipes(105);
    let user_id = "bench_user".to_string();
    let preferences = UserPreferences::default();

    c.bench_function("generate_multi_week_5_weeks", |b| {
        b.iter(|| {
            rt.block_on(async {
                generate_multi_week_meal_plans(
                    black_box(user_id.clone()),
                    black_box(recipes.clone()),
                    black_box(preferences.clone()),
                )
                .await
                .expect("Benchmark generation should succeed")
            })
        })
    });
}

criterion_group!(
    benches,
    bench_select_main_course_100_recipes,
    bench_generate_multi_week_5_weeks
);
criterion_main!(benches);
