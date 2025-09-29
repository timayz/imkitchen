// Recommendation service for personalized meal planning

use crate::domain::UserProfile;
use imkitchen_shared::{DietaryRestriction, SkillLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recommendation service for personalized meal planning
#[derive(Debug, Clone)]
pub struct RecommendationService;

/// Recipe recommendation criteria based on user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeRecommendationCriteria {
    pub max_cooking_time: u32,
    pub skill_level_tags: Vec<String>,
    pub dietary_filters: Vec<DietaryRestriction>,
    pub portion_multiplier: f32,
    pub complexity_levels: Vec<String>,
    pub preferred_categories: Vec<String>,
}

/// Meal planning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealPlanningRecommendations {
    pub weekly_meal_count: u32,
    pub prep_day_suggestions: Vec<String>,
    pub cooking_schedule: HashMap<String, u32>, // Day -> minutes
    pub batch_cooking_opportunities: Vec<String>,
    pub quick_meal_slots: Vec<String>,
}

/// Skill progression recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProgressionRecommendations {
    pub current_level: SkillLevel,
    pub next_level_requirements: Vec<String>,
    pub recommended_techniques: Vec<String>,
    pub practice_recipes: Vec<String>,
    pub estimated_progression_time: String,
}

impl RecommendationService {
    /// Generate recipe recommendation criteria based on user profile
    pub fn generate_recipe_criteria(
        profile: &UserProfile,
        is_weekday: bool,
    ) -> RecipeRecommendationCriteria {
        let max_cooking_time = if is_weekday {
            profile.weekday_cooking_minutes
        } else {
            profile.weekend_cooking_minutes
        };

        let skill_level_tags = match profile.cooking_skill_level {
            SkillLevel::Beginner => vec![
                "quick".to_string(),
                "simple".to_string(),
                "one-pot".to_string(),
            ],
            SkillLevel::Intermediate => vec![
                "quick".to_string(),
                "simple".to_string(),
                "moderate".to_string(),
                "multi-step".to_string(),
            ],
            SkillLevel::Advanced => vec![
                "moderate".to_string(),
                "complex".to_string(),
                "gourmet".to_string(),
                "technique-focused".to_string(),
            ],
        };

        let complexity_levels = Self::determine_complexity_levels(profile, is_weekday);
        let preferred_categories = Self::determine_preferred_categories(profile);

        RecipeRecommendationCriteria {
            max_cooking_time,
            skill_level_tags,
            dietary_filters: profile.dietary_restrictions.clone(),
            portion_multiplier: profile.calculate_portions(4) as f32 / 4.0,
            complexity_levels,
            preferred_categories,
        }
    }

    /// Determine appropriate complexity levels for recipes
    pub fn determine_complexity_levels(profile: &UserProfile, is_weekday: bool) -> Vec<String> {
        let mut levels = Vec::new();

        let time_limit = if is_weekday {
            profile.weekday_cooking_minutes
        } else {
            profile.weekend_cooking_minutes
        };

        // Base complexity on skill level
        match profile.cooking_skill_level {
            SkillLevel::Beginner => {
                levels.push("Simple".to_string());
                if time_limit >= 45 || !is_weekday {
                    levels.push("Easy".to_string());
                }
            }
            SkillLevel::Intermediate => {
                levels.push("Easy".to_string());
                levels.push("Medium".to_string());
                if time_limit >= 60 || !is_weekday {
                    levels.push("Complex".to_string());
                }
            }
            SkillLevel::Advanced => {
                if time_limit < 30 && is_weekday {
                    levels.push("Easy".to_string());
                    levels.push("Medium".to_string());
                } else {
                    levels.push("Medium".to_string());
                    levels.push("Complex".to_string());
                    levels.push("Gourmet".to_string());
                }
            }
        }

        levels
    }

    /// Determine preferred recipe categories based on dietary restrictions and profile
    pub fn determine_preferred_categories(profile: &UserProfile) -> Vec<String> {
        let mut categories = Vec::new();

        // Base categories for everyone
        categories.extend([
            "dinner".to_string(),
            "lunch".to_string(),
            "breakfast".to_string(),
        ]);

        // Add categories based on dietary restrictions
        for restriction in &profile.dietary_restrictions {
            match restriction {
                DietaryRestriction::Vegetarian => {
                    categories.push("vegetarian".to_string());
                    categories.push("plant-based".to_string());
                }
                DietaryRestriction::Vegan => {
                    categories.push("vegan".to_string());
                    categories.push("plant-based".to_string());
                }
                DietaryRestriction::GlutenFree => {
                    categories.push("gluten-free".to_string());
                }
                DietaryRestriction::Keto => {
                    categories.push("keto".to_string());
                    categories.push("low-carb".to_string());
                }
                DietaryRestriction::LowCarb => {
                    categories.push("low-carb".to_string());
                }
                DietaryRestriction::Paleo => {
                    categories.push("paleo".to_string());
                    categories.push("whole-foods".to_string());
                }
                _ => {} // Other restrictions don't add specific categories
            }
        }

        // Add categories based on family size
        if profile.family_size.value >= 6 {
            categories.push("large-family".to_string());
            categories.push("batch-cooking".to_string());
        } else if profile.family_size.value <= 2 {
            categories.push("small-portions".to_string());
            categories.push("couples".to_string());
        }

        categories.sort();
        categories.dedup();
        categories
    }

    /// Generate meal planning recommendations
    pub fn generate_meal_planning_recommendations(
        profile: &UserProfile,
    ) -> MealPlanningRecommendations {
        let weekly_meal_count = Self::calculate_optimal_weekly_meals(profile);
        let prep_day_suggestions = Self::suggest_prep_days(profile);
        let cooking_schedule = Self::create_cooking_schedule(profile);
        let batch_cooking_opportunities = Self::identify_batch_cooking_opportunities(profile);
        let quick_meal_slots = Self::identify_quick_meal_slots(profile);

        MealPlanningRecommendations {
            weekly_meal_count,
            prep_day_suggestions,
            cooking_schedule,
            batch_cooking_opportunities,
            quick_meal_slots,
        }
    }

    /// Calculate optimal number of meals to plan per week
    fn calculate_optimal_weekly_meals(profile: &UserProfile) -> u32 {
        let base_meals = 14; // 2 meals per day * 7 days
        let cooking_capacity =
            (profile.weekday_cooking_minutes * 5 + profile.weekend_cooking_minutes * 2) / 60;

        match cooking_capacity {
            0..=5 => base_meals / 2, // Very limited time - focus on key meals
            6..=10 => (base_meals * 3 / 4).max(10), // Moderate time - at least 10 meals
            11..=15 => base_meals * 4 / 5, // Good time - 11-12 meals
            _ => base_meals,         // Plenty of time - full meal planning
        }
    }

    /// Suggest optimal prep days based on cooking time allocation
    fn suggest_prep_days(profile: &UserProfile) -> Vec<String> {
        let mut suggestions = Vec::new();

        if profile.weekend_cooking_minutes >= 90 {
            suggestions.push("Sunday - Batch cooking and meal prep".to_string());
        }

        if profile.weekend_cooking_minutes >= 60 {
            suggestions.push("Saturday - Fresh ingredients prep".to_string());
        }

        if profile.weekday_cooking_minutes >= 45 {
            suggestions.push("Wednesday - Mid-week prep refresh".to_string());
        }

        if suggestions.is_empty() {
            suggestions
                .push("Choose your least busy evening for 30-minute prep sessions".to_string());
        }

        suggestions
    }

    /// Create a weekly cooking schedule
    fn create_cooking_schedule(profile: &UserProfile) -> HashMap<String, u32> {
        let mut schedule = HashMap::new();

        let weekday_time = profile.weekday_cooking_minutes;
        let weekend_time = profile.weekend_cooking_minutes;

        // Weekdays
        for day in ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"] {
            schedule.insert(day.to_string(), weekday_time);
        }

        // Weekends
        schedule.insert("Saturday".to_string(), weekend_time);
        schedule.insert("Sunday".to_string(), weekend_time);

        schedule
    }

    /// Identify opportunities for batch cooking
    fn identify_batch_cooking_opportunities(profile: &UserProfile) -> Vec<String> {
        let mut opportunities = Vec::new();

        if profile.weekend_cooking_minutes >= 120 {
            opportunities.push("Weekend batch cooking: Prepare 4-5 meals at once".to_string());
            opportunities
                .push("Freezer-friendly meals: Cook double portions and freeze half".to_string());
        }

        if profile.weekend_cooking_minutes >= 90 {
            opportunities.push("Sunday meal prep: Prepare ingredients for the week".to_string());
            opportunities.push("One-pot meals: Stews, casseroles, and grain bowls".to_string());
        }

        if profile.family_size.value >= 4 {
            opportunities.push("Large batch recipes: Scale up family favorites".to_string());
        }

        if opportunities.is_empty() && profile.weekday_cooking_minutes >= 30 {
            opportunities.push("Double up: Cook extra portions for next day's lunch".to_string());
        }

        opportunities
    }

    /// Identify slots for quick meals
    fn identify_quick_meal_slots(profile: &UserProfile) -> Vec<String> {
        let mut slots = Vec::new();

        if profile.weekday_cooking_minutes <= 30 {
            slots.push("Weekday dinners: 15-20 minute meals".to_string());
            slots.push("Breakfast: 5-10 minute options".to_string());
            slots.push("Lunch: Pre-prepared or assembly meals".to_string());
        }

        if profile.weekday_cooking_minutes <= 20 {
            slots.push("Emergency meals: 10-minute pantry staples".to_string());
            slots.push("No-cook options: Salads, sandwiches, smoothie bowls".to_string());
        }

        slots
    }

    /// Generate skill progression recommendations
    pub fn generate_skill_progression_recommendations(
        profile: &UserProfile,
    ) -> SkillProgressionRecommendations {
        let current_level = profile.cooking_skill_level;
        let next_level_requirements = Self::get_next_level_requirements(&current_level);
        let recommended_techniques = Self::get_recommended_techniques(&current_level);
        let practice_recipes = Self::get_practice_recipes(&current_level);
        let estimated_progression_time = Self::estimate_progression_time(&current_level, profile);

        SkillProgressionRecommendations {
            current_level,
            next_level_requirements,
            recommended_techniques,
            practice_recipes,
            estimated_progression_time,
        }
    }

    /// Get requirements for advancing to next skill level
    fn get_next_level_requirements(current_level: &SkillLevel) -> Vec<String> {
        match current_level {
            SkillLevel::Beginner => vec![
                "Master basic knife skills (dicing, chopping, slicing)".to_string(),
                "Learn fundamental cooking methods (sautéing, boiling, baking)".to_string(),
                "Successfully prepare 10 different recipes".to_string(),
                "Understand basic flavor combinations".to_string(),
            ],
            SkillLevel::Intermediate => vec![
                "Master advanced techniques (braising, roasting, grilling)".to_string(),
                "Learn to cook without recipes for familiar dishes".to_string(),
                "Successfully prepare complex multi-component meals".to_string(),
                "Understand ingredient substitutions and adaptations".to_string(),
            ],
            SkillLevel::Advanced => vec![
                "Already at advanced level!".to_string(),
                "Continue exploring international cuisines".to_string(),
                "Develop signature dishes and personal style".to_string(),
                "Consider teaching others or professional development".to_string(),
            ],
        }
    }

    /// Get recommended techniques to practice
    fn get_recommended_techniques(current_level: &SkillLevel) -> Vec<String> {
        match current_level {
            SkillLevel::Beginner => vec![
                "Proper knife grip and basic cuts".to_string(),
                "Sautéing vegetables without burning".to_string(),
                "Cooking pasta to al dente".to_string(),
                "Pan-frying proteins to proper doneness".to_string(),
                "Making simple sauces and dressings".to_string(),
            ],
            SkillLevel::Intermediate => vec![
                "Braising tough cuts of meat".to_string(),
                "Making stocks and broths from scratch".to_string(),
                "Proper seasoning and taste adjustment".to_string(),
                "Baking breads and pastries".to_string(),
                "Grilling and outdoor cooking techniques".to_string(),
            ],
            SkillLevel::Advanced => vec![
                "Molecular gastronomy techniques".to_string(),
                "Advanced fermentation projects".to_string(),
                "Butchering and fish filleting".to_string(),
                "Creating original recipe combinations".to_string(),
                "Professional plating and presentation".to_string(),
            ],
        }
    }

    /// Get practice recipes appropriate for skill level
    fn get_practice_recipes(current_level: &SkillLevel) -> Vec<String> {
        match current_level {
            SkillLevel::Beginner => vec![
                "Simple stir-fry with vegetables".to_string(),
                "Basic pasta with tomato sauce".to_string(),
                "Scrambled eggs and toast".to_string(),
                "Roasted chicken thighs".to_string(),
                "Green salad with homemade vinaigrette".to_string(),
            ],
            SkillLevel::Intermediate => vec![
                "Beef stew with root vegetables".to_string(),
                "Homemade pizza with fresh dough".to_string(),
                "Pan-seared salmon with lemon butter".to_string(),
                "Risotto with seasonal vegetables".to_string(),
                "Chocolate chip cookies from scratch".to_string(),
            ],
            SkillLevel::Advanced => vec![
                "Duck confit with cherry gastrique".to_string(),
                "Handmade pasta with complex sauce".to_string(),
                "Beef Wellington with mushroom duxelles".to_string(),
                "Sourdough bread with natural starter".to_string(),
                "Multi-course tasting menu".to_string(),
            ],
        }
    }

    /// Estimate time needed to progress to next level
    fn estimate_progression_time(current_level: &SkillLevel, profile: &UserProfile) -> String {
        let weekly_cooking_hours = (profile.weekday_cooking_minutes * 5
            + profile.weekend_cooking_minutes * 2) as f32
            / 60.0;

        let base_weeks = match current_level {
            SkillLevel::Beginner => 12,     // 3 months to intermediate
            SkillLevel::Intermediate => 24, // 6 months to advanced
            SkillLevel::Advanced => 0,      // Already at top
        };

        if current_level == &SkillLevel::Advanced {
            return "Continue exploring and refining your advanced skills!".to_string();
        }

        let adjusted_weeks = if weekly_cooking_hours >= 8.0 {
            base_weeks * 3 / 4 // Faster progression with more practice
        } else if weekly_cooking_hours >= 5.0 {
            base_weeks // Normal progression
        } else {
            base_weeks * 4 / 3 // Slower progression with less practice
        };

        if adjusted_weeks <= 4 {
            format!("About {} weeks with consistent practice", adjusted_weeks)
        } else if adjusted_weeks <= 12 {
            format!("About {} months with regular cooking", adjusted_weeks / 4)
        } else {
            format!(
                "About {} months with dedicated practice",
                adjusted_weeks / 4
            )
        }
    }

    /// Generate personalized cooking tips based on profile
    pub fn generate_cooking_tips(profile: &UserProfile) -> Vec<String> {
        let mut tips = Vec::new();

        // Time-based tips
        if profile.weekday_cooking_minutes <= 20 {
            tips.push("Pre-cut vegetables on weekends to save weekday time".to_string());
            tips.push("Keep a well-stocked pantry for quick meal assembly".to_string());
        }

        // Skill-based tips
        match profile.cooking_skill_level {
            SkillLevel::Beginner => {
                tips.push("Taste as you go - seasoning is key to good food".to_string());
                tips.push(
                    "Don't be afraid to make mistakes - they're learning opportunities".to_string(),
                );
            }
            SkillLevel::Intermediate => {
                tips.push("Experiment with new cuisines to expand your repertoire".to_string());
                tips.push("Learn to cook by sight and smell, not just timers".to_string());
            }
            SkillLevel::Advanced => {
                tips.push(
                    "Try teaching others - it will deepen your own understanding".to_string(),
                );
                tips.push(
                    "Challenge yourself with unfamiliar ingredients and techniques".to_string(),
                );
            }
        }

        // Dietary restriction tips
        if profile
            .dietary_restrictions
            .contains(&DietaryRestriction::Vegetarian)
        {
            tips.push("Explore plant-based proteins like legumes, tofu, and tempeh".to_string());
        }

        if profile
            .dietary_restrictions
            .contains(&DietaryRestriction::GlutenFree)
        {
            tips.push("Stock up on alternative grains like quinoa, rice, and millet".to_string());
        }

        // Family size tips
        if profile.family_size.value >= 6 {
            tips.push("Invest in larger pots and pans for efficient big-batch cooking".to_string());
        } else if profile.family_size.value <= 2 {
            tips.push(
                "Look for recipes that scale down well or provide good leftovers".to_string(),
            );
        }

        tips
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use imkitchen_shared::FamilySize;

    fn create_test_profile() -> UserProfile {
        UserProfile {
            family_size: FamilySize::new(4).unwrap(),
            cooking_skill_level: SkillLevel::Intermediate,
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        }
    }

    #[test]
    fn test_generate_recipe_criteria_weekday() {
        let profile = create_test_profile();
        let criteria = RecommendationService::generate_recipe_criteria(&profile, true);

        assert_eq!(criteria.max_cooking_time, 45);
        assert!(criteria.skill_level_tags.contains(&"moderate".to_string()));
        assert_eq!(
            criteria.dietary_filters,
            vec![DietaryRestriction::Vegetarian]
        );
        assert!(criteria.complexity_levels.contains(&"Easy".to_string()));
        assert!(criteria.complexity_levels.contains(&"Medium".to_string()));
    }

    #[test]
    fn test_generate_recipe_criteria_weekend() {
        let profile = create_test_profile();
        let criteria = RecommendationService::generate_recipe_criteria(&profile, false);

        assert_eq!(criteria.max_cooking_time, 90);
        assert!(criteria.complexity_levels.contains(&"Complex".to_string()));
    }

    #[test]
    fn test_determine_complexity_levels_beginner() {
        let mut profile = create_test_profile();
        profile.cooking_skill_level = SkillLevel::Beginner;

        let levels = RecommendationService::determine_complexity_levels(&profile, true);

        assert!(levels.contains(&"Simple".to_string()));
        assert!(levels.contains(&"Easy".to_string()));
        assert!(!levels.contains(&"Complex".to_string()));
    }

    #[test]
    fn test_determine_preferred_categories() {
        let profile = create_test_profile();
        let categories = RecommendationService::determine_preferred_categories(&profile);

        assert!(categories.contains(&"vegetarian".to_string()));
        assert!(categories.contains(&"plant-based".to_string()));
        assert!(categories.contains(&"dinner".to_string()));
    }

    #[test]
    fn test_generate_meal_planning_recommendations() {
        let profile = create_test_profile();
        let recommendations =
            RecommendationService::generate_meal_planning_recommendations(&profile);

        assert!(recommendations.weekly_meal_count > 0);
        assert!(!recommendations.prep_day_suggestions.is_empty());
        assert_eq!(recommendations.cooking_schedule.len(), 7); // 7 days
        assert!(!recommendations.batch_cooking_opportunities.is_empty());
    }

    #[test]
    fn test_calculate_optimal_weekly_meals() {
        let profile = create_test_profile();
        let meal_count = RecommendationService::calculate_optimal_weekly_meals(&profile);

        // With good cooking time allocation, should plan most meals
        assert!(meal_count >= 10);
    }

    #[test]
    fn test_generate_skill_progression_recommendations() {
        let profile = create_test_profile();
        let recommendations =
            RecommendationService::generate_skill_progression_recommendations(&profile);

        assert_eq!(recommendations.current_level, SkillLevel::Intermediate);
        assert!(!recommendations.next_level_requirements.is_empty());
        assert!(!recommendations.recommended_techniques.is_empty());
        assert!(!recommendations.practice_recipes.is_empty());
    }

    #[test]
    fn test_get_practice_recipes_by_skill_level() {
        let beginner_recipes = RecommendationService::get_practice_recipes(&SkillLevel::Beginner);
        let advanced_recipes = RecommendationService::get_practice_recipes(&SkillLevel::Advanced);

        assert!(beginner_recipes.iter().any(|r| r.contains("stir-fry")));
        assert!(advanced_recipes.iter().any(|r| r.contains("Wellington")));
    }

    #[test]
    fn test_generate_cooking_tips() {
        let mut profile = create_test_profile();
        profile.weekday_cooking_minutes = 15; // Very limited time
        profile.cooking_skill_level = SkillLevel::Beginner;

        let tips = RecommendationService::generate_cooking_tips(&profile);

        assert!(tips.iter().any(|t| t.contains("Pre-cut vegetables")));
        assert!(tips.iter().any(|t| t.contains("Taste as you go")));
        assert!(tips.iter().any(|t| t.contains("plant-based proteins")));
    }

    #[test]
    fn test_estimate_progression_time() {
        let mut profile = create_test_profile();
        profile.cooking_skill_level = SkillLevel::Beginner;

        let time_estimate =
            RecommendationService::estimate_progression_time(&SkillLevel::Beginner, &profile);

        assert!(time_estimate.contains("months") || time_estimate.contains("weeks"));
    }

    #[test]
    fn test_advanced_skill_level_progression() {
        let profile = create_test_profile();
        let time_estimate =
            RecommendationService::estimate_progression_time(&SkillLevel::Advanced, &profile);

        assert!(time_estimate.contains("Continue exploring"));
    }
}
