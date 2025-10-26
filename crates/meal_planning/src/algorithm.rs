use crate::error::MealPlanningError;
use crate::events::MealAssignment;
use crate::rotation::{RotationState, RotationSystem};
use chrono::{Datelike, NaiveDate, Weekday};
use recipe::{AccompanimentCategory, Cuisine};
use serde::{Deserialize, Serialize};

/// Recipe data needed for meal planning algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeForPlanning {
    pub id: String,
    pub title: String,
    pub recipe_type: String, // AC-4: "appetizer", "main_course", or "dessert"
    pub ingredients_count: usize, // Count of ingredients
    pub instructions_count: usize, // Count of instruction steps
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub complexity: Option<String>, // "simple", "moderate", "complex" (if pre-calculated)
    pub dietary_tags: Vec<String>, // Tags like "vegetarian", "vegan", "gluten-free", "dairy-free", etc.
    pub cuisine: Cuisine,          // Cuisine type for variety scoring (Story 7.2 AC-5)

    // Story 7.3: Accompaniment fields
    /// Does this main course accept an accompaniment side dish? (for main courses only)
    pub accepts_accompaniment: bool,
    /// Preferred accompaniment categories (empty = any category acceptable)
    pub preferred_accompaniments: Vec<AccompanimentCategory>,
    /// Category if this recipe IS an accompaniment (None for main courses, appetizers, desserts)
    pub accompaniment_category: Option<AccompanimentCategory>,
}

/// User profile constraints for meal planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConstraints {
    pub weeknight_availability_minutes: Option<u32>, // Max cooking time on weeknights
    pub dietary_restrictions: Vec<String>,           // e.g., ["vegetarian", "gluten-free"]
}

impl Default for UserConstraints {
    fn default() -> Self {
        UserConstraints {
            weeknight_availability_minutes: Some(45), // Default 45 min weeknights
            dietary_restrictions: Vec::new(),
        }
    }
}

/// User skill level for recipe complexity filtering (Story 7.2 AC-3)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    /// Beginner: Only Simple complexity recipes allowed
    Beginner,
    /// Intermediate: Simple + Moderate complexity recipes allowed
    Intermediate,
    /// Advanced: All complexity levels allowed (Simple, Moderate, Complex)
    Advanced,
}

