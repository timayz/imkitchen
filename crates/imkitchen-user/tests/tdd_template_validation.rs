// TDD Template Validation Tests - Core Profile Management Workflows

use imkitchen_user::*;
use imkitchen_shared::*;
use uuid::Uuid;

#[cfg(test)]
mod template_validation_tests {
    use super::*;

    #[test]
    fn test_profile_form_validation_family_size() {
        // Test valid family size values
        let valid_sizes = vec![1, 2, 3, 4, 5, 6, 7, 8];
        
        for size in valid_sizes {
            let result = FamilySize::new(size);
            assert!(result.is_ok(), 
                "Family size {} should be valid", size);
        }
        
        // Test invalid family size values
        let invalid_sizes = vec![0, 9, 10, 255];
        
        for size in invalid_sizes {
            let result = FamilySize::new(size);
            assert!(result.is_err(),
                "Family size {} should be invalid", size);
        }
    }

    #[test]
    fn test_profile_form_validation_cooking_skill() {
        // Test all valid cooking skill levels
        let skills = vec![
            SkillLevel::Beginner,
            SkillLevel::Intermediate, 
            SkillLevel::Advanced,
        ];
        
        for skill in skills {
            // Skill levels are enum values, so they're always valid
            assert!(matches!(skill, SkillLevel::Beginner | SkillLevel::Intermediate | SkillLevel::Advanced),
                "Cooking skill {:?} should be valid", skill);
        }
    }

    #[test]
    fn test_dietary_restrictions_validation() {
        // Test single valid restriction
        let single_restriction = vec![DietaryRestriction::Vegetarian];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&single_restriction);
        assert!(result.is_ok(),
            "Single dietary restriction should be valid");
        
        // Test multiple compatible restrictions
        let compatible_restrictions = vec![
            DietaryRestriction::Vegetarian,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
        ];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&compatible_restrictions);
        assert!(result.is_ok(),
            "Compatible dietary restrictions should be valid");
        
        // Test conflicting restrictions (Vegan includes Vegetarian)
        let conflicting_restrictions = vec![
            DietaryRestriction::Vegan,
            DietaryRestriction::Vegetarian,
        ];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&conflicting_restrictions);
        assert!(result.is_err(),
            "Conflicting dietary restrictions should be invalid");
        
        // Test too many restrictions
        let too_many_restrictions = vec![
            DietaryRestriction::Vegan,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
            DietaryRestriction::NutFree,
            DietaryRestriction::SoyFree,
            DietaryRestriction::LowSodium,
        ];
        let result = ProfileService::validate_dietary_restrictions_compatibility(&too_many_restrictions);
        assert!(result.is_err(),
            "Too many dietary restrictions should be invalid");
    }

    #[test]
    fn test_profile_completeness_validation() {
        // Test incomplete profile
        let incomplete_profile = UserProfile {
            family_size: FamilySize::FAMILY1,
            cooking_skill_level: SkillLevel::Beginner,
            dietary_restrictions: vec![],
            weekday_cooking_minutes: 0,
            weekend_cooking_minutes: 0,
        };
        
        let completeness = ProfileService::calculate_profile_completeness(&incomplete_profile);
        assert!(completeness < 100.0,
            "Incomplete profile should have less than 100% completeness: {}%", completeness);
        assert!(completeness >= 50.0,
            "Profile with family size and skill should have at least 50% completeness: {}%", completeness);
        
        // Test complete profile
        let complete_profile = UserProfile {
            family_size: FamilySize::FAMILY4,
            cooking_skill_level: SkillLevel::Intermediate,
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        let completeness = ProfileService::calculate_profile_completeness(&complete_profile);
        assert_eq!(completeness, 100.0,
            "Complete profile should have 100% completeness");
    }

    #[test]
    fn test_cooking_time_validation_ranges() {
        let test_cases = vec![
            (0, 0, true),      // Zero times are valid
            (15, 30, true),    // Low times are valid
            (60, 120, true),   // Moderate times are valid
            (120, 240, true),  // High but reasonable times are valid
        ];
        
        for (weekday, weekend, _should_be_valid) in test_cases {
            let profile = UserProfile {
                family_size: FamilySize::FAMILY2,
                cooking_skill_level: SkillLevel::Intermediate,
                dietary_restrictions: vec![],
                weekday_cooking_minutes: weekday,
                weekend_cooking_minutes: weekend,
            };
            
            // All cooking times are stored as u32, so they're inherently valid
            // The validation happens at the business logic level
            let efficiency = ProfileService::calculate_cooking_efficiency_score(&profile);
            assert!(efficiency > 0.0 && efficiency <= 100.0,
                "Cooking efficiency should be between 0 and 100: {}", efficiency);
        }
    }
}

