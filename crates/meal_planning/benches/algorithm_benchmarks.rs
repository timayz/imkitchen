use chrono::{NaiveDate, Weekday};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meal_planning::{
    algorithm::{select_main_course_with_preferences, RecipeForPlanning, UserPreferences},
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
    }
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

criterion_group!(benches, bench_select_main_course_100_recipes);
criterion_main!(benches);