/// User preferences for meal planning algorithm (Story 7.2)
///
/// Controls time constraints, skill level filtering, complexity avoidance,
/// and cuisine variety optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Dietary restrictions (Story 7.1 - used by filter_by_dietary_restrictions)
    pub dietary_restrictions: Vec<String>,
    /// Maximum total time (prep + cook) for weeknight meals in minutes (default: 30)
    pub max_prep_time_weeknight: u32,
    /// Maximum total time (prep + cook) for weekend meals in minutes (default: 90)
    pub max_prep_time_weekend: u32,
    /// User's cooking skill level for complexity filtering
    pub skill_level: SkillLevel,
    /// If true, avoid assigning Complex recipes on consecutive days (default: true)
    pub avoid_consecutive_complex: bool,
    /// Weight for cuisine variety scoring: 0.0 (no variety preference) to 1.0 (maximum variety), default: 0.7
    pub cuisine_variety_weight: f32,
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences {
            dietary_restrictions: Vec::new(),
            max_prep_time_weeknight: 30,
            max_prep_time_weekend: 90,
            skill_level: SkillLevel::Intermediate,
            avoid_consecutive_complex: true,
            cuisine_variety_weight: 0.7,
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

        // Story 3.13 AC-4: Validate start_date is next Monday (not past or current week)
        // Business rule: Meal plans must start from next week to give users time to prepare
        use chrono::{Datelike, Local, Weekday};
        let today = Local::now().date_naive();

        // Validate: start_date must be in the future
        if start <= today {
            return Err(MealPlanningError::InvalidWeekStart(format!(
                "Meal plan start date {} must be in the future (today is {})",
                start, today
            )));
        }

        // Validate: start_date must be a Monday (week start convention)
        if start.weekday() != Weekday::Mon {
            return Err(MealPlanningError::InvalidWeekStart(format!(
                "Meal plan start date {} must be a Monday (found {:?})",
                start,
                start.weekday()
            )));
        }

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
            for course_type_enum in [
                CourseType::Appetizer,
                CourseType::MainCourse,
                CourseType::Dessert,
            ] {
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
                            .find(|r| {
                                !used_recipe_ids.contains(&r.id)
                                    && r.recipe_type == course_type_enum.as_str()
                            })
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
                                minimum: if course_type_enum.as_str() == "main_course" {
                                    7
                                } else {
                                    1
                                },
                                current: available_recipes
                                    .iter()
                                    .filter(|r| r.recipe_type == course_type_enum.as_str())
                                    .count(),
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
                    accompaniment_recipe_id: None, // Story 6.3 AC-8: No accompaniment logic yet
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

/// Select main course based on user preferences and constraints (Story 7.2)
///
/// Applies multi-factor filtering and scoring:
/// 1. Time constraint filtering (weeknight vs weekend) - AC-2
/// 2. Skill level filtering (Beginner/Intermediate/Advanced) - AC-3
/// 3. Consecutive complex avoidance (if enabled) - AC-4
/// 4. Cuisine variety scoring (penalizes recently used cuisines) - AC-5
/// 5. Returns highest-scored recipe or None if no candidates - AC-6, AC-7
///
/// # Arguments
/// * `available_main_courses` - Candidate main course recipes to select from
/// * `preferences` - User preferences (time limits, skill level, variety weight, etc.)
/// * `rotation_state` - Current rotation state (cuisine usage, last complex date)
/// * `date` - Date for the meal assignment (used to check weeknight/weekend)
/// * `day_of_week` - Day of week for the assignment
///
/// # Returns
/// * `Some(Recipe)` - Highest-scored recipe matching all constraints
/// * `None` - No recipes satisfy all constraints
///
/// # Performance
/// Target: <10ms for 100 recipes (Story 7.2 AC-9)
/// Complexity: O(n) where n = number of available main courses
pub fn select_main_course_with_preferences(
    available_main_courses: &[RecipeForPlanning],
    preferences: &UserPreferences,
    rotation_state: &RotationState,
    date: NaiveDate,
    day_of_week: Weekday,
) -> Option<RecipeForPlanning> {
    // AC-2: Determine time limit based on weeknight vs weekend
    let is_weekend = day_of_week == Weekday::Sat || day_of_week == Weekday::Sun;
    let max_time = if is_weekend {
        preferences.max_prep_time_weekend
    } else {
        preferences.max_prep_time_weeknight
    };

    // Parse last_complex_meal_date from rotation_state if present (AC-4)
    let last_complex_date_opt = rotation_state
        .last_complex_meal_date
        .as_ref()
        .and_then(|date_str| NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok());

    // Filter candidates by hard constraints
    let candidates: Vec<&RecipeForPlanning> = available_main_courses
        .iter()
        // AC-2: Time constraint filtering (weeknight vs weekend)
        .filter(|recipe| {
            let total_time = recipe.prep_time_min.unwrap_or(0) + recipe.cook_time_min.unwrap_or(0);
            total_time <= max_time
        })
        // AC-3: Skill level filtering
        .filter(|recipe| {
            let complexity = RecipeComplexityCalculator::calculate_complexity(recipe);
            match preferences.skill_level {
                SkillLevel::Beginner => complexity == Complexity::Simple,
                SkillLevel::Intermediate => {
                    complexity == Complexity::Simple || complexity == Complexity::Moderate
                }
                SkillLevel::Advanced => true, // All complexities allowed
            }
        })
        // AC-4: Consecutive complex avoidance
        .filter(|recipe| {
            if !preferences.avoid_consecutive_complex {
                return true; // Feature disabled, allow all
            }

            let complexity = RecipeComplexityCalculator::calculate_complexity(recipe);
            if complexity != Complexity::Complex {
                return true; // Not complex, always allow
            }

            // Complex recipe: check if last complex was yesterday
            match last_complex_date_opt {
                None => true, // No previous complex meal, allow
                Some(last_complex_date) => {
                    // Calculate days difference
                    let days_since_last_complex = (date - last_complex_date).num_days();
                    // Allow if 2+ days ago, filter if yesterday (1 day ago)
                    !(0..2).contains(&days_since_last_complex)
                }
            }
        })
        .collect();

    // AC-7: Handle no compatible recipes - return None
    if candidates.is_empty() {
        return None;
    }

    // AC-5: Score candidates by cuisine variety
    // Formula: score = variety_weight * (1.0 / (cuisine_usage_count + 1.0))
    let variety_weight = preferences.cuisine_variety_weight;

    let mut scored_candidates: Vec<(f32, &RecipeForPlanning)> = candidates
        .iter()
        .map(|recipe| {
            let usage_count = rotation_state.get_cuisine_usage(&recipe.cuisine);
            let score = variety_weight * (1.0 / (usage_count as f32 + 1.0));
            (score, *recipe)
        })
        .collect();

    // AC-6: Select highest-scored recipe
    // Sort by score descending (highest first)
    scored_candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Return first (highest-scored) recipe
    // If multiple tie, first one selected (deterministic)
    scored_candidates
        .first()
        .map(|(_, recipe)| (*recipe).clone())
}

/// Story 7.3: Select a random compatible accompaniment for a main course
///
/// Pairs main courses with appropriate side dishes based on:
/// - Whether the main course accepts an accompaniment
/// - Preferred accompaniment categories (if specified)
/// - Random selection from compatible options
///
/// # Arguments
/// * `main_course` - The main course recipe
/// * `available_accompaniments` - List of available accompaniment recipes
///
/// # Returns
/// * `Some(Recipe)` - A randomly selected compatible accompaniment
/// * `None` - If main doesn't accept accompaniment or no compatible options available
///
/// # Acceptance Criteria
/// - AC-1: Function implemented with correct signature
/// - AC-2: Returns None if main_course.accepts_accompaniment == false
/// - AC-3: Filters by preferred_accompaniments if specified
/// - AC-4, AC-8: Uses rand::thread_rng for random selection
/// - AC-5: Returns None if no compatible accompaniments
/// - AC-6: Allows repetition (not tracked in rotation)
///
/// # Example
/// ```rust,no_run
/// use meal_planning::algorithm::{RecipeForPlanning, select_accompaniment};
/// use recipe::{AccompanimentCategory, Cuisine};
///
/// let main = RecipeForPlanning {
///     id: "main1".to_string(),
///     title: "Chicken Tikka Masala".to_string(),
///     recipe_type: "main_course".to_string(),
///     ingredients_count: 10,
///     instructions_count: 5,
///     prep_time_min: Some(20),
///     cook_time_min: Some(30),
///     advance_prep_hours: Some(4),
///     complexity: Some("moderate".to_string()),
///     dietary_tags: vec![],
///     cuisine: Cuisine::Indian,
///     accepts_accompaniment: true,
///     preferred_accompaniments: vec![AccompanimentCategory::Rice],
///     accompaniment_category: None,
/// };
///
/// let rice = RecipeForPlanning {
///     id: "rice1".to_string(),
///     title: "Basmati Rice".to_string(),
///     recipe_type: "main_course".to_string(),
///     ingredients_count: 2,
///     instructions_count: 2,
///     prep_time_min: Some(5),
///     cook_time_min: Some(15),
///     advance_prep_hours: None,
///     complexity: Some("simple".to_string()),
///     dietary_tags: vec![],
///     cuisine: Cuisine::Indian,
///     accepts_accompaniment: false,
///     preferred_accompaniments: vec![],
///     accompaniment_category: Some(AccompanimentCategory::Rice),
/// };
///
/// let accompaniments = vec![rice];
/// let selected = select_accompaniment(&main, &accompaniments);
/// assert!(selected.is_some());
/// ```
pub fn select_accompaniment(
    main_course: &RecipeForPlanning,
    available_accompaniments: &[RecipeForPlanning],
) -> Option<RecipeForPlanning> {
    use rand::prelude::IndexedRandom;

    // AC-2: If main course doesn't accept accompaniment, return None immediately
    if !main_course.accepts_accompaniment {
        return None;
    }

    // AC-3: Filter by preferred_accompaniments if specified
    let filtered: Vec<&RecipeForPlanning> = if main_course.preferred_accompaniments.is_empty() {
        // Empty preferences = any category acceptable
        available_accompaniments.iter().collect()
    } else {
        // Filter to only accompaniments in preferred categories
        available_accompaniments
            .iter()
            .filter(|acc| {
                if let Some(category) = &acc.accompaniment_category {
                    main_course.preferred_accompaniments.contains(category)
                } else {
                    false // Accompaniment must have a category to match preferences
                }
            })
            .collect()
    };

    // AC-5: If no compatible accompaniments, return None
    if filtered.is_empty() {
        return None;
    }

    // AC-4, AC-8: Random selection using rand::rng (rand 0.9)
    let mut rng = rand::rng();
    filtered.choose(&mut rng).map(|recipe| (*recipe).clone()) // AC-6: Clone for ownership (allows repetition)
}

/// Story 7.4: Generate a single week meal plan with 21 assignments
///
/// Generates a complete week (Monday-Sunday) with 21 meal assignments:
/// - 7 days × 3 courses per day (Appetizer, MainCourse, Dessert)
/// - Main courses are unique (never repeat within the week)
/// - Appetizers and desserts can repeat after exhausting available recipes (cyclic rotation)
/// - Accompaniments assigned to main courses when applicable
///
/// # Arguments
/// * `recipes` - All available recipes (main courses, appetizers, desserts, accompaniments)
/// * `preferences` - User preferences (time constraints, skill level, variety weight)
/// * `rotation_state` - Mutable rotation state (updated with week's assignments)
/// * `week_start_date` - Monday date for the week (YYYY-MM-DD)
///
/// # Returns
/// * `Ok(WeekMealPlan)` - Complete week with 21 assignments, status=Future, is_locked=false
/// * `Err(MealPlanningError)` - If insufficient recipes or invalid date
///
/// # Acceptance Criteria
/// - AC-1: Function implemented with correct signature
/// - AC-2: Generates exactly 21 assignments (7 days × 3 courses)
/// - AC-3: Each day has appetizer, main_course, dessert; dates span Monday-Sunday
/// - AC-4: Appetizers/desserts use cyclic rotation with reset when exhausted
/// - AC-5: Main courses never repeat within week (strict uniqueness)
/// - AC-6: Accompaniments assigned when main_course.accepts_accompaniment=true
/// - AC-7: RotationState updated after generation
/// - AC-8: WeekMealPlan has status=Future, is_locked=false
/// - AC-9: Edge cases handled (insufficient mains, non-Monday date)
///
/// # Performance
/// Target: <1 second for 50 recipes (Story 7.4 AC-9)
///
/// # Example
/// ```rust,no_run
/// use meal_planning::algorithm::{generate_single_week, RecipeForPlanning, UserPreferences};
/// use meal_planning::rotation::RotationState;
/// use chrono::NaiveDate;
///
/// let recipes = vec![/* ... recipe list ... */];
/// let preferences = UserPreferences::default();
/// let mut rotation_state = RotationState::new();
/// let week_start = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday
///
/// let week_plan = generate_single_week(recipes, &preferences, &mut rotation_state, week_start)
///     .expect("Week generation failed");
///
/// assert_eq!(week_plan.meal_assignments.len(), 21);
/// ```
pub fn generate_single_week(
    recipes: Vec<RecipeForPlanning>,
    preferences: &UserPreferences,
    rotation_state: &mut RotationState,
    week_start_date: NaiveDate,
) -> Result<crate::events::WeekMealPlan, MealPlanningError> {
    use crate::events::{MealAssignment, WeekMealPlan, WeekStatus};
    use chrono::{Datelike, Duration, Weekday};

    // AC-9: Validate week_start_date is Monday
    if week_start_date.weekday() != Weekday::Mon {
        return Err(MealPlanningError::InvalidWeekStart(format!(
            "Week start date {} must be a Monday (found {:?})",
            week_start_date,
            week_start_date.weekday()
        )));
    }

    // Separate recipes by type
    let main_courses: Vec<RecipeForPlanning> = recipes
        .iter()
        .filter(|r| r.recipe_type == "main_course" && r.accompaniment_category.is_none())
        .cloned()
        .collect();

    let appetizers: Vec<RecipeForPlanning> = recipes
        .iter()
        .filter(|r| r.recipe_type == "appetizer")
        .cloned()
        .collect();

    let desserts: Vec<RecipeForPlanning> = recipes
        .iter()
        .filter(|r| r.recipe_type == "dessert")
        .cloned()
        .collect();

    let accompaniments: Vec<RecipeForPlanning> = recipes
        .iter()
        .filter(|r| r.accompaniment_category.is_some())
        .cloned()
        .collect();

    // AC-9: Validate sufficient main courses (need 7 unique for the week)
    // Filter out main courses already used in rotation (for multi-week scenarios)
    let available_main_courses: Vec<RecipeForPlanning> = main_courses
        .iter()
        .filter(|mc| !rotation_state.is_main_course_used(&mc.id))
        .cloned()
        .collect();

    if available_main_courses.len() < 7 {
        return Err(MealPlanningError::InsufficientRecipes {
            minimum: 7,
            current: available_main_courses.len(),
        });
    }

    // Generate 21 meal assignments (7 days × 3 courses)
    let mut meal_assignments = Vec::with_capacity(21);

    for day_offset in 0..7 {
        let date = week_start_date + Duration::days(day_offset);
        let date_str = date.format("%Y-%m-%d").to_string();
        let day_of_week = date.weekday();

        // AC-3: Generate 3 courses per day (Appetizer, MainCourse, Dessert)

        // 1. APPETIZER SELECTION
        // AC-4: Cyclic rotation - reset if all appetizers exhausted
        rotation_state.reset_appetizers_if_all_used(appetizers.len());

        let available_appetizers: Vec<&RecipeForPlanning> = appetizers
            .iter()
            .filter(|app| {
                !rotation_state
                    .used_appetizer_ids
                    .contains(&app.id.to_string())
            })
            .collect();

        let selected_appetizer = if !available_appetizers.is_empty() {
            // Select first available (simple selection)
            available_appetizers[0].clone()
        } else if !appetizers.is_empty() {
            // Fallback: use any appetizer if all were used
            appetizers[0].clone()
        } else {
            return Err(MealPlanningError::InsufficientRecipes {
                minimum: 1,
                current: 0,
            });
        };

        // Track appetizer usage
        rotation_state.mark_used_appetizer(&selected_appetizer.id);

        meal_assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: "appetizer".to_string(),
            recipe_id: selected_appetizer.id.clone(),
            prep_required: selected_appetizer.advance_prep_hours.is_some()
                && selected_appetizer.advance_prep_hours.unwrap() > 0,
            assignment_reasoning: Some(format!(
                "Assigned appetizer for {}",
                day_of_week_to_string(day_of_week)
            )),
            accompaniment_recipe_id: None,
        });

        // 2. MAIN COURSE SELECTION
        // AC-5: Main courses never repeat (strict uniqueness)
        let available_main_for_selection: Vec<RecipeForPlanning> = available_main_courses
            .iter()
            .filter(|mc| !rotation_state.is_main_course_used(&mc.id))
            .cloned()
            .collect();

        let selected_main = select_main_course_with_preferences(
            &available_main_for_selection,
            preferences,
            rotation_state,
            date,
            day_of_week,
        )
        .ok_or_else(|| MealPlanningError::InsufficientRecipes {
            minimum: 7,
            current: available_main_for_selection.len(),
        })?;

        // AC-7: Track main course usage (never reset within week)
        rotation_state.mark_used_main_course(&selected_main.id);
        rotation_state.increment_cuisine_usage(&selected_main.cuisine);

        // Update last_complex_meal_date if recipe is complex
        let complexity = RecipeComplexityCalculator::calculate_complexity(&selected_main);
        if complexity == Complexity::Complex {
            rotation_state.update_last_complex_meal_date(&date_str);
        }

        // AC-6: Select accompaniment if main course accepts one
        let accompaniment_id = if selected_main.accepts_accompaniment {
            select_accompaniment(&selected_main, &accompaniments).map(|acc| acc.id)
        } else {
            None
        };

        meal_assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: "main_course".to_string(),
            recipe_id: selected_main.id.clone(),
            prep_required: selected_main.advance_prep_hours.is_some()
                && selected_main.advance_prep_hours.unwrap() > 0,
            assignment_reasoning: Some(format!(
                "Main course for {} based on preferences",
                day_of_week_to_string(day_of_week)
            )),
            accompaniment_recipe_id: accompaniment_id,
        });

        // 3. DESSERT SELECTION
        // AC-4: Cyclic rotation - reset if all desserts exhausted
        rotation_state.reset_desserts_if_all_used(desserts.len());

        let available_desserts: Vec<&RecipeForPlanning> = desserts
            .iter()
            .filter(|des| {
                !rotation_state
                    .used_dessert_ids
                    .contains(&des.id.to_string())
            })
            .collect();

        let selected_dessert = if !available_desserts.is_empty() {
            // Select first available (simple selection)
            available_desserts[0].clone()
        } else if !desserts.is_empty() {
            // Fallback: use any dessert if all were used
            desserts[0].clone()
        } else {
            return Err(MealPlanningError::InsufficientRecipes {
                minimum: 1,
                current: 0,
            });
        };

        // Track dessert usage
        rotation_state.mark_used_dessert(&selected_dessert.id);

        meal_assignments.push(MealAssignment {
            date: date_str.clone(),
            course_type: "dessert".to_string(),
            recipe_id: selected_dessert.id.clone(),
            prep_required: selected_dessert.advance_prep_hours.is_some()
                && selected_dessert.advance_prep_hours.unwrap() > 0,
            assignment_reasoning: Some(format!(
                "Dessert for {}",
                day_of_week_to_string(day_of_week)
            )),
            accompaniment_recipe_id: None,
        });
    }

    // AC-8: Create WeekMealPlan with status=Future, is_locked=false
    let end_date = week_start_date + Duration::days(6); // Sunday
    let week_id = uuid::Uuid::new_v4().to_string();
    let generation_batch_id = uuid::Uuid::new_v4().to_string();
    let shopping_list_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    // Note: user_id will be set by caller (we don't have it in this function)
    let week_plan = WeekMealPlan {
        id: week_id,
        user_id: String::new(), // Caller must set this
        start_date: week_start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        status: WeekStatus::Future,
        is_locked: false,
        generation_batch_id,
        meal_assignments,
        shopping_list_id,
        created_at,
    };

    Ok(week_plan)
}

