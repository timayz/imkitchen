// Profile domain service for business logic calculations

use imkitchen_shared::{DietaryRestriction, FamilySize, SkillLevel};
use crate::domain::{User, UserProfile};
use std::collections::HashSet;

/// Profile service for domain logic calculations and validations
#[derive(Debug, Clone)]
pub struct ProfileService;

impl ProfileService {
    /// Calculate portion multiplier based on family size
    /// Base serving size is 4, so we calculate how to scale recipes
    pub fn calculate_portion_multiplier(family_size: &FamilySize) -> f32 {
        const BASE_SERVING_SIZE: f32 = 4.0;
        family_size.value as f32 / BASE_SERVING_SIZE
    }

    /// Calculate recommended recipe quantity based on family size
    /// Returns number of servings to prepare
    pub fn calculate_recipe_quantity(family_size: &FamilySize, base_servings: u32) -> u32 {
        let multiplier = Self::calculate_portion_multiplier(family_size);
        ((base_servings as f32) * multiplier).ceil() as u32
    }

    /// Calculate profile completeness percentage
    /// Returns percentage of profile completion for gamification
    pub fn calculate_profile_completeness(profile: &UserProfile) -> f32 {
        let mut completeness_score = 0.0;
        let total_possible_score = 100.0;

        // Family size (25% weight - always present as required)
        if profile.family_size.value > 0 {
            completeness_score += 25.0;
        }

        // Skill level (25% weight - always present as enum)
        completeness_score += 25.0;

        // Dietary restrictions (25% weight)
        if !profile.dietary_restrictions.is_empty() {
            completeness_score += 25.0;
        }

        // Cooking time preferences (25% weight)
        if profile.weekday_cooking_minutes > 0 && profile.weekend_cooking_minutes > 0 {
            completeness_score += 25.0;
        }

        (completeness_score / total_possible_score) * 100.0
    }

    /// Validate dietary restrictions compatibility
    /// Returns list of conflicts if any exist
    pub fn validate_dietary_restrictions_compatibility(
        restrictions: &[DietaryRestriction]
    ) -> Result<(), Vec<String>> {
        let mut conflicts = Vec::new();
        let restriction_set: HashSet<_> = restrictions.iter().collect();

        // Check for vegan/vegetarian conflict
        if restriction_set.contains(&DietaryRestriction::Vegan) 
            && restriction_set.contains(&DietaryRestriction::Vegetarian) {
            conflicts.push("Vegan diet already includes vegetarian restrictions".to_string());
        }

        // Check for keto/low-carb redundancy
        if restriction_set.contains(&DietaryRestriction::Keto) 
            && restriction_set.contains(&DietaryRestriction::LowCarb) {
            conflicts.push("Keto diet already includes low-carb restrictions".to_string());
        }

        // Check for too many restrictions (practical limit)
        if restrictions.len() > 5 {
            conflicts.push(format!(
                "Too many dietary restrictions ({}) may limit meal options significantly",
                restrictions.len()
            ));
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }

    /// Calculate cooking time efficiency score
    /// Returns score from 0-100 based on time allocation
    pub fn calculate_cooking_efficiency_score(profile: &UserProfile) -> f32 {
        let _weekday_ratio = profile.weekday_cooking_minutes as f32 / 60.0; // Convert to hours
        let _weekend_ratio = profile.weekend_cooking_minutes as f32 / 60.0;
        
        // Optimal cooking times: 30-60 min weekdays, 60-120 min weekends
        let weekday_score = match profile.weekday_cooking_minutes {
            15..=30 => 80.0,
            31..=45 => 100.0,
            46..=60 => 90.0,
            61..=90 => 70.0,
            _ => 50.0,
        };

        let weekend_score = match profile.weekend_cooking_minutes {
            30..=60 => 80.0,
            61..=90 => 100.0,
            91..=120 => 90.0,
            121..=180 => 70.0,
            _ => 50.0,
        };

        (weekday_score + weekend_score) / 2.0
    }

    /// Determine if user profile suggests they're ready for meal planning
    pub fn is_ready_for_meal_planning(user: &User) -> bool {
        let completeness = Self::calculate_profile_completeness(&user.profile);
        let has_valid_restrictions = Self::validate_dietary_restrictions_compatibility(
            &user.profile.dietary_restrictions
        ).is_ok();
        
        completeness >= 75.0 && has_valid_restrictions
    }

    /// Get profile improvement suggestions
    pub fn get_profile_improvement_suggestions(profile: &UserProfile) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Dietary restrictions suggestion
        if profile.dietary_restrictions.is_empty() {
            suggestions.push("Consider adding dietary restrictions for more personalized meal recommendations".to_string());
        }

        // Cooking time balance suggestion
        if profile.weekday_cooking_minutes > profile.weekend_cooking_minutes {
            suggestions.push("Consider allocating more cooking time on weekends when you have more flexibility".to_string());
        }

        // Very low cooking time warning
        if profile.weekday_cooking_minutes < 15 {
            suggestions.push("Very short weekday cooking times may limit recipe options - consider quick meal prep techniques".to_string());
        }

        // Skill level progression suggestion
        match profile.cooking_skill_level {
            SkillLevel::Beginner => {
                suggestions.push("Start with simple recipes and gradually work up to more complex dishes".to_string());
            },
            SkillLevel::Intermediate => {
                suggestions.push("Try challenging yourself with advanced techniques to improve your skills".to_string());
            },
            SkillLevel::Advanced => {
                suggestions.push("Share your expertise! Consider creating your own recipe variations".to_string());
            },
        }

        suggestions
    }

    /// Calculate weekly cooking load score
    /// Returns estimated total cooking minutes per week
    pub fn calculate_weekly_cooking_load(profile: &UserProfile) -> u32 {
        // Assume 5 weekdays and 2 weekend days
        (profile.weekday_cooking_minutes * 5) + (profile.weekend_cooking_minutes * 2)
    }

    /// Determine optimal meal complexity for user
    pub fn determine_optimal_meal_complexity(profile: &UserProfile) -> Vec<String> {
        let mut complexities = Vec::new();

        match profile.cooking_skill_level {
            SkillLevel::Beginner => {
                complexities.push("Simple".to_string());
                if profile.weekend_cooking_minutes > 60 {
                    complexities.push("Easy".to_string());
                }
            },
            SkillLevel::Intermediate => {
                complexities.push("Easy".to_string());
                complexities.push("Medium".to_string());
                if profile.weekend_cooking_minutes >= 90 {
                    complexities.push("Complex".to_string());
                }
            },
            SkillLevel::Advanced => {
                complexities.push("Easy".to_string());
                complexities.push("Medium".to_string());
                complexities.push("Complex".to_string());
                complexities.push("Gourmet".to_string());
            },
        }

        // Adjust for cooking time constraints
        if profile.weekday_cooking_minutes < 30 {
            complexities.retain(|c| c == "Simple" || c == "Easy");
        }

        complexities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_calculate_portion_multiplier() {
        let family_size = FamilySize::new(6).unwrap();
        let multiplier = ProfileService::calculate_portion_multiplier(&family_size);
        assert_eq!(multiplier, 1.5); // 6/4 = 1.5
    }

    #[test]
    fn test_calculate_recipe_quantity() {
        let family_size = FamilySize::new(3).unwrap();
        let quantity = ProfileService::calculate_recipe_quantity(&family_size, 4);
        assert_eq!(quantity, 3); // ceil(4 * 0.75) = 3
    }

    #[test]
    fn test_calculate_profile_completeness() {
        let profile = create_test_profile();
        let completeness = ProfileService::calculate_profile_completeness(&profile);
        assert_eq!(completeness, 100.0); // All fields complete
    }

    #[test]
    fn test_calculate_profile_completeness_incomplete() {
        let mut profile = create_test_profile();
        profile.dietary_restrictions.clear();
        profile.weekday_cooking_minutes = 0;
        
        let completeness = ProfileService::calculate_profile_completeness(&profile);
        assert_eq!(completeness, 50.0); // Only family_size and skill_level
    }

    #[test]
    fn test_validate_dietary_restrictions_compatibility_valid() {
        let restrictions = vec![DietaryRestriction::Vegetarian, DietaryRestriction::GlutenFree];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&restrictions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dietary_restrictions_compatibility_conflict() {
        let restrictions = vec![DietaryRestriction::Vegan, DietaryRestriction::Vegetarian];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&restrictions);
        assert!(result.is_err());
        
        let conflicts = result.unwrap_err();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("Vegan diet already includes vegetarian"));
    }

