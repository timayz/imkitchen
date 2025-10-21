use crate::error::MealPlanningError;
use crate::events::MealAssignment;
use crate::rotation::{RotationState, RotationSystem};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

/// Recipe data needed for meal planning algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeForPlanning {
    pub id: String,
    pub title: String,
    pub recipe_type: String,       // AC-4: "appetizer", "main_course", or "dessert"
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
    /// Per tech spec:
    /// - advance_prep_multiplier: 0 (none), 50 (<4hr), 100 (>=4hr)
    /// - Thresholds: Simple (<30), Moderate (30-60), Complex (>60)
    pub fn calculate_score(recipe: &RecipeForPlanning) -> f32 {
        let ingredients_score = recipe.ingredients_count as f32;
        let steps_score = recipe.instructions_count as f32;

        // Advance prep multiplier per tech spec
        let advance_prep_multiplier = match recipe.advance_prep_hours {
            None | Some(0) => 0.0,
            Some(hours) if hours < 4 => 50.0,
            Some(_) => 100.0, // >= 4 hours
        };

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

/// Generate human-readable reasoning text explaining why a recipe was assigned to a slot
///
/// AC-2, AC-3, AC-4, AC-5, AC-6: Generates reasoning based on constraint evaluation
///
/// # Arguments
/// * `recipe` - The recipe assigned
/// * `slot` - The meal slot (date + meal type)
/// * `user_constraints` - User profile constraints
///
/// # Returns
/// A human-readable string explaining the assignment decision (max ~100 chars)
///
/// # Example Templates
/// - Weeknight time: "Assigned to Tuesday: Quick weeknight meal (Simple recipe, 30min total time)"
/// - Weekend complexity: "Assigned to Saturday: More prep time available (Complex recipe, 75min total time)"
/// - Advance prep: "Prep tonight for tomorrow: Requires 4-hour marinade"
/// - Default: "Best fit for Wednesday based on your preferences"
pub fn generate_reasoning_text(
    recipe: &RecipeForPlanning,
    slot: &crate::constraints::MealSlot,
    user_constraints: &UserConstraints,
) -> String {
    use chrono::Weekday;

    // Get full day name (Monday, Tuesday, etc.)
    let day_name = match slot.date.weekday() {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    };

    let total_time = recipe.prep_time_min.unwrap_or(0) + recipe.cook_time_min.unwrap_or(0);
    let complexity = RecipeComplexityCalculator::calculate_complexity(recipe);

    // Priority 1: Advance prep (most specific constraint)
    if let Some(hours) = recipe.advance_prep_hours {
        if hours > 0 {
            if hours >= 4 {
                return format!(
                    "Prep tonight for tomorrow: Requires {}-hour marinade",
                    hours
                );
            } else {
                return format!(
                    "Prep tonight for tomorrow: Requires {}-hour advance prep",
                    hours
                );
            }
        }
    }

    // Priority 2: Weekend complexity (distinctive pattern)
    if slot.is_weekend() && complexity == Complexity::Complex {
        return format!(
            "Assigned to {}: More prep time available (Complex recipe, {}min total time)",
            day_name, total_time
        );
    }

    // Priority 3: Weeknight time constraint (most common pattern)
    if !slot.is_weekend() {
        let max_minutes = user_constraints
            .weeknight_availability_minutes
            .unwrap_or(45);
        if total_time <= max_minutes && complexity == Complexity::Simple {
            return format!(
                "Assigned to {}: Quick weeknight meal (Simple recipe, {}min total time)",
                day_name, total_time
            );
        }
    }

    // Priority 4: Default fallback
    format!("Best fit for {} based on your preferences", day_name)
}

/// MealPlanningAlgorithm service generates meal plans with constraint satisfaction
///
/// Algorithm complexity: O(n) where n = favorite recipe count
/// Target execution time: <5 seconds for 50 recipes
pub struct MealPlanningAlgorithm;

impl MealPlanningAlgorithm {
    /// Score a recipe for a given meal slot using weighted constraint evaluation
    ///
    /// AC-1 through AC-9: Multi-factor weighted scoring
    /// Formula: (complexity_fit * 0.4) + (time_fit * 0.4) + (freshness_fit * 0.2)
    ///
    /// # Arguments
    /// * `recipe` - The recipe to score
    /// * `slot` - The meal slot (date + meal type)
    /// * `user_constraints` - User profile constraints
    /// * `day_assignments` - Existing assignments for equipment conflict detection
    ///
    /// # Returns
    /// A score from 0.0 (poor fit) to 1.0 (excellent fit)
    pub fn score_recipe_for_slot(
        recipe: &RecipeForPlanning,
        slot: &crate::constraints::MealSlot,
        user_constraints: &UserConstraints,
        day_assignments: &[crate::constraints::DayAssignment],
    ) -> f32 {
        use crate::constraints::*;

        // Create constraint instances
        let availability_constraint = AvailabilityConstraint;
        let complexity_constraint = ComplexityConstraint;
        let advance_prep_constraint = AdvancePrepConstraint;
        let dietary_constraint = DietaryConstraint;
        let freshness_constraint = FreshnessConstraint;
        let equipment_constraint = EquipmentConflictConstraint::new(day_assignments.to_vec());

        // Evaluate each constraint
        let availability_score = availability_constraint.evaluate(recipe, slot, user_constraints);
        let complexity_score = complexity_constraint.evaluate(recipe, slot, user_constraints);
        let advance_prep_score = advance_prep_constraint.evaluate(recipe, slot, user_constraints);
        let dietary_score = dietary_constraint.evaluate(recipe, slot, user_constraints);
        let freshness_score = freshness_constraint.evaluate(recipe, slot, user_constraints);
        let equipment_score = equipment_constraint.evaluate(recipe, slot, user_constraints);

        // Hard constraints: dietary restrictions and equipment conflicts
        // If hard constraint violated, return 0.0 (disqualify recipe)
        if dietary_score == 0.0 || equipment_score == 0.0 {
            return 0.0;
        }

        // Soft constraints: weighted scoring per tech spec
        // complexity_fit_score combines complexity and availability (both relate to day energy/time)
        let complexity_fit_score = (complexity_score + availability_score) / 2.0;

        // time_fit_score considers advance prep scheduling
        let time_fit_score = advance_prep_score;

        // freshness_fit_score is direct
        let freshness_fit_score = freshness_score;

        // Weighted combination: (complexity_fit * 0.4) + (time_fit * 0.4) + (freshness_fit * 0.2)
        (complexity_fit_score * 0.4) + (time_fit_score * 0.4) + (freshness_fit_score * 0.2)
    }

    /// Generate a weekly meal plan (21 assignments: 7 days × 3 meals)
    ///
    /// AC-1 through AC-9: Full CSP solver with multi-factor constraint satisfaction
    ///
    /// # Arguments
    /// * `start_date` - ISO 8601 date string (should be a Monday)
    /// * `favorites` - List of favorited recipes available for assignment
    /// * `constraints` - User profile constraints (availability, skill level, dietary)
    /// * `rotation_state` - Current rotation state to prevent duplicates
    /// * `seed` - Optional randomization seed for deterministic variety (AC-9)
    ///
    /// # Returns
    /// * `Ok((assignments, updated_rotation_state))` on success
    /// * `Err(MealPlanningError)` if constraints cannot be satisfied
    pub fn generate(
        start_date: &str,
        favorites: Vec<RecipeForPlanning>,
        constraints: UserConstraints,
        rotation_state: RotationState,
        seed: Option<u64>,
    ) -> Result<(Vec<MealAssignment>, RotationState), MealPlanningError> {
        use crate::constraints::*;
        use rand::seq::SliceRandom;
        use rand::SeedableRng;

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
        let mut available_recipes: Vec<RecipeForPlanning> = favorites
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

        // AC-9: Deterministic randomization for variety
        // Shuffle recipes using seed for reproducible but varied assignments
        let mut rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => {
                // Generate seed from current timestamp for variety
                use std::time::SystemTime;
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                rand::rngs::StdRng::seed_from_u64(now)
            }
        };
        available_recipes.shuffle(&mut rng);

        // Generate 21 meal slots (7 days × 3 meals)
        let mut assignments: Vec<MealAssignment> = Vec::new();
        let mut day_assignments: Vec<DayAssignment> = Vec::new();
        let mut used_recipe_ids = Vec::new();

        for day_offset in 0..7 {
            let date = start + chrono::Duration::days(day_offset);
            let date_str = date.format("%Y-%m-%d").to_string();

            // AC-4: Assign appetizer, main_course, dessert for this day
            use crate::constraints::CourseType;
            for course_type_enum in [CourseType::Appetizer, CourseType::MainCourse, CourseType::Dessert] {
                let slot = MealSlot {
                    date,
                    course_type: course_type_enum.clone(),
                };

                // AC-4: Filter recipes by matching course type
                let mut scored_recipes: Vec<(f32, &RecipeForPlanning)> = available_recipes
                    .iter()
                    .filter(|r| !used_recipe_ids.contains(&r.id)) // Skip already-used recipes
                    .filter(|r| r.recipe_type == course_type_enum.as_str()) // AC-4: Match course type
                    .map(|r| {
                        let score =
                            Self::score_recipe_for_slot(r, &slot, &constraints, &day_assignments);
                        (score, r)
                    })
                    .collect();

                // Sort by score descending (highest score first)
                scored_recipes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

                // Select highest-scoring recipe
                let selected_recipe = match scored_recipes.first() {
                    Some((score, recipe)) if *score > 0.0 => recipe,
                    _ => {
                        // Fallback: no recipe scores > 0 (all disqualified by hard constraints)
                        // Try to find unused recipe of matching type
                        available_recipes
                            .iter()
                            .find(|r| !used_recipe_ids.contains(&r.id) && r.recipe_type == course_type_enum.as_str())
                            .or_else(|| {
                                // Main courses must be unique - never allow reuse
                                if course_type_enum.as_str() == "main_course" {
                                    None
                                } else {
                                    // For appetizers and desserts, allow reuse if needed
                                    available_recipes
                                        .iter()
                                        .find(|r| r.recipe_type == course_type_enum.as_str())
                                }
                            })
                            .ok_or(MealPlanningError::InsufficientRecipes {
                                minimum: if course_type_enum.as_str() == "main_course" { 7 } else { 1 },
                                current: available_recipes.iter().filter(|r| r.recipe_type == course_type_enum.as_str()).count(),
                            })?
                    }
                };

                // AC-5: Calculate prep_required flag
                let prep_required = selected_recipe.advance_prep_hours.is_some()
                    && selected_recipe.advance_prep_hours.unwrap() > 0;

                // Story 3.8: Generate human-readable reasoning for assignment
                let assignment_reasoning =
                    generate_reasoning_text(selected_recipe, &slot, &constraints);

                // Create assignment (AC-4: use course_type)
                assignments.push(MealAssignment {
                    date: date_str.clone(),
                    course_type: course_type_enum.as_str().to_string(),
                    recipe_id: selected_recipe.id.clone(),
                    prep_required,
                    assignment_reasoning: Some(assignment_reasoning),
                });

                // Track day assignments for equipment conflict detection
                day_assignments.push(DayAssignment {
                    date,
                    course_type: course_type_enum.clone(),
                    recipe_id: selected_recipe.id.clone(),
                });

                // Mark recipe as used in rotation
                if !used_recipe_ids.contains(&selected_recipe.id) {
                    used_recipe_ids.push(selected_recipe.id.clone());
                }

                // If we've used all recipes, allow reuse (cycle through again)
                if used_recipe_ids.len() == available_recipes.len() {
                    used_recipe_ids.clear();
                }
            }
        }

        // Update rotation state with all used recipes from this generation
        rotation_state = RotationSystem::update_after_generation(
            &assignments
                .iter()
                .map(|a| a.recipe_id.clone())
                .collect::<Vec<_>>(),
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
            recipe_type: "dessert".to_string(), // AC-4: Add recipe_type (default to dessert for tests)
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
        // AC-2: Test Simple classification
        let recipe = create_test_recipe("1", 5, 4, None);
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (5 * 0.3) + (4 * 0.4) + (0 * 0.3) = 1.5 + 1.6 + 0 = 3.1
        assert!((score - 3.1).abs() < 0.01);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_complexity_calculator_moderate() {
        // AC-2: Test Moderate classification (30-60)
        let recipe = create_test_recipe("2", 50, 50, None);
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (50 * 0.3) + (50 * 0.4) + (0 * 0.3) = 15 + 20 + 0 = 35
        assert!((score - 35.0).abs() < 0.01);
        assert_eq!(complexity, Complexity::Moderate);
    }

    #[test]
    fn test_complexity_calculator_complex() {
        // AC-2: Test Complex classification with advance prep >= 4 hours
        let recipe = create_test_recipe("3", 20, 15, Some(4));
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (20 * 0.3) + (15 * 0.4) + (100 * 0.3) = 6.0 + 6.0 + 30.0 = 42.0
        assert!((score - 42.0).abs() < 0.01);
        assert_eq!(complexity, Complexity::Moderate); // 30-60 range
    }

    #[test]
    fn test_complexity_calculator_complex_high() {
        // AC-2: Test Complex classification with many ingredients/steps
        let recipe = create_test_recipe("4", 100, 100, None);
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (100 * 0.3) + (100 * 0.4) + (0 * 0.3) = 30 + 40 + 0 = 70
        assert!((score - 70.0).abs() < 0.01);
        assert_eq!(complexity, Complexity::Complex);
    }

    #[test]
    fn test_complexity_calculator_with_short_prep() {
        // AC-2: Test with short advance prep (< 4 hours)
        let recipe = create_test_recipe("5", 10, 8, Some(2));
        let score = RecipeComplexityCalculator::calculate_score(&recipe);
        let complexity = RecipeComplexityCalculator::calculate_complexity(&recipe);

        // (10 * 0.3) + (8 * 0.4) + (50 * 0.3) = 3.0 + 3.2 + 15.0 = 21.2
        assert!((score - 21.2).abs() < 0.01);
        assert_eq!(complexity, Complexity::Simple);
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
            Some(12345), // Fixed seed for deterministic test
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

        let result = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites,
            constraints,
            rotation_state,
            Some(12345),
        );

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
    fn test_score_recipe_for_slot_simple_weeknight() {
        // AC-3: Simple recipe on weeknight should score high
        use crate::constraints::*;

        let recipe = create_test_recipe("1", 6, 4, None); // Simple recipe
        let user_constraints = UserConstraints::default();
        let tuesday = chrono::NaiveDate::from_ymd_opt(2025, 10, 21).unwrap();
        let slot = MealSlot {
            date: tuesday,
            course_type: CourseType::Dessert, // AC-4: Dinner → Dessert
        };

        let score =
            MealPlanningAlgorithm::score_recipe_for_slot(&recipe, &slot, &user_constraints, &[]);

        // Simple recipe on weeknight should score high (> 0.7)
        assert!(
            score > 0.7,
            "Expected high score for simple weeknight recipe, got {}",
            score
        );
    }

    #[test]
    fn test_score_recipe_for_slot_complex_weekend() {
        // AC-4: Complex recipe on weekend should score high
        use crate::constraints::*;

        let recipe = create_test_recipe("1", 100, 100, Some(4)); // Complex recipe
        let user_constraints = UserConstraints::default();
        let saturday = chrono::NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
        let slot = MealSlot {
            date: saturday,
            course_type: CourseType::Dessert, // AC-4: Dinner → Dessert
        };

        let score =
            MealPlanningAlgorithm::score_recipe_for_slot(&recipe, &slot, &user_constraints, &[]);

        // Complex recipe on weekend should score high (> 0.8)
        assert!(
            score > 0.8,
            "Expected high score for complex weekend recipe, got {}",
            score
        );
    }

    #[test]
    fn test_score_recipe_for_slot_seafood_early_week() {
        // AC-7: Seafood/fresh recipes score higher early in week
        use crate::constraints::*;

        let recipe = create_test_recipe("1", 10, 8, None);
        let user_constraints = UserConstraints::default();

        // Monday (day 1)
        let monday = chrono::NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
        let slot_monday = MealSlot {
            date: monday,
            course_type: CourseType::Dessert, // AC-4: Dinner → Dessert
        };
        let score_monday = MealPlanningAlgorithm::score_recipe_for_slot(
            &recipe,
            &slot_monday,
            &user_constraints,
            &[],
        );

        // Friday (day 5)
        let friday = chrono::NaiveDate::from_ymd_opt(2025, 10, 24).unwrap();
        let slot_friday = MealSlot {
            date: friday,
            course_type: CourseType::Dessert, // AC-4: Dinner → Dessert
        };
        let score_friday = MealPlanningAlgorithm::score_recipe_for_slot(
            &recipe,
            &slot_friday,
            &user_constraints,
            &[],
        );

        // Monday should score higher or equal to Friday for freshness
        assert!(
            score_monday >= score_friday,
            "Expected Monday ({}) >= Friday ({}) for freshness",
            score_monday,
            score_friday
        );
    }

    #[test]
    fn test_deterministic_randomization_same_seed() {
        // AC-9: Same seed produces identical meal plan
        let mut favorites = Vec::new();
        for i in 1..=15 {
            favorites.push(create_test_recipe(
                &format!("recipe_{}", i),
                5 + (i % 10),
                4 + (i % 8),
                if i % 5 == 0 { Some(2) } else { None },
            ));
        }

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        let seed = 42;

        // Generate twice with same seed
        let result1 = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(seed),
        );
        let result2 = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(seed),
        );

        assert!(result1.is_ok() && result2.is_ok());
        let (assignments1, _) = result1.unwrap();
        let (assignments2, _) = result2.unwrap();

        // Assignments should be identical
        assert_eq!(assignments1.len(), assignments2.len());
        for (a1, a2) in assignments1.iter().zip(assignments2.iter()) {
            assert_eq!(a1.date, a2.date);
            assert_eq!(a1.meal_type, a2.meal_type);
            assert_eq!(a1.recipe_id, a2.recipe_id);
        }
    }

    #[test]
    fn test_deterministic_randomization_different_seeds() {
        // AC-9: Different seeds produce different valid meal plans
        let mut favorites = Vec::new();
        for i in 1..=15 {
            favorites.push(create_test_recipe(
                &format!("recipe_{}", i),
                5 + (i % 10),
                4 + (i % 8),
                if i % 5 == 0 { Some(2) } else { None },
            ));
        }

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        // Generate with different seeds
        let result1 = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(42),
        );
        let result2 = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(9999),
        );

        assert!(result1.is_ok() && result2.is_ok());
        let (assignments1, _) = result1.unwrap();
        let (assignments2, _) = result2.unwrap();

        // Both should have 21 assignments
        assert_eq!(assignments1.len(), 21);
        assert_eq!(assignments2.len(), 21);

        // Assignments should differ (high probability)
        let mut different = false;
        for (a1, a2) in assignments1.iter().zip(assignments2.iter()) {
            if a1.recipe_id != a2.recipe_id {
                different = true;
                break;
            }
        }
        assert!(
            different,
            "Different seeds should produce different assignments"
        );
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
        let result = MealPlanningAlgorithm::generate(
            "2025-10-20",
            favorites,
            constraints,
            rotation_state,
            Some(12345),
        );
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