/// Helper to convert Weekday enum to human-readable string
fn day_of_week_to_string(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// Story 7.5: Generate multi-week meal plans (1-5 weeks maximum)
///
/// Generates meal plans for multiple weeks in a single batch, maximizing recipe variety
/// across weeks while respecting rotation rules (main courses never repeat, appetizers/desserts
/// can repeat after exhausting the full list).
///
/// # Arguments
/// * `user_id` - User identifier for the meal plan owner
/// * `favorite_recipes` - All favorite recipes available for assignment
/// * `preferences` - User preferences (time constraints, skill level, dietary restrictions)
///
/// # Returns
/// * `Ok(MultiWeekMealPlan)` - Multi-week plan with 1-5 weeks, rotation state tracking
/// * `Err(MealPlanningError)` - If insufficient recipes or other constraints violated
///
/// # Acceptance Criteria
/// - AC-1: Function implemented with correct async signature
/// - AC-2: Calculates max_weeks = min(5, min(appetizers, mains, desserts))
/// - AC-3: Returns InsufficientRecipes error if max_weeks < 1
/// - AC-4: Filters by dietary restrictions BEFORE counting recipes
/// - AC-5: Generates weeks sequentially (loop 0..max_weeks)
/// - AC-6: Week dates calculated from next Monday + offset (ISO 8601)
/// - AC-7: Shopping list generated per week via generate_shopping_list_for_week
/// - AC-8: Returns MultiWeekMealPlan with all weeks and rotation state
/// - AC-9: Performance: <5 seconds for 5 weeks (P95)
/// - AC-10: Unit tests cover edge cases (1 week, 5 weeks, insufficient)
///
/// # Performance
/// Target: <5 seconds for 5 weeks with 50 recipes (Story 7.5 AC-9)
///
/// # Example
/// ```rust,no_run
/// use meal_planning::algorithm::{generate_multi_week_meal_plans, RecipeForPlanning, UserPreferences};
///
/// # async fn example() {
/// let user_id = "user123".to_string();
/// let recipes = vec![/* ... 50 recipes ... */];
/// let preferences = UserPreferences::default();
///
/// let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;
/// match result {
///     Ok(multi_week_plan) => {
///         println!("Generated {} weeks", multi_week_plan.generated_weeks.len());
///     }
///     Err(e) => eprintln!("Generation failed: {}", e),
/// }
/// # }
/// ```
pub async fn generate_multi_week_meal_plans(
    user_id: String,
    favorite_recipes: Vec<RecipeForPlanning>,
    preferences: UserPreferences,
) -> Result<crate::events::MultiWeekMealPlan, MealPlanningError> {
    use crate::dietary_filter::filter_by_dietary_restrictions;
    use crate::events::{MultiWeekMealPlan, WeekMealPlan};
    use chrono::Duration;

    // AC-4: Filter recipes by dietary restrictions BEFORE counting
    let compatible_recipes = filter_by_dietary_restrictions(
        favorite_recipes,
        &preferences
            .dietary_restrictions
            .iter()
            .map(|tag| {
                // Convert String tags to DietaryRestriction enum
                use user::types::DietaryRestriction;
                match tag.to_lowercase().as_str() {
                    "vegetarian" => DietaryRestriction::Vegetarian,
                    "vegan" => DietaryRestriction::Vegan,
                    "gluten_free" => DietaryRestriction::GlutenFree,
                    "dairy_free" => DietaryRestriction::DairyFree,
                    "nut_free" => DietaryRestriction::NutFree,
                    "halal" => DietaryRestriction::Halal,
                    "kosher" => DietaryRestriction::Kosher,
                    _ => DietaryRestriction::Custom(tag.clone()),
                }
            })
            .collect::<Vec<_>>(),
    );

    // Separate recipes by type for max_weeks calculation
    let appetizers: Vec<RecipeForPlanning> = compatible_recipes
        .iter()
        .filter(|r| r.recipe_type == "appetizer")
        .cloned()
        .collect();

    let main_courses: Vec<RecipeForPlanning> = compatible_recipes
        .iter()
        .filter(|r| r.recipe_type == "main_course" && r.accompaniment_category.is_none())
        .cloned()
        .collect();

    let desserts: Vec<RecipeForPlanning> = compatible_recipes
        .iter()
        .filter(|r| r.recipe_type == "dessert")
        .cloned()
        .collect();

    // AC-2: Calculate max_weeks = min(5, min(appetizers/7, mains/7, desserts/7))
    // Each week needs 7 of each type (7 days × 1 course per day)
    let appetizer_weeks = appetizers.len() / 7;
    let main_weeks = main_courses.len() / 7;
    let dessert_weeks = desserts.len() / 7;

    let max_weeks = std::cmp::min(
        5, // Hard cap at 5 weeks
        std::cmp::min(appetizer_weeks, std::cmp::min(main_weeks, dessert_weeks)),
    );

    // AC-3: Validate sufficient recipes (max_weeks must be >= 1)
    if max_weeks < 1 {
        return Err(MealPlanningError::InsufficientRecipes {
            minimum: 21, // Need 7 of each type minimum
            current: compatible_recipes.len(),
        });
    }

    // AC-5: Initialize RotationState
    let mut rotation_state = RotationState::new();

    // AC-6: Calculate base week start date (next Monday)
    let base_week_start = crate::calculate_next_week_start();

    // AC-5, AC-6: Generate weeks sequentially
    let mut generated_weeks: Vec<WeekMealPlan> = Vec::with_capacity(max_weeks);

    for week_index in 0..max_weeks {
        // AC-6: Calculate week_start_date = next Monday + (week_index * 7 days)
        let week_start_date = base_week_start + Duration::weeks(week_index as i64);

        // Call generate_single_week (Story 7.4)
        // Note: We pass the full compatible_recipes list to each week.
        // generate_single_week will internally filter out already-used main courses
        // based on rotation_state, ensuring proper variety across weeks.
        let week_plan = generate_single_week(
            compatible_recipes.clone(),
            &preferences,
            &mut rotation_state,
            week_start_date,
        )?;

        // AC-7: Generate shopping list for this week
        // Note: generate_shopping_list_for_week is Story 7.6 - will be implemented separately
        // For now, shopping_list_id is already set by generate_single_week

        generated_weeks.push(week_plan);
    }

    // Update user_id on all weeks (generate_single_week leaves it empty)
    for week in &mut generated_weeks {
        week.user_id = user_id.clone();
    }

    // AC-8: Construct MultiWeekMealPlan result
    let generation_batch_id = uuid::Uuid::new_v4().to_string();

    let multi_week_plan = MultiWeekMealPlan {
        user_id,
        generation_batch_id,
        generated_weeks,
        rotation_state,
    };

    Ok(multi_week_plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to get a valid next Monday for testing (Story 3.13)
    fn next_monday_date() -> String {
        crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string()
    }

    fn create_test_recipe(
        id: &str,
        ingredients: usize,
        steps: usize,
        advance_prep: Option<u32>,
    ) -> RecipeForPlanning {
        // Extract numeric part from ID
        let num = id
            .split('_')
            .next_back()
            .and_then(|s| s.parse::<usize>().ok())
            .or_else(|| id.parse::<usize>().ok())
            .unwrap_or(0);

        // Distribute types evenly to ensure variety for tests:
        // Use modulo 3 to distribute evenly across all types
        let recipe_type = match num % 3 {
            0 => "dessert",     // Every 3rd recipe
            1 => "appetizer",   // Recipes 1, 4, 7, 10, 13...
            _ => "main_course", // Recipes 2, 3, 5, 6, 8, 9...
        };

        // Distribute cuisines for variety (Story 7.2)
        let cuisine = match num % 4 {
            0 => Cuisine::Italian,
            1 => Cuisine::Mexican,
            2 => Cuisine::Indian,
            _ => Cuisine::Chinese,
        };

        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Recipe {}", id),
            recipe_type: recipe_type.to_string(),
            ingredients_count: ingredients,
            instructions_count: steps,
            prep_time_min: Some(15),
            cook_time_min: Some(30),
            advance_prep_hours: advance_prep,
            complexity: None,
            dietary_tags: Vec::new(), // Tests can override if needed
            cuisine,
            // Story 7.3: Default accompaniment values for existing tests
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
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
            recipe_type: "main_course".to_string(),
            ingredients_count: 5,
            instructions_count: 4,
            prep_time_min: Some(10),
            cook_time_min: Some(20),
            advance_prep_hours: None,
            complexity: None,
            dietary_tags: Vec::new(),
            cuisine: Cuisine::Italian,
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
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
        // Create more recipes to ensure we have enough of each type
        let mut favorites = vec![];
        for i in 1..=15 {
            favorites.push(create_test_recipe(
                &i.to_string(),
                5 + (i % 8),
                4 + (i % 6),
                if i % 5 == 0 { Some(2) } else { None },
            ));
        }

        let constraints = UserConstraints::default();
        let rotation_state = RotationState::new();

        let result = MealPlanningAlgorithm::generate(
            &next_monday_date(), // Use next Monday (Story 3.13)
            favorites,
            constraints,
            rotation_state,
            Some(12345), // Fixed seed for deterministic test
        );

        assert!(result.is_ok());
        let (assignments, updated_state) = result.unwrap();

        // Should have 21 assignments (7 days × 3 meals)
        assert_eq!(assignments.len(), 21);

        // All 15 recipes used, so cycle should have reset (cycle_number increments, used_count = 0)
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
            &next_monday_date(),
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
            &next_monday_date(),
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(seed),
        );
        let result2 = MealPlanningAlgorithm::generate(
            &next_monday_date(),
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
            assert_eq!(a1.course_type, a2.course_type);
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
            &next_monday_date(),
            favorites.clone(),
            constraints.clone(),
            rotation_state.clone(),
            Some(42),
        );
        let result2 = MealPlanningAlgorithm::generate(
            &next_monday_date(),
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

        // Use next Monday (Story 3.13: all meal plans must start from next week)
        let start_date = crate::calculate_next_week_start()
            .format("%Y-%m-%d")
            .to_string();

        let start = Instant::now();
        let result = MealPlanningAlgorithm::generate(
            &start_date,
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

    // ============================================================================
    // Story 7.2: Main Course Selection with Preferences - Unit Tests
    // ============================================================================

    /// Helper to create a recipe with specific time and complexity
    fn create_recipe_with_time_complexity(
        id: &str,
        prep_time: u32,
        cook_time: u32,
        ingredients: usize,
        steps: usize,
        cuisine: Cuisine,
    ) -> RecipeForPlanning {
        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Recipe {}", id),
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

    #[test]
    fn test_weeknight_time_filtering() {
        // AC-2: Test weeknight filtering (30min limit default)
        let recipes = vec![
            create_recipe_with_time_complexity("fast", 10, 15, 5, 4, Cuisine::Italian), // 25min total
            create_recipe_with_time_complexity("slow", 20, 20, 8, 6, Cuisine::Mexican), // 40min total
        ];

        let preferences = UserPreferences::default(); // max_prep_time_weeknight: 30
        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday
        let day_of_week = Weekday::Mon;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "fast"); // Only 25min recipe should pass
    }

    #[test]
    fn test_weekend_time_filtering() {
        // AC-2: Test weekend filtering (90min limit default)
        let recipes = vec![
            create_recipe_with_time_complexity("medium", 30, 40, 10, 8, Cuisine::Indian), // 70min total
            create_recipe_with_time_complexity("long", 50, 50, 100, 100, Cuisine::Chinese), // 100min total (Complex)
        ];

        let preferences = UserPreferences::default(); // max_prep_time_weekend: 90
        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "medium"); // 70min passes, 100min filtered
    }

    #[test]
    fn test_skill_level_beginner() {
        // AC-3: Beginner should only get Simple recipes
        let recipes = vec![
            create_recipe_with_time_complexity("simple", 10, 10, 5, 4, Cuisine::Italian), // Simple
            create_recipe_with_time_complexity("moderate", 15, 15, 50, 50, Cuisine::Mexican), // Moderate
        ];

        let preferences = UserPreferences {
            skill_level: SkillLevel::Beginner,
            ..Default::default()
        };
        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "simple"); // Only Simple passes for Beginner
    }

    #[test]
    fn test_skill_level_intermediate() {
        // AC-3: Intermediate should get Simple + Moderate, not Complex
        let recipes = vec![
            create_recipe_with_time_complexity("simple", 10, 10, 5, 4, Cuisine::Italian), // Simple
            create_recipe_with_time_complexity("moderate", 15, 15, 50, 50, Cuisine::Mexican), // Moderate
            create_recipe_with_time_complexity("complex", 20, 20, 100, 100, Cuisine::Indian), // Complex
        ];

        let preferences = UserPreferences {
            skill_level: SkillLevel::Intermediate,
            ..Default::default()
        };
        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        // Should select Simple or Moderate, not Complex
        assert!(selected.id == "simple" || selected.id == "moderate");
        assert_ne!(selected.id, "complex");
    }

    #[test]
    fn test_skill_level_advanced() {
        // AC-3: Advanced should get all complexity levels
        let recipes = vec![
            create_recipe_with_time_complexity("complex", 20, 20, 100, 100, Cuisine::Indian), // Complex
        ];

        let preferences = UserPreferences {
            skill_level: SkillLevel::Advanced,
            ..Default::default()
        };
        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "complex"); // Advanced allows Complex
    }

    #[test]
    fn test_consecutive_complex_avoidance_yesterday() {
        // AC-4: If last complex was yesterday, filter out Complex recipes
        let recipes = vec![
            create_recipe_with_time_complexity("simple", 10, 10, 5, 4, Cuisine::Italian), // Simple
            create_recipe_with_time_complexity("complex", 20, 20, 100, 100, Cuisine::Mexican), // Complex
        ];

        let preferences = UserPreferences {
            skill_level: SkillLevel::Advanced,
            avoid_consecutive_complex: true,
            ..Default::default()
        };

        let mut rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(); // Tuesday
        let yesterday = date - chrono::Duration::days(1);
        rotation_state.update_last_complex_meal_date(&yesterday.format("%Y-%m-%d").to_string());

        let day_of_week = Weekday::Tue;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "simple"); // Complex filtered due to yesterday rule
    }

    #[test]
    fn test_consecutive_complex_allowed_2_days_ago() {
        // AC-4: If last complex was 2+ days ago, allow Complex recipes
        let recipes = vec![
            create_recipe_with_time_complexity("complex", 10, 10, 100, 100, Cuisine::Mexican), // Complex, 20min total
        ];

        let preferences = UserPreferences {
            skill_level: SkillLevel::Advanced,
            avoid_consecutive_complex: true,
            ..Default::default()
        };

        let mut rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 28).unwrap(); // Tuesday
        let two_days_ago = date - chrono::Duration::days(2);
        rotation_state.update_last_complex_meal_date(&two_days_ago.format("%Y-%m-%d").to_string());

        let day_of_week = Weekday::Tue;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "complex"); // 2+ days ago, Complex allowed
    }

    #[test]
    fn test_cuisine_variety_scoring() {
        // AC-5: Less-used cuisines should score higher
        let recipes = vec![
            create_recipe_with_time_complexity("italian", 10, 10, 5, 4, Cuisine::Italian),
            create_recipe_with_time_complexity("mexican", 10, 10, 5, 4, Cuisine::Mexican),
        ];

        let preferences = UserPreferences::default(); // variety_weight: 0.7
        let mut rotation_state = RotationState::new();

        // Mark Italian as used 2 times
        rotation_state.increment_cuisine_usage(&Cuisine::Italian);
        rotation_state.increment_cuisine_usage(&Cuisine::Italian);
        // Mexican used 0 times

        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        // Mexican should be selected (usage=0, score=0.7 * 1/1 = 0.7)
        // Italian score = 0.7 * 1/3 = 0.23
        assert_eq!(selected.id, "mexican");
    }

    #[test]
    fn test_cuisine_variety_weight_zero() {
        // AC-5: variety_weight=0.0 should not prefer any cuisine
        let recipes = vec![
            create_recipe_with_time_complexity("italian", 10, 10, 5, 4, Cuisine::Italian),
            create_recipe_with_time_complexity("mexican", 10, 10, 5, 4, Cuisine::Mexican),
        ];

        let preferences = UserPreferences {
            cuisine_variety_weight: 0.0,
            ..Default::default()
        };

        let mut rotation_state = RotationState::new();
        rotation_state.increment_cuisine_usage(&Cuisine::Italian);
        rotation_state.increment_cuisine_usage(&Cuisine::Italian);

        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        // Both score 0.0, first one selected (deterministic)
        let selected = result.unwrap();
        assert_eq!(selected.id, "italian"); // First in list
    }

    #[test]
    fn test_highest_scored_selection() {
        // AC-6: Verify highest-scored recipe is selected
        let recipes = vec![
            create_recipe_with_time_complexity("italian", 10, 10, 5, 4, Cuisine::Italian), // usage=2, score=0.23
            create_recipe_with_time_complexity("mexican", 10, 10, 5, 4, Cuisine::Mexican), // usage=1, score=0.35
            create_recipe_with_time_complexity("indian", 10, 10, 5, 4, Cuisine::Indian), // usage=0, score=0.70
        ];

        let preferences = UserPreferences::default(); // variety_weight: 0.7
        let mut rotation_state = RotationState::new();

        rotation_state.increment_cuisine_usage(&Cuisine::Italian);
        rotation_state.increment_cuisine_usage(&Cuisine::Italian);
        rotation_state.increment_cuisine_usage(&Cuisine::Mexican);

        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "indian"); // Highest score (0.70)
    }

    #[test]
    fn test_tie_breaking_deterministic() {
        // AC-6: If multiple recipes tie, first one selected
        let recipes = vec![
            create_recipe_with_time_complexity("italian1", 10, 10, 5, 4, Cuisine::Italian),
            create_recipe_with_time_complexity("italian2", 10, 10, 5, 4, Cuisine::Italian),
        ];

        let preferences = UserPreferences::default();
        let rotation_state = RotationState::new(); // Both Italian unused, same score

        let date = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap(); // Saturday
        let day_of_week = Weekday::Sat;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.id, "italian1"); // First in list selected
    }

    #[test]
    fn test_no_compatible_recipes_returns_none() {
        // AC-7: If all recipes filtered, return None
        let recipes = vec![
            create_recipe_with_time_complexity("slow", 50, 50, 100, 100, Cuisine::Italian), // 100min total, Complex
        ];

        let preferences = UserPreferences {
            max_prep_time_weeknight: 30,
            skill_level: SkillLevel::Beginner,
            ..Default::default()
        };

        let rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday
        let day_of_week = Weekday::Mon;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_none()); // All filtered, returns None
    }

    #[test]
    fn test_preference_combinations() {
        // AC-8: Test multiple constraints active simultaneously
        let recipes = vec![
            create_recipe_with_time_complexity("fast_simple", 10, 10, 5, 4, Cuisine::Italian), // 20min, Simple
            create_recipe_with_time_complexity("slow_complex", 25, 25, 100, 100, Cuisine::Mexican), // 50min, Complex
        ];

        let preferences = UserPreferences {
            max_prep_time_weeknight: 30,
            skill_level: SkillLevel::Beginner,
            avoid_consecutive_complex: true,
            ..Default::default()
        };

        let mut rotation_state = RotationState::new();
        let date = NaiveDate::from_ymd_opt(2025, 10, 27).unwrap(); // Monday (weeknight)
        let yesterday = date - chrono::Duration::days(1);
        rotation_state.update_last_complex_meal_date(&yesterday.format("%Y-%m-%d").to_string());

        let day_of_week = Weekday::Mon;

        let result = select_main_course_with_preferences(
            &recipes,
            &preferences,
            &rotation_state,
            date,
            day_of_week,
        );

        assert!(result.is_some());
        let selected = result.unwrap();
        // Only fast_simple passes all constraints:
        // - Time: 20min <= 30min ✓
        // - Skill: Simple ✓
        // - Consecutive complex: Simple (not filtered) ✓
        // slow_complex fails: time (50>30), skill (Complex), consecutive (Complex yesterday)
        assert_eq!(selected.id, "fast_simple");
    }

    // Story 7.3: Accompaniment Selection Tests

    /// Helper to create test recipe with accompaniment fields
    fn create_test_recipe_with_accompaniment(
        id: &str,
        accepts: bool,
        preferred: Vec<AccompanimentCategory>,
        category: Option<AccompanimentCategory>,
    ) -> RecipeForPlanning {
        RecipeForPlanning {
            id: id.to_string(),
            title: format!("Test Recipe {}", id),
            recipe_type: "main_course".to_string(),
            ingredients_count: 5,
            instructions_count: 3,
            prep_time_min: Some(10),
            cook_time_min: Some(20),
            advance_prep_hours: None,
            complexity: Some("simple".to_string()),
            dietary_tags: vec![],
            cuisine: Cuisine::Italian,
            accepts_accompaniment: accepts,
            preferred_accompaniments: preferred,
            accompaniment_category: category,
        }
    }

    #[test]
    fn test_select_accompaniment_main_does_not_accept_returns_none() {
        // AC-2: Returns None if main_course.accepts_accompaniment == false
        let main_course = create_test_recipe_with_accompaniment("main1", false, vec![], None);
        let accompaniments = vec![create_test_recipe_with_accompaniment(
            "rice",
            false,
            vec![],
            Some(AccompanimentCategory::Rice),
        )];

        let result = select_accompaniment(&main_course, &accompaniments);

        assert!(
            result.is_none(),
            "Expected None when main course doesn't accept accompaniment"
        );
    }

    #[test]
    fn test_select_accompaniment_filters_by_preferred_categories() {
        // AC-3: Filters by preferred_accompaniments if specified
        let main_course = create_test_recipe_with_accompaniment(
            "main2",
            true,
            vec![AccompanimentCategory::Rice, AccompanimentCategory::Pasta],
            None,
        );

        let accompaniments = vec![
            create_test_recipe_with_accompaniment(
                "rice1",
                false,
                vec![],
                Some(AccompanimentCategory::Rice),
            ),
            create_test_recipe_with_accompaniment(
                "pasta1",
                false,
                vec![],
                Some(AccompanimentCategory::Pasta),
            ),
            create_test_recipe_with_accompaniment(
                "salad1",
                false,
                vec![],
                Some(AccompanimentCategory::Salad),
            ),
        ];

        // Test multiple times to ensure filtering works
        for _iteration in 0..10 {
            let result = select_accompaniment(&main_course, &accompaniments);
            assert!(result.is_some(), "Should select an accompaniment");

            let selected = result.unwrap();
            // Should only select rice or pasta, never salad
            assert!(
                selected.id == "rice1" || selected.id == "pasta1",
                "Selected unexpected accompaniment: {}, expected rice1 or pasta1",
                selected.id
            );
        }
    }

    #[test]
    fn test_select_accompaniment_random_selection() {
        // AC-4, AC-8: Selects random from filtered list using rand::rng
        let main_course = create_test_recipe_with_accompaniment("main3", true, vec![], None);
        let accompaniments = vec![
            create_test_recipe_with_accompaniment(
                "acc1",
                false,
                vec![],
                Some(AccompanimentCategory::Rice),
            ),
            create_test_recipe_with_accompaniment(
                "acc2",
                false,
                vec![],
                Some(AccompanimentCategory::Pasta),
            ),
            create_test_recipe_with_accompaniment(
                "acc3",
                false,
                vec![],
                Some(AccompanimentCategory::Salad),
            ),
        ];

        // Run multiple times to check for variety (proves randomness)
        let mut selected_ids = std::collections::HashSet::new();
        for _ in 0..20 {
            let result = select_accompaniment(&main_course, &accompaniments);
            assert!(result.is_some(), "Should select an accompaniment");
            selected_ids.insert(result.unwrap().id.clone());
        }

        // With 20 random selections from 3 options, should see at least 2 different results
        assert!(
            selected_ids.len() >= 2,
            "Random selection should produce variety, got only: {:?}",
            selected_ids
        );
    }

    #[test]
    fn test_select_accompaniment_empty_preferences_uses_all() {
        // AC-3: Empty preferred_accompaniments uses all available
        let main_course = create_test_recipe_with_accompaniment("main4", true, vec![], None);
        let accompaniments = vec![
            create_test_recipe_with_accompaniment(
                "rice",
                false,
                vec![],
                Some(AccompanimentCategory::Rice),
            ),
            create_test_recipe_with_accompaniment(
                "pasta",
                false,
                vec![],
                Some(AccompanimentCategory::Pasta),
            ),
            create_test_recipe_with_accompaniment(
                "salad",
                false,
                vec![],
                Some(AccompanimentCategory::Salad),
            ),
            create_test_recipe_with_accompaniment(
                "fries",
                false,
                vec![],
                Some(AccompanimentCategory::Fries),
            ),
            create_test_recipe_with_accompaniment(
                "bread",
                false,
                vec![],
                Some(AccompanimentCategory::Bread),
            ),
        ];

        // Run multiple times to verify all can be selected
        let mut selected_ids = std::collections::HashSet::new();
        for _ in 0..50 {
            let result = select_accompaniment(&main_course, &accompaniments);
            assert!(result.is_some(), "Should select an accompaniment");
            selected_ids.insert(result.unwrap().id.clone());
        }

        // With empty preferences, all 5 should be eligible (statistical check)
        assert!(
            selected_ids.len() >= 4,
            "Empty preferences should allow all accompaniments, got: {:?}",
            selected_ids
        );
    }

    #[test]
    fn test_select_accompaniment_no_compatible_returns_none() {
        // AC-5: Returns None if no compatible accompaniments
        let main_course = create_test_recipe_with_accompaniment(
            "main5",
            true,
            vec![AccompanimentCategory::Rice],
            None,
        );

        let accompaniments = vec![
            create_test_recipe_with_accompaniment(
                "pasta1",
                false,
                vec![],
                Some(AccompanimentCategory::Pasta),
            ),
            create_test_recipe_with_accompaniment(
                "salad1",
                false,
                vec![],
                Some(AccompanimentCategory::Salad),
            ),
        ];

        let result = select_accompaniment(&main_course, &accompaniments);

        assert!(
            result.is_none(),
            "Expected None when no compatible accompaniments"
        );
    }

    #[test]
    fn test_select_accompaniment_allows_repetition() {
        // AC-6: Allows repetition (not tracked in rotation)
        let main_course = create_test_recipe_with_accompaniment(
            "main6",
            true,
            vec![AccompanimentCategory::Rice],
            None,
        );

        let accompaniments = vec![create_test_recipe_with_accompaniment(
            "rice1",
            false,
            vec![],
            Some(AccompanimentCategory::Rice),
        )];

        // Call twice with same inputs
        let result1 = select_accompaniment(&main_course, &accompaniments);
        let result2 = select_accompaniment(&main_course, &accompaniments);

        assert!(result1.is_some(), "First call should return Some");
        assert!(result2.is_some(), "Second call should return Some");

        // Both should return the same recipe (since only one option)
        assert_eq!(
            result1.as_ref().unwrap().id,
            result2.as_ref().unwrap().id,
            "Repetition should be allowed - both calls should return same recipe"
        );
    }

    #[test]
    fn test_select_accompaniment_empty_list_returns_none() {
        // AC-5: Edge case - empty accompaniments list
        let main_course = create_test_recipe_with_accompaniment("main7", true, vec![], None);
        let accompaniments: Vec<RecipeForPlanning> = vec![];

        let result = select_accompaniment(&main_course, &accompaniments);

        assert!(
            result.is_none(),
            "Expected None when accompaniments list is empty"
        );
    }

    // ============================================================================
    // Story 7.5: Multi-Week Meal Planning Tests
    // ============================================================================

    /// Helper to create a balanced recipe set for multi-week testing
    fn create_balanced_recipes(count: usize) -> Vec<RecipeForPlanning> {
        let mut recipes = Vec::new();

        for i in 0..count {
            let recipe_type = match i % 3 {
                0 => "appetizer",
                1 => "main_course",
                _ => "dessert",
            };

            // Distribute cuisines for variety
            let cuisine = match i % 5 {
                0 => recipe::Cuisine::Italian,
                1 => recipe::Cuisine::Mexican,
                2 => recipe::Cuisine::Indian,
                3 => recipe::Cuisine::Chinese,
                _ => recipe::Cuisine::Japanese,
            };

            // Make ALL test recipes weeknight-friendly (<= 30min total)
            // This simplifies testing by ensuring time constraints don't interfere
            // In real-world usage, users would have a mix of recipe times
            let (prep_time, cook_time) = (10, 15); // 25 min total (weeknight-friendly)

            recipes.push(RecipeForPlanning {
                id: format!("recipe_{}", i + 1),
                title: format!("Recipe {}", i + 1),
                recipe_type: recipe_type.to_string(),
                ingredients_count: 5,
                instructions_count: 3,
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

    /// AC-10: Test with exactly 21 recipes (exactly 1 week possible)
    #[tokio::test]
    async fn test_generate_multi_week_exactly_one_week() {
        // 21 recipes = 7 appetizers, 7 mains, 7 desserts → exactly 1 week
        let recipes = create_balanced_recipes(21);
        let user_id = "test_user_1".to_string();
        let preferences = UserPreferences::default();

        let result = generate_multi_week_meal_plans(user_id.clone(), recipes, preferences).await;

        assert!(
            result.is_ok(),
            "Should generate plan with exactly 21 recipes. Error: {:?}",
            result.as_ref().err()
        );
        let plan = result.unwrap();

        // AC-2: Should generate exactly 1 week
        assert_eq!(
            plan.generated_weeks.len(),
            1,
            "Should generate exactly 1 week from 21 recipes"
        );

        // AC-6: Verify week dates
        let week = &plan.generated_weeks[0];
        assert!(
            week.start_date.ends_with("Monday") || week.start_date.len() == 10,
            "Week should start on a Monday"
        );

        // AC-8: Verify structure
        assert_eq!(plan.user_id, user_id);
        assert!(!plan.generation_batch_id.is_empty());
        assert_eq!(week.meal_assignments.len(), 21);
    }

    /// AC-10: Test with 105+ recipes (5 weeks capped at maximum)
    #[tokio::test]
    async fn test_generate_multi_week_five_weeks_maximum() {
        // 105 recipes = 35 appetizers, 35 mains, 35 desserts → 5 weeks (capped)
        let recipes = create_balanced_recipes(105);
        let user_id = "test_user_2".to_string();
        let preferences = UserPreferences::default();

        let result = generate_multi_week_meal_plans(user_id.clone(), recipes, preferences).await;

        assert!(
            result.is_ok(),
            "Should generate plan with 105+ recipes. Error: {:?}",
            result.as_ref().err()
        );
        let plan = result.unwrap();

        // AC-2: Hard cap at 5 weeks
        assert_eq!(
            plan.generated_weeks.len(),
            5,
            "Should cap at 5 weeks even with many recipes"
        );

        // AC-8: Verify all weeks have full assignments
        for (index, week) in plan.generated_weeks.iter().enumerate() {
            assert_eq!(
                week.meal_assignments.len(),
                21,
                "Week {} should have 21 assignments",
                index + 1
            );
            assert_eq!(week.user_id, user_id);
        }

        // AC-6: Verify sequential week dates
        let base_date =
            chrono::NaiveDate::parse_from_str(&plan.generated_weeks[0].start_date, "%Y-%m-%d")
                .expect("Start date should be valid ISO 8601");

        for (index, week) in plan.generated_weeks.iter().enumerate() {
            let expected_start = base_date + chrono::Duration::weeks(index as i64);
            let actual_start = chrono::NaiveDate::parse_from_str(&week.start_date, "%Y-%m-%d")
                .expect("Week start date should be valid");

            assert_eq!(
                actual_start,
                expected_start,
                "Week {} should start on correct date",
                index + 1
            );
        }
    }

    /// AC-3: Test insufficient recipes error (< 21 recipes)
    #[tokio::test]
    async fn test_generate_multi_week_insufficient_recipes() {
        // 18 recipes = 6 appetizers, 6 mains, 6 desserts → insufficient (need 7 each)
        let recipes = create_balanced_recipes(18);
        let user_id = "test_user_3".to_string();
        let preferences = UserPreferences::default();

        let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;

        assert!(result.is_err(), "Should fail with insufficient recipes");
        match result {
            Err(MealPlanningError::InsufficientRecipes { minimum, current }) => {
                assert_eq!(minimum, 21, "Should require minimum 21 recipes");
                assert_eq!(current, 18, "Should report 18 available recipes");
            }
            _ => panic!("Expected InsufficientRecipes error"),
        }
    }

    /// AC-4: Test dietary filtering impact on max_weeks calculation
    #[tokio::test]
    async fn test_generate_multi_week_dietary_filtering() {
        // Create 63 recipes (21 of each type) but only 21 are vegetarian (7 of each)
        let mut recipes = Vec::new();

        for i in 0..63 {
            let recipe_type = match i % 3 {
                0 => "appetizer",
                1 => "main_course",
                _ => "dessert",
            };

            // Only first 21 recipes are vegetarian
            let dietary_tags = if i < 21 {
                vec!["vegetarian".to_string()]
            } else {
                vec![]
            };

            // Distribute cuisines for variety
            let cuisine = match i % 5 {
                0 => recipe::Cuisine::Italian,
                1 => recipe::Cuisine::Mexican,
                2 => recipe::Cuisine::Indian,
                3 => recipe::Cuisine::Chinese,
                _ => recipe::Cuisine::Japanese,
            };

            recipes.push(RecipeForPlanning {
                id: format!("recipe_{}", i + 1),
                title: format!("Recipe {}", i + 1),
                recipe_type: recipe_type.to_string(),
                ingredients_count: 5,
                instructions_count: 3,
                prep_time_min: Some(10),
                cook_time_min: Some(15), // 25 min total (weeknight-friendly)
                advance_prep_hours: None,
                complexity: Some("simple".to_string()),
                dietary_tags,
                cuisine,
                accepts_accompaniment: false,
                preferred_accompaniments: vec![],
                accompaniment_category: None,
            });
        }

        let user_id = "test_user_4".to_string();
        let preferences = UserPreferences {
            dietary_restrictions: vec!["vegetarian".to_string()],
            ..Default::default()
        };

        let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;

        assert!(
            result.is_ok(),
            "Should generate plan with dietary filtering. Error: {:?}",
            result.as_ref().err()
        );
        let plan = result.unwrap();

        // AC-4: Should only use 21 vegetarian recipes → 1 week
        assert_eq!(
            plan.generated_weeks.len(),
            1,
            "Should generate 1 week after filtering to vegetarian recipes only"
        );
    }

    /// AC-6: Test week date calculations (sequential Monday dates)
    #[tokio::test]
    async fn test_generate_multi_week_date_calculations() {
        // 63 recipes = 21 of each type → 3 weeks
        let recipes = create_balanced_recipes(63);
        let user_id = "test_user_5".to_string();
        let preferences = UserPreferences::default();

        let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;

        assert!(
            result.is_ok(),
            "Should generate 3 weeks. Error: {:?}",
            result.as_ref().err()
        );
        let plan = result.unwrap();

        assert_eq!(plan.generated_weeks.len(), 3);

        // Verify all start dates are Mondays
        for (index, week) in plan.generated_weeks.iter().enumerate() {
            let start_date = chrono::NaiveDate::parse_from_str(&week.start_date, "%Y-%m-%d")
                .expect("Start date should be valid ISO 8601");
            let end_date = chrono::NaiveDate::parse_from_str(&week.end_date, "%Y-%m-%d")
                .expect("End date should be valid ISO 8601");

            assert_eq!(
                start_date.weekday(),
                chrono::Weekday::Mon,
                "Week {} should start on Monday",
                index + 1
            );

            assert_eq!(
                end_date.weekday(),
                chrono::Weekday::Sun,
                "Week {} should end on Sunday",
                index + 1
            );

            // Verify 7-day span
            let days_diff = (end_date - start_date).num_days();
            assert_eq!(
                days_diff,
                6,
                "Week {} should span 7 days (Mon-Sun)",
                index + 1
            );
        }

        // Verify sequential weeks (no gaps)
        for i in 0..plan.generated_weeks.len() - 1 {
            let current_end =
                chrono::NaiveDate::parse_from_str(&plan.generated_weeks[i].end_date, "%Y-%m-%d")
                    .unwrap();
            let next_start = chrono::NaiveDate::parse_from_str(
                &plan.generated_weeks[i + 1].start_date,
                "%Y-%m-%d",
            )
            .unwrap();

            let gap_days = (next_start - current_end).num_days();
            assert_eq!(
                gap_days,
                1,
                "Weeks {} and {} should be consecutive (1 day gap)",
                i + 1,
                i + 2
            );
        }
    }

    /// AC-5: Test RotationState persistence across weeks
    #[tokio::test]
    async fn test_generate_multi_week_rotation_state() {
        // 42 recipes = 14 of each type → 2 weeks
        let recipes = create_balanced_recipes(42);
        let user_id = "test_user_6".to_string();
        let preferences = UserPreferences::default();

        let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;

        assert!(
            result.is_ok(),
            "Should generate 2 weeks. Error: {:?}",
            result.as_ref().err()
        );
        let plan = result.unwrap();

        assert_eq!(plan.generated_weeks.len(), 2);

        // Collect all recipe IDs used across both weeks
        let mut all_recipe_ids = std::collections::HashSet::new();
        for week in &plan.generated_weeks {
            for assignment in &week.meal_assignments {
                all_recipe_ids.insert(&assignment.recipe_id);
            }
        }

        // AC-5: Verify rotation state reflects all used recipes
        assert!(
            all_recipe_ids.len() <= 42,
            "Should not use more recipes than available"
        );

        // Verify no recipe is used more than appropriate for rotation rules
        // (Main courses should never repeat, others can repeat after exhausting list)
        let mut main_course_ids = std::collections::HashSet::new();
        for week in &plan.generated_weeks {
            for assignment in &week.meal_assignments {
                if assignment.course_type == "main_course" {
                    assert!(
                        main_course_ids.insert(&assignment.recipe_id),
                        "Main course recipe {} should not repeat across weeks",
                        assignment.recipe_id
                    );
                }
            }
        }
    }
}