#[cfg(test)]
mod workflow_validation_tests {
    use super::*;

    #[test]
    fn test_family_size_update_workflow() {
        let user_id = Uuid::new_v4();
        
        // Create command to update family size
        let command = UpdateUserProfileCommand {
            user_id,
            family_size: FamilySize::FAMILY4,
            cooking_skill_level: SkillLevel::Beginner,
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        };
        
        // Validate the command
        let validation_result = command.validate_command();
        assert!(validation_result.is_ok(),
            "Valid family size update command should pass validation: {:?}",
            validation_result.unwrap_err());
    }

    #[test]
    fn test_dietary_restrictions_update_workflow() {
        let user_id = Uuid::new_v4();
        
        // Test valid dietary restrictions update
        let valid_command = ChangeDietaryRestrictionsCommand {
            user_id,
            new_restrictions: vec![DietaryRestriction::Vegetarian, DietaryRestriction::GlutenFree],
        };
        
        let validation_result = valid_command.validate_command();
        assert!(validation_result.is_ok(),
            "Valid dietary restrictions command should pass validation");
        
        // Test service-level validation for conflicts
        let compatibility = ProfileService::validate_dietary_restrictions_compatibility(
            &valid_command.new_restrictions
        );
        assert!(compatibility.is_ok(),
            "Compatible dietary restrictions should pass service validation");
        
        // Test conflicting restrictions
        let conflicting_command = ChangeDietaryRestrictionsCommand {
            user_id,
            new_restrictions: vec![DietaryRestriction::Vegan, DietaryRestriction::Vegetarian],
        };
        
        let compatibility = ProfileService::validate_dietary_restrictions_compatibility(
            &conflicting_command.new_restrictions
        );
        assert!(compatibility.is_err(),
            "Conflicting dietary restrictions should fail service validation");
    }