    #[test]
    fn test_validate_dietary_restrictions_too_many() {
        let restrictions = vec![
            DietaryRestriction::Vegan,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
            DietaryRestriction::NutFree,
            DietaryRestriction::SoyFree,
            DietaryRestriction::LowSodium,
        ];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&restrictions);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_cooking_efficiency_score() {
        let profile = create_test_profile(); // 45 min weekday, 90 min weekend
        let score = ProfileService::calculate_cooking_efficiency_score(&profile);
        assert_eq!(score, 100.0); // (100 + 100) / 2 = 100 (45min->100, 90min->100)
    }

    #[test]
    fn test_is_ready_for_meal_planning() {
        let user = User {
            user_id: uuid::Uuid::new_v4(),
            email: imkitchen_shared::Email::new("test@example.com".to_string()).unwrap(),
            password_hash: "hash".to_string(),
            profile: create_test_profile(),
            is_email_verified: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let ready = ProfileService::is_ready_for_meal_planning(&user);
        assert!(ready);
    }

    #[test]
    fn test_get_profile_improvement_suggestions() {
        let mut profile = create_test_profile();
        profile.dietary_restrictions.clear();
        profile.weekday_cooking_minutes = 10;
        
        let suggestions = ProfileService::get_profile_improvement_suggestions(&profile);
        assert!(suggestions.len() >= 2);
        assert!(suggestions.iter().any(|s| s.contains("dietary restrictions")));
        assert!(suggestions.iter().any(|s| s.contains("Very short weekday cooking times")));
    }

    #[test]
    fn test_calculate_weekly_cooking_load() {
        let profile = create_test_profile(); // 45 min weekday, 90 min weekend
        let weekly_load = ProfileService::calculate_weekly_cooking_load(&profile);
        assert_eq!(weekly_load, 405); // (45 * 5) + (90 * 2) = 225 + 180 = 405
    }

    #[test]
    fn test_determine_optimal_meal_complexity() {
        let profile = create_test_profile(); // Intermediate skill, 90 min weekend
        let complexities = ProfileService::determine_optimal_meal_complexity(&profile);
        
        assert!(complexities.contains(&"Easy".to_string()));
        assert!(complexities.contains(&"Medium".to_string()));
        assert!(complexities.contains(&"Complex".to_string())); // Weekend time > 90
    }

    #[test]
    fn test_determine_optimal_meal_complexity_beginner() {
        let mut profile = create_test_profile();
        profile.cooking_skill_level = SkillLevel::Beginner;
        profile.weekend_cooking_minutes = 45; // Lower weekend time
        
        let complexities = ProfileService::determine_optimal_meal_complexity(&profile);
        
        assert!(complexities.contains(&"Simple".to_string()));
        assert!(!complexities.contains(&"Complex".to_string()));
    }

    #[test]
    fn test_determine_optimal_meal_complexity_time_constrained() {
        let mut profile = create_test_profile();
        profile.cooking_skill_level = SkillLevel::Advanced;
        profile.weekday_cooking_minutes = 20; // Very short weekday time
        
        let complexities = ProfileService::determine_optimal_meal_complexity(&profile);
        
        // Should be limited to simple/easy despite advanced skill
        assert!(complexities.len() <= 2);
        assert!(complexities.contains(&"Simple".to_string()) || complexities.contains(&"Easy".to_string()));
    }
}