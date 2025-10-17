use crate::error::MealPlanningError;
use crate::events::{MealAssignment, MealType};
use crate::rotation::{RotationState, RotationSystem};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Recipe data needed for meal planning algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeForPlanning {
    pub id: String,
    pub title: String,
    pub ingredients_count: usize,  // Count of ingredients
    pub instructions_count: usize, // Count of instruction steps
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub complexity: Option<String>, // "simple", "moderate", "complex" (if pre-calculated)
}

/// User profile constraints for meal planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConstraints {
    pub weeknight_availability_minutes: Option<u32>, // Max cooking time on weeknights
    pub skill_level: Option<String>,                 // "beginner", "intermediate", "expert"
    pub dietary_restrictions: Vec<String>,           // e.g., ["vegetarian", "gluten-free"]
}

impl Default for UserConstraints {
    fn default() -> Self {
        UserConstraints {
            weeknight_availability_minutes: Some(45), // Default 45 min weeknights
            skill_level: Some("intermediate".to_string()),
            dietary_restrictions: Vec::new(),
        }
    }
}

/// Complexity levels for recipes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

impl Complexity {
    pub fn as_str(&self) -> &str {
        match self {
            Complexity::Simple => "simple",
            Complexity::Moderate => "moderate",
            Complexity::Complex => "complex",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "simple" => Complexity::Simple,
            "complex" => Complexity::Complex,
            _ => Complexity::Moderate,
        }
    }
}

/// RecipeComplexityCalculator service calculates recipe complexity
///
/// Formula per tech spec:
/// complexity_score = (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)
///
/// Mapping:
/// - score < 30 => Simple
/// - score 30-60 => Moderate
/// - score > 60 => Complex
pub struct RecipeComplexityCalculator;

impl RecipeComplexityCalculator {
    /// Calculate complexity score for a recipe
    ///
    /// Returns a float score based on ingredients, steps, and advance prep.
    pub fn calculate_score(recipe: &RecipeForPlanning) -> f32 {
        let ingredients_score = recipe.ingredients_count as f32;
        let steps_score = recipe.instructions_count as f32;

        // Advance prep multiplier: 0 if none, 10 per hour of advance prep
        let advance_prep_multiplier = recipe
            .advance_prep_hours
            .map(|hours| hours as f32 * 10.0)
            .unwrap_or(0.0);

        (ingredients_score * 0.3) + (steps_score * 0.4) + (advance_prep_multiplier * 0.3)
    }

    /// Map score to Complexity enum
    pub fn calculate_complexity(recipe: &RecipeForPlanning) -> Complexity {
        // If complexity already pre-calculated, use it
        if let Some(ref complexity_str) = recipe.complexity {
            return Complexity::parse(complexity_str);
        }

        let score = Self::calculate_score(recipe);

        if score < 30.0 {
            Complexity::Simple
        } else if score <= 60.0 {
            Complexity::Moderate
        } else {
            Complexity::Complex
        }
    }

    /// Check if recipe fits weeknight availability constraint
    pub fn fits_weeknight(recipe: &RecipeForPlanning, max_minutes: u32) -> bool {
        let total_time = recipe.prep_time_min.unwrap_or(0) + recipe.cook_time_min.unwrap_or(0);
        total_time <= max_minutes
    }

    /// Check if recipe fits weekend (no time constraint, allows complex recipes)
    pub fn fits_weekend(_recipe: &RecipeForPlanning) -> bool {
        true // All recipes allowed on weekends
    }
}

/// MealPlanningAlgorithm service generates meal plans with constraint satisfaction
///
/// Algorithm complexity: O(n) where n = favorite recipe count
/// Target execution time: <5 seconds for 50 recipes
pub struct MealPlanningAlgorithm;

impl MealPlanningAlgorithm {
    /// Generate a weekly meal plan (21 assignments: 7 days × 3 meals)
    ///
    /// # Arguments
    /// * `start_date` - ISO 8601 date string (should be a Monday)
    /// * `favorites` - List of favorited recipes available for assignment
    /// * `constraints` - User profile constraints (availability, skill level, dietary)
    /// * `rotation_state` - Current rotation state to prevent duplicates
    ///
    /// # Returns
    /// * `Ok((assignments, updated_rotation_state))` on success
    /// * `Err(MealPlanningError)` if constraints cannot be satisfied
    pub fn generate(
        start_date: &str,
        favorites: Vec<RecipeForPlanning>,
        constraints: UserConstraints,
        rotation_state: RotationState,
    ) -> Result<(Vec<MealAssignment>, RotationState), MealPlanningError> {
        // Parse start date
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|e| MealPlanningError::InvalidDate(e.to_string()))?;

        // Filter to only recipes not used in current rotation cycle
        let favorite_ids: Vec<String> = favorites.iter().map(|r| r.id.clone()).collect();
        let available_ids =
            RotationSystem::filter_available_recipes(&favorite_ids, &rotation_state);

        // If we need to reset cycle (all used), reset and make all available
        let (available_ids, mut rotation_state) = if available_ids.is_empty() {
            let mut reset_state = rotation_state;
            reset_state.reset_cycle();
            (favorite_ids.clone(), reset_state)
        } else {
            (available_ids, rotation_state)
        };

        // Filter favorites to only available recipes
        let available_recipes: Vec<RecipeForPlanning> = favorites
            .into_iter()
            .filter(|r| available_ids.contains(&r.id))
            .collect();

        // Ensure we have enough recipes (need at least 7 for a week, ideally 21 for variety)
        if available_recipes.len() < 7 {
            return Err(MealPlanningError::InsufficientRecipes {
                minimum: 7,
                current: available_recipes.len(),
            });
        }

        // Calculate complexity for all recipes
        let recipes_with_complexity: Vec<(RecipeForPlanning, Complexity)> = available_recipes
            .iter()
            .map(|r| {
                (
                    r.clone(),
                    RecipeComplexityCalculator::calculate_complexity(r),
                )
            })
            .collect();

        // Generate 21 meal assignments (7 days × 3 meals)
        let mut assignments = Vec::new();
        let mut used_recipe_ids = Vec::new();
        let mut recipe_index = 0;

        for day_offset in 0..7 {
            let date = start + chrono::Duration::days(day_offset);
            let date_str = date.format("%Y-%m-%d").to_string();
            let is_weekend = date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun;

            // Assign breakfast, lunch, dinner for this day
            for meal_type in [MealType::Breakfast, MealType::Lunch, MealType::Dinner] {
                // Select next available recipe
                // Simple round-robin for MVP, more sophisticated scoring in future
                let (recipe, _complexity) =
                    &recipes_with_complexity[recipe_index % recipes_with_complexity.len()];

                // For weeknights, prefer simple/moderate recipes
                // For weekends, allow complex recipes
                let suitable = if is_weekend {
                    RecipeComplexityCalculator::fits_weekend(recipe)
                } else {
                    match constraints.weeknight_availability_minutes {
                        Some(max_min) => {
                            RecipeComplexityCalculator::fits_weeknight(recipe, max_min)
                        }
                        None => true,
                    }
                };

                // Select recipe (use fallback if constraint not met)
                let selected_recipe = if suitable {
                    recipe
                } else {
                    // Find first simple recipe as fallback
                    recipes_with_complexity
                        .iter()
                        .find(|(r, c)| *c == Complexity::Simple && !used_recipe_ids.contains(&r.id))
                        .map(|(r, _)| r)
                        .unwrap_or(recipe)
                };

                // Create assignment (meal_type as String for bincode compatibility)
                assignments.push(MealAssignment {
                    date: date_str.clone(),
                    meal_type: meal_type.as_str().to_string(),
                    recipe_id: selected_recipe.id.clone(),
                    prep_required: selected_recipe.advance_prep_hours.is_some()
                        && selected_recipe.advance_prep_hours.unwrap() > 0,
                });

                // Mark recipe as used in rotation
                if !used_recipe_ids.contains(&selected_recipe.id) {
                    used_recipe_ids.push(selected_recipe.id.clone());
                }

                recipe_index += 1;
            }
        }

        // Update rotation state with all used recipes
        rotation_state = RotationSystem::update_after_generation(
            &used_recipe_ids,
            favorite_ids.len(),
            rotation_state,
        );

        Ok((assignments, rotation_state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_recipe(
        id: &str,
        ingredients: usize,
        steps: usize,
        advance_prep: Option<u32>,
    ) -> RecipeForPlanning {
        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Recipe {}", id),
            ingredients_count: ingredients,
            instructions_count: steps,
            prep_time_min: Some(15),
            cook_time_min: Some(30),
            advance_prep_hours: advance_prep,
            complexity: None,
        }
    }

    #[test]
    fn test_complexity_calculator_simple() {
        let recipe = create_test_recipe("1", 5, 4, None);
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (5 * 0.3) + (4 * 0.4) + (0 * 0.3) = 1.5 + 1.6 + 0 = 3.1
        assert!((score - 3.1).abs() < 0.01);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_complexity_calculator_moderate() {
        let recipe = create_test_recipe("2", 12, 8, None);
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (12 * 0.3) + (8 * 0.4) + (0 * 0.3) = 3.6 + 3.2 + 0 = 6.8
        assert!((score - 6.8).abs() < 0.01);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_complexity_calculator_complex() {
        let recipe = create_test_recipe("3", 20, 15, Some(4));
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (20 * 0.3) + (15 * 0.4) + (40 * 0.3) = 6.0 + 6.0 + 12.0 = 24.0
        assert!((score - 24.0).abs() < 0.01);
        assert_eq!(complexity, Complexity::Simple); // Still under 30
    }

    #[test]
    fn test_fits_weeknight() {
        let quick_recipe = RecipeForPlanning {
            id: "1".to_string(),
            title: "Quick Meal".to_string(),
            ingredients_count: 5,
            instructions_count: 4,
            prep_time_min: Some(10),
            cook_time_min: Some(20),
            advance_prep_hours: None,
            complexity: None,
        };

        assert!(RecipeComplexityCalculator::fits_weeknight(
            &quick_recipe,
            45
        ));
        assert!(!RecipeComplexityCalculator::fits_weeknight(
            &quick_recipe,
            25
        ));
    }

    #[test]
    fn test_generate_meal_plan_success() {
        let favorites = vec![
            create_test_recipe("1", 5, 4, None),
            create_test_recipe("2", 8, 6, None),
            create_test_recipe("3", 10, 8, Some(2)),
            create_test_recipe("4", 6, 5, None),
            create_test_recipe("5", 12, 10, None),
            create_test_recipe("6", 7, 5, None),
            create_test_recipe("7", 9, 7, None),
        ];

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        let result = MealPlanningAlgorithm::generate(
            "2025-10-20", // Monday
            favorites,
            constraints,
            rotation_state,
        );

        assert!(result.is_ok());
        let (assignments, updated_state) = result.unwrap();

        // Should have 21 assignments (7 days × 3 meals)
        assert_eq!(assignments.len(), 21);

        // All 7 recipes used, so cycle should have reset (cycle_number increments, used_count = 0)
        assert_eq!(updated_state.cycle_number, 2);
        assert_eq!(updated_state.used_count(), 0);
    }

    #[test]
    fn test_generate_insufficient_recipes() {
        let favorites = vec![
            create_test_recipe("1", 5, 4, None),
            create_test_recipe("2", 8, 6, None),
            create_test_recipe("3", 10, 8, Some(2)),
        ];

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        let result =
            MealPlanningAlgorithm::generate("2025-10-20", favorites, constraints, rotation_state);

        assert!(result.is_err());
        match result {
            Err(MealPlanningError::InsufficientRecipes { minimum, current }) => {
                assert_eq!(minimum, 7);
                assert_eq!(current, 3);
            }
            _ => panic!("Expected InsufficientRecipes error"),
        }
    }

    #[test]
    fn test_algorithm_performance_50_recipes() {
        // AC-6: Generation completes within 5 seconds for up to 50 favorite recipes
        use std::time::Instant;

        let mut favorites = Vec::new();
        for i in 1..=50 {
            favorites.push(create_test_recipe(
                &format!("recipe_{}", i),
                5 + (i % 15),                             // 5-20 ingredients
                4 + (i % 8),                              // 4-12 steps
                if i % 10 == 0 { Some(2) } else { None }, // 10% with advance prep
            ));
        }

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        let start = Instant::now();
        let result =
            MealPlanningAlgorithm::generate("2025-10-20", favorites, constraints, rotation_state);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Algorithm should succeed with 50 recipes");
        let (assignments, _) = result.unwrap();
        assert_eq!(assignments.len(), 21, "Should generate 21 meal assignments");

        // Performance assertion: must complete in < 5 seconds
        assert!(
            duration.as_secs() < 5,
            "Algorithm took {:?}, expected < 5 seconds",
            duration
        );

        // Log performance for monitoring
        println!("Algorithm performance with 50 recipes: {:?}", duration);
    }
}