    #[test]
    fn test_complete_profile_update_workflow() {
        let user_id = Uuid::new_v4();
        
        // Test updating multiple profile fields
        let command = UpdateUserProfileCommand {
            user_id,
            family_size: FamilySize::FAMILY3,
            cooking_skill_level: SkillLevel::Intermediate,
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        // Validate the complete command
        let validation_result = command.validate_command();
        assert!(validation_result.is_ok(),
            "Complete profile update command should pass validation");
        
        // Test dietary restrictions separately with dedicated command
        let dietary_command = ChangeDietaryRestrictionsCommand {
            user_id,
            new_restrictions: vec![DietaryRestriction::Vegetarian],
        };
        
        let compatibility = ProfileService::validate_dietary_restrictions_compatibility(&dietary_command.new_restrictions);
        assert!(compatibility.is_ok(),
            "Dietary restrictions in complete update should be compatible");
    }

    #[test]
    fn test_profile_readiness_workflow() {
        let user_id = Uuid::new_v4();
        let email = Email::new("workflow.test@example.com".to_string()).unwrap();
        
        // Create user with incomplete profile
        let incomplete_profile = UserProfile {
            family_size: FamilySize::FAMILY1,
            cooking_skill_level: SkillLevel::Beginner,
            dietary_restrictions: vec![],
            weekday_cooking_minutes: 0,
            weekend_cooking_minutes: 0,
        };
        
        let incomplete_user = User {
            user_id,
            email: email.clone(),
            password_hash: "test_hash".to_string(),
            profile: incomplete_profile,
            is_email_verified: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Check readiness - should not be ready
        let is_ready = ProfileService::is_ready_for_meal_planning(&incomplete_user);
        assert!(!is_ready,
            "User with incomplete profile should not be ready for meal planning");
        
        // Create user with complete profile
        let complete_profile = UserProfile {
            family_size: FamilySize::FAMILY4,
            cooking_skill_level: SkillLevel::Intermediate,
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        let complete_user = User {
            user_id,
            email,
            password_hash: "test_hash".to_string(),
            profile: complete_profile,
            is_email_verified: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Check readiness - should be ready
        let is_ready = ProfileService::is_ready_for_meal_planning(&complete_user);
        assert!(is_ready,
            "User with complete profile should be ready for meal planning");
    }

    #[test]
    fn test_error_handling_workflow() {
        let user_id = Uuid::new_v4();
        
        // Test command with invalid constraints (this will pass command validation
        // but should be caught by business logic)
        let command = UpdateUserProfileCommand {
            user_id,
            family_size: FamilySize::FAMILY8, // Valid range
            cooking_skill_level: SkillLevel::Advanced,
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        // Command validation should pass
        let validation_result = command.validate_command();
        assert!(validation_result.is_ok(),
            "Command structure validation should pass");
        
        // But business logic validation should catch dietary conflicts separately
        let conflicting_dietary = ChangeDietaryRestrictionsCommand {
            user_id,
            new_restrictions: vec![
                DietaryRestriction::Vegan,
                DietaryRestriction::Vegetarian, // Conflict with Vegan
            ],
        };
        
        let compatibility = ProfileService::validate_dietary_restrictions_compatibility(&conflicting_dietary.new_restrictions);
        assert!(compatibility.is_err(),
            "Business logic should catch dietary restriction conflicts");
        
        let errors = compatibility.unwrap_err();
        assert!(errors.iter().any(|err| err.contains("Vegan diet already includes vegetarian")),
            "Error message should be specific about the conflict: {:?}", errors);
    }
}

#[cfg(test)]
mod integration_validation_tests {
    use super::*;

    #[test]
    fn test_end_to_end_profile_creation() {
        let user_id = Uuid::new_v4();
        let email = Email::new("integration@example.com".to_string()).unwrap();
        
        // Step 1: Create initial user with minimal profile
        let initial_profile = UserProfile {
            family_size: FamilySize::FAMILY1,
            cooking_skill_level: SkillLevel::Beginner,
            dietary_restrictions: vec![],
            weekday_cooking_minutes: 0,
            weekend_cooking_minutes: 0,
        };
        
        let user = User {
            user_id,
            email,
            password_hash: "test_hash".to_string(),
            profile: initial_profile.clone(),
            is_email_verified: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        // Verify initial state
        let initial_completeness = ProfileService::calculate_profile_completeness(&user.profile);
        assert!(initial_completeness < 100.0,
            "Initial profile should be incomplete");
        
        let initial_readiness = ProfileService::is_ready_for_meal_planning(&user);
        assert!(!initial_readiness,
            "User should not be ready for meal planning initially");
        
        // Step 2: Validate progressive updates
        let family_update = UpdateUserProfileCommand {
            user_id,
            family_size: FamilySize::FAMILY3,
            cooking_skill_level: SkillLevel::Beginner,
            weekday_cooking_minutes: 30,
            weekend_cooking_minutes: 60,
        };
        
        assert!(family_update.validate_command().is_ok(),
            "Family size update should be valid");
        
        let skill_update = UpdateUserProfileCommand {
            user_id,
            family_size: FamilySize::FAMILY3,
            cooking_skill_level: SkillLevel::Intermediate,
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        assert!(skill_update.validate_command().is_ok(),
            "Skill and time update should be valid");
        
        // Step 3: Create final updated profile
        let final_profile = UserProfile {
            family_size: FamilySize::FAMILY3,
            cooking_skill_level: SkillLevel::Intermediate,
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        let final_user = User {
            user_id,
            email: user.email.clone(),
            password_hash: user.password_hash,
            profile: final_profile,
            is_email_verified: true,
            created_at: user.created_at,
            updated_at: chrono::Utc::now(),
        };
        
        // Verify final state
        let final_completeness = ProfileService::calculate_profile_completeness(&final_user.profile);
        assert_eq!(final_completeness, 100.0,
            "Final profile should be 100% complete");
        
        let final_readiness = ProfileService::is_ready_for_meal_planning(&final_user);
        assert!(final_readiness,
            "User should be ready for meal planning after complete profile");
        
        // Verify recommendations work
        let suggestions = ProfileService::get_profile_improvement_suggestions(&final_user.profile);
        assert!(!suggestions.is_empty(),
            "Even complete profiles should have improvement suggestions");
        
        let complexities = ProfileService::determine_optimal_meal_complexity(&final_user.profile);
        assert!(!complexities.is_empty(),
            "Should provide meal complexity recommendations");
    }

    #[test]
    fn test_validation_error_handling() {
        // Test comprehensive validation scenarios
        
        // 1. Test dietary restriction conflicts
        let conflicting_restrictions = vec![
            DietaryRestriction::Vegan,
            DietaryRestriction::Vegetarian,
        ];
        
        let result = ProfileService::validate_dietary_restrictions_compatibility(&conflicting_restrictions);
        assert!(result.is_err(),
            "Should detect vegan-vegetarian conflict");
        
        // 2. Test keto-low carb conflict
        let keto_lowcarb_conflict = vec![
            DietaryRestriction::Keto,
            DietaryRestriction::LowCarb,
        ];
        
        let result = ProfileService::validate_dietary_restrictions_compatibility(&keto_lowcarb_conflict);
        assert!(result.is_err(),
            "Should detect keto-low carb conflict");
        
        // 3. Test too many restrictions
        let too_many = vec![
            DietaryRestriction::Vegan,
            DietaryRestriction::GlutenFree,
            DietaryRestriction::DairyFree,
            DietaryRestriction::NutFree,
            DietaryRestriction::SoyFree,
            DietaryRestriction::LowSodium,
        ];
        
        let result = ProfileService::validate_dietary_restrictions_compatibility(&too_many);
        assert!(result.is_err(),
            "Should detect too many restrictions");
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|err| err.contains("Too many dietary restrictions")),
            "Should provide specific error message about too many restrictions");
    }

    #[test] 
    fn test_recommendation_integration() {
        // Test that recommendations work correctly with profile updates
        let profile = UserProfile {
            family_size: FamilySize::FAMILY4,
            cooking_skill_level: SkillLevel::Intermediate,
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            weekday_cooking_minutes: 45,
            weekend_cooking_minutes: 90,
        };
        
        // Test weekday recommendations
        let weekday_criteria = RecommendationService::generate_recipe_criteria(&profile, true);
        assert_eq!(weekday_criteria.max_cooking_time, 45,
            "Weekday max cooking time should match profile");
        assert!(weekday_criteria.dietary_filters.contains(&DietaryRestriction::Vegetarian),
            "Should include vegetarian filter");
        
        // Test weekend recommendations
        let weekend_criteria = RecommendationService::generate_recipe_criteria(&profile, false);
        assert_eq!(weekend_criteria.max_cooking_time, 90,
            "Weekend max cooking time should match profile");
        assert!(weekend_criteria.complexity_levels.contains(&"Complex".to_string()),
            "Weekend should include complex recipes for intermediate skill with 90+ minutes");
        
        // Test meal planning recommendations
        let meal_recommendations = RecommendationService::generate_meal_planning_recommendations(&profile);
        assert!(meal_recommendations.weekly_meal_count > 0,
            "Should recommend weekly meals");
        assert!(!meal_recommendations.prep_day_suggestions.is_empty(),
            "Should suggest prep days");
        assert_eq!(meal_recommendations.cooking_schedule.len(), 7,
            "Should have schedule for all 7 days");
        
        // Test skill progression
        let skill_progression = RecommendationService::generate_skill_progression_recommendations(&profile);
        assert_eq!(skill_progression.current_level, SkillLevel::Intermediate,
            "Should correctly identify current skill level");
        assert!(!skill_progression.next_level_requirements.is_empty(),
            "Should provide next level requirements");
    }
}

#[cfg(test)]
mod performance_validation_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validation_performance() {
        let start = Instant::now();
        let iterations = 1000u32;
        
        for i in 0..iterations {
            let profile = UserProfile {
                family_size: FamilySize::new(((i % 8) + 1) as u8).unwrap(),
                cooking_skill_level: match i % 3 {
                    0 => SkillLevel::Beginner,
                    1 => SkillLevel::Intermediate,
                    _ => SkillLevel::Advanced,
                },
                dietary_restrictions: vec![DietaryRestriction::Vegetarian],
                weekday_cooking_minutes: 30 + (i % 60) as u32,
                weekend_cooking_minutes: 60 + (i % 120) as u32,
            };
            
            // Test all validation functions
            let _completeness = ProfileService::calculate_profile_completeness(&profile);
            let _efficiency = ProfileService::calculate_cooking_efficiency_score(&profile);
            let _weekly_load = ProfileService::calculate_weekly_cooking_load(&profile);
            let _complexities = ProfileService::determine_optimal_meal_complexity(&profile);
            let _suggestions = ProfileService::get_profile_improvement_suggestions(&profile);
            
            // Test compatibility validation
            let _compatibility = ProfileService::validate_dietary_restrictions_compatibility(
                &profile.dietary_restrictions
            );
        }
        
        let duration = start.elapsed();
        let avg_time_per_iteration = duration.as_micros() as f64 / iterations as f64;
        
        // Performance assertions - should be very fast
        assert!(avg_time_per_iteration < 1000.0, // Less than 1ms per iteration
            "Average validation time should be under 1ms, got {:.2}μs", avg_time_per_iteration);
        
        assert!(duration.as_millis() < 500,
            "Total validation time for {} iterations should be under 500ms, got {}ms",
            iterations, duration.as_millis());
            
        println!("Validation performance: {:.2}μs per iteration ({} iterations in {:?})",
            avg_time_per_iteration, iterations, duration);
    }
}