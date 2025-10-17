use crate::algorithm::{
    Complexity, RecipeComplexityCalculator, RecipeForPlanning, UserConstraints,
};
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};

/// Meal slot represents a specific meal on a specific date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealSlot {
    pub date: NaiveDate,
    pub meal_type: MealType,
}

/// Meal type enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

impl MealType {
    pub fn as_str(&self) -> &str {
        match self {
            MealType::Breakfast => "breakfast",
            MealType::Lunch => "lunch",
            MealType::Dinner => "dinner",
        }
    }
}

impl MealSlot {
    pub fn is_weekend(&self) -> bool {
        let weekday = self.date.weekday();
        weekday == Weekday::Sat || weekday == Weekday::Sun
    }

    pub fn day_of_week(&self) -> u32 {
        // 1 = Monday, 7 = Sunday
        self.date.weekday().num_days_from_monday() + 1
    }
}

/// Day assignment used for tracking equipment conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayAssignment {
    pub date: NaiveDate,
    pub meal_type: MealType,
    pub recipe_id: String,
}

/// Constraint trait for evaluating recipe-to-slot fit
pub trait Constraint {
    /// Evaluate how well a recipe fits a given meal slot
    ///
    /// Returns a score from 0.0 (poor fit) to 1.0 (excellent fit)
    fn evaluate(
        &self,
        recipe: &RecipeForPlanning,
        slot: &MealSlot,
        user_constraints: &UserConstraints,
    ) -> f32;
}

/// AvailabilityConstraint matches recipe cooking time to user's weeknight availability
///
/// AC-3, AC-4: Complex recipes on high-availability days, simple recipes on busy weeknights
pub struct AvailabilityConstraint;

impl Constraint for AvailabilityConstraint {
    fn evaluate(
        &self,
        recipe: &RecipeForPlanning,
        slot: &MealSlot,
        user_constraints: &UserConstraints,
    ) -> f32 {
        let total_time = recipe.prep_time_min.unwrap_or(0) + recipe.cook_time_min.unwrap_or(0);

        // Weekends allow any time commitment
        if slot.is_weekend() {
            return 1.0;
        }

        // Weeknights: check against user's availability
        match user_constraints.weeknight_availability_minutes {
            Some(max_minutes) => {
                if total_time <= max_minutes {
                    // Recipe fits within availability - score based on how well it fits
                    // Recipe using full time slot scores 0.7, shorter recipes score higher
                    let time_ratio = total_time as f32 / max_minutes as f32;
                    (1.0 - time_ratio * 0.3).max(0.7)
                } else {
                    // Recipe exceeds availability - penalize based on how much over
                    let over_ratio = (total_time - max_minutes) as f32 / max_minutes as f32;
                    (0.3 - over_ratio * 0.3).max(0.0)
                }
            }
            None => 1.0, // No constraint specified, all recipes acceptable
        }
    }
}

/// ComplexityConstraint matches recipe complexity to day type (weekday vs weekend)
///
/// AC-2, AC-3, AC-4: Complex recipes on weekends, simple recipes on weeknights
pub struct ComplexityConstraint;

impl Constraint for ComplexityConstraint {
    fn evaluate(
        &self,
        recipe: &RecipeForPlanning,
        slot: &MealSlot,
        _user_constraints: &UserConstraints,
    ) -> f32 {
        let complexity = RecipeComplexityCalculator::calculate_complexity(recipe);

        if slot.is_weekend() {
            // Weekends prefer complex recipes
            match complexity {
                Complexity::Complex => 1.0,
                Complexity::Moderate => 0.85,
                Complexity::Simple => 0.7,
            }
        } else {
            // Weeknights prefer simple recipes
            match complexity {
                Complexity::Simple => 1.0,
                Complexity::Moderate => 0.75,
                Complexity::Complex => 0.3, // Strong penalty for complex weeknight meals
            }
        }
    }
}

/// AdvancePrepConstraint handles recipes requiring advance preparation
///
/// AC-5: Recipes with advance prep scheduled to allow proper lead time
pub struct AdvancePrepConstraint;

impl Constraint for AdvancePrepConstraint {
    fn evaluate(
        &self,
        recipe: &RecipeForPlanning,
        slot: &MealSlot,
        _user_constraints: &UserConstraints,
    ) -> f32 {
        match recipe.advance_prep_hours {
            None | Some(0) => 1.0, // No advance prep required - always fits
            Some(hours) => {
                // Recipes with advance prep need sufficient lead time
                // Prefer scheduling them on days 2-7 (not Monday, to allow prep time)
                let day_of_week = slot.day_of_week();

                if hours < 4 {
                    // Short prep (< 4 hours): can do same-day prep for dinner
                    1.0
                } else if hours < 24 {
                    // Medium prep (4-24 hours): need prep day before
                    if day_of_week >= 2 {
                        0.9 // Good fit - can prep night before
                    } else {
                        0.5 // Monday is less ideal (need weekend prep)
                    }
                } else {
                    // Long prep (24+ hours): need weekend prep
                    if day_of_week >= 3 {
                        0.8 // Mid-week or later gives time for weekend prep
                    } else {
                        0.6 // Early week is less ideal
                    }
                }
            }
        }
    }
}

/// DietaryConstraint matches recipe dietary tags to user restrictions
///
/// AC-6: Recipe dietary tags matched against user dietary restrictions
/// MVP: Placeholder implementation (requires recipe tags to be added)
pub struct DietaryConstraint;

impl Constraint for DietaryConstraint {
    fn evaluate(
        &self,
        _recipe: &RecipeForPlanning,
        _slot: &MealSlot,
        user_constraints: &UserConstraints,
    ) -> f32 {
        // MVP: Return 1.0 (no filtering)
        // Future: Check recipe tags against user_constraints.dietary_restrictions
        // If recipe contains restricted ingredient, return 0.0 (hard constraint violation)
        // Otherwise return 1.0

        if user_constraints.dietary_restrictions.is_empty() {
            1.0
        } else {
            // For now, assume all recipes are compatible
            // TODO: Add dietary tags to RecipeForPlanning and implement filtering
            1.0
        }
    }
}

/// FreshnessConstraint considers ingredient freshness priority
///
/// AC-7: Ingredient freshness considered (produce-heavy meals earlier in week)
/// MVP: Simplified implementation favoring early-week slots
pub struct FreshnessConstraint;

impl Constraint for FreshnessConstraint {
    fn evaluate(
        &self,
        _recipe: &RecipeForPlanning,
        slot: &MealSlot,
        _user_constraints: &UserConstraints,
    ) -> f32 {
        // MVP: Apply simple freshness heuristic
        // Early week (days 1-3): prefer fresh ingredients (higher score)
        // Mid-week (days 4-5): neutral
        // Late week (days 6-7): lower score for freshness-sensitive recipes

        let day_of_week = slot.day_of_week();

        match day_of_week {
            1..=3 => 1.0,  // Early week: best for fresh ingredients
            4..=5 => 0.85, // Mid-week: still good
            6..=7 => 0.75, // Weekend: less ideal for freshness
            _ => 0.75,
        }

        // Future enhancement: Analyze recipe ingredients to determine freshness category
        // High priority (seafood, leafy greens): boost score for days 1-3
        // Medium priority (produce, dairy): boost score for days 1-5
        // Low priority (pantry, frozen): neutral across all days
    }
}

/// EquipmentConflictConstraint avoids back-to-back equipment conflicts
///
/// AC-8: Equipment conflicts avoided (no two oven-dependent meals back-to-back)
pub struct EquipmentConflictConstraint {
    /// Existing assignments for the day (to check for equipment conflicts)
    day_assignments: Vec<DayAssignment>,
}

impl EquipmentConflictConstraint {
    pub fn new(day_assignments: Vec<DayAssignment>) -> Self {
        Self { day_assignments }
    }

    /// Infer equipment type from recipe (MVP: placeholder)
    fn infer_equipment(_recipe: &RecipeForPlanning) -> Vec<Equipment> {
        // Future: Parse recipe instructions for keywords
        // "bake", "roast" → Oven
        // "slow cook", "crockpot" → SlowCooker
        // "simmer", "sauté" → Stovetop
        // "grill", "bbq" → Grill

        // MVP: Assume all recipes use stovetop (allows multiple per day)
        vec![Equipment::Stovetop]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum Equipment {
    Oven,
    SlowCooker,
    Stovetop,
    Grill,
}

impl Constraint for EquipmentConflictConstraint {
    fn evaluate(
        &self,
        recipe: &RecipeForPlanning,
        slot: &MealSlot,
        _user_constraints: &UserConstraints,
    ) -> f32 {
        // Check if any meals already assigned for this day use conflicting equipment
        let _recipe_equipment = Self::infer_equipment(recipe);

        let same_day_assignments: Vec<&DayAssignment> = self
            .day_assignments
            .iter()
            .filter(|a| a.date == slot.date)
            .collect();

        if same_day_assignments.is_empty() {
            return 1.0; // No conflicts
        }

        // MVP: All recipes use stovetop, so no conflicts
        // Future: Check for oven/slow cooker conflicts (only one allowed per day)
        1.0

        // Future logic:
        // if recipe_equipment.contains(&Equipment::Oven) {
        //     let has_oven_conflict = same_day_assignments.iter().any(|a| {
        //         Self::infer_equipment(&load_recipe(a.recipe_id)).contains(&Equipment::Oven)
        //     });
        //     if has_oven_conflict {
        //         return 0.0; // Hard conflict
        //     }
        // }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
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
        }
    }

    #[test]
    fn test_meal_slot_is_weekend() {
        let saturday = NaiveDate::from_ymd_opt(2025, 10, 25).unwrap();
        let slot = MealSlot {
            date: saturday,
            meal_type: MealType::Dinner,
        };
        assert!(slot.is_weekend());

        let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
        let slot = MealSlot {
            date: monday,
            meal_type: MealType::Dinner,
        };
        assert!(!slot.is_weekend());
    }

    #[test]
    fn test_meal_slot_day_of_week() {
        let monday = NaiveDate::from_ymd_opt(2025, 10, 20).unwrap();
        let slot = MealSlot {
            date: monday,
            meal_type: MealType::Dinner,
        };
        assert_eq!(slot.day_of_week(), 1);

        let sunday = NaiveDate::from_ymd_opt(2025, 10, 26).unwrap();
        let slot = MealSlot {
            date: sunday,
            meal_type: MealType::Dinner,
        };
        assert_eq!(slot.day_of_week(), 7);
    }
}
