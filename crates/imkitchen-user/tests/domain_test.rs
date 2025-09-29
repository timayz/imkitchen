// Basic domain model tests for User aggregate and value objects

use imkitchen_shared::{DietaryRestriction, Email, FamilySize, Password, SkillLevel};
use imkitchen_user::domain::{User, UserProfile};

#[test]
fn test_create_user_with_valid_data() {
    // Test the basic User creation functionality
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile {
        family_size: FamilySize::new(4).unwrap(),
        cooking_skill_level: SkillLevel::Intermediate,
        dietary_restrictions: vec![],
        weekday_cooking_minutes: 45,
        weekend_cooking_minutes: 90,
    };

    let (user, _event) = User::new(email.clone(), password, profile.clone());

    assert_eq!(user.email.value, "test@example.com");
    assert!(!user.is_email_verified);
    assert_eq!(user.profile.family_size.value, 4);
    assert!(matches!(
        user.profile.cooking_skill_level,
        SkillLevel::Intermediate
    ));
}

#[test]
fn test_user_login_requires_email_verification() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (user, _event) = User::new(email, password, profile);

    // Login should fail because email is not verified
    let result = user.login();
    assert!(result.is_err());
}

#[test]
fn test_user_login_succeeds_when_email_verified() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (mut user, _event) = User::new(email, password, profile);
    user.verify_email();

    // Login should succeed now
    let result = user.login();
    assert!(result.is_ok());
}

#[test]
fn test_password_validation() {
    // Valid password
    let valid_password = Password::new("StrongPass123!".to_string());
    assert!(valid_password.is_ok());

    // Too short password
    let short_password = Password::new("Pass1!".to_string());
    assert!(short_password.is_err());

    // Password without special character
    let no_special = Password::new("Password123".to_string());
    assert!(no_special.is_err());
}

#[test]
fn test_email_validation() {
    // Valid email
    let valid_email = Email::new("user@domain.com".to_string());
    assert!(valid_email.is_ok());

    // Invalid email
    let invalid_email = Email::new("not-an-email".to_string());
    assert!(invalid_email.is_err());
}

#[test]
fn test_family_size_validation() {
    // Valid family sizes
    assert!(FamilySize::new(1).is_ok());
    assert!(FamilySize::new(4).is_ok());
    assert!(FamilySize::new(8).is_ok());

    // Invalid family sizes
    assert!(FamilySize::new(0).is_err());
    assert!(FamilySize::new(9).is_err());
}

#[test]
fn test_update_dietary_restrictions() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (mut user, _event) = User::new(email, password, profile);
    
    let restrictions = vec![
        DietaryRestriction::Vegetarian,
        DietaryRestriction::GlutenFree,
    ];
    
    let update_event = user.update_dietary_restrictions(restrictions.clone());
    
    assert_eq!(user.profile.dietary_restrictions, restrictions);
    assert_eq!(update_event.user_id, user.user_id);
    assert_eq!(update_event.new_restrictions, restrictions);
    assert!(update_event.has_changes());
}

#[test]
fn test_update_family_size() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (mut user, _event) = User::new(email, password, profile);
    
    let new_family_size = FamilySize::new(6).unwrap();
    let update_event = user.update_family_size(new_family_size);
    
    assert_eq!(user.profile.family_size, new_family_size);
    assert_eq!(update_event.user_id, user.user_id);
    assert_eq!(update_event.new_size, new_family_size);
    assert!(update_event.has_changes());
}

#[test]
fn test_update_skill_level() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (mut user, _event) = User::new(email, password, profile);
    
    let new_skill = SkillLevel::Advanced;
    let update_event = user.update_skill_level(new_skill);
    
    assert_eq!(user.profile.cooking_skill_level, new_skill);
    assert_eq!(update_event.user_id, user.user_id);
    assert_eq!(update_event.profile.cooking_skill_level, new_skill);
}

#[test]
fn test_update_cooking_time() {
    let email = Email::new("test@example.com".to_string()).unwrap();
    let password = Password::new("TestPass123!".to_string()).unwrap();
    let profile = UserProfile::default();

    let (mut user, _event) = User::new(email, password, profile);
    
    let weekday_minutes = 45;
    let weekend_minutes = 120;
    let update_event = user.update_cooking_time(weekday_minutes, weekend_minutes);
    
    assert_eq!(user.profile.weekday_cooking_minutes, weekday_minutes);
    assert_eq!(user.profile.weekend_cooking_minutes, weekend_minutes);
    assert_eq!(update_event.profile.weekday_cooking_minutes, weekday_minutes);
    assert_eq!(update_event.profile.weekend_cooking_minutes, weekend_minutes);
}

#[test]
fn test_profile_completeness_validation() {
    let complete_profile = UserProfile {
        family_size: FamilySize::new(4).unwrap(),
        cooking_skill_level: SkillLevel::Intermediate,
        dietary_restrictions: vec![DietaryRestriction::Vegetarian],
        weekday_cooking_minutes: 30,
        weekend_cooking_minutes: 60,
    };
    
    assert!(complete_profile.is_complete_for_meal_planning());
    
    let incomplete_profile = UserProfile {
        family_size: FamilySize::new(2).unwrap(),
        cooking_skill_level: SkillLevel::Beginner,
        dietary_restrictions: vec![],
        weekday_cooking_minutes: 2, // Too short
        weekend_cooking_minutes: 3, // Too short
    };
    
    assert!(!incomplete_profile.is_complete_for_meal_planning());
}

#[test]
fn test_recommended_complexity_by_skill_level() {
    let beginner_profile = UserProfile {
        cooking_skill_level: SkillLevel::Beginner,
        ..UserProfile::default()
    };
    let beginner_complexity = beginner_profile.get_recommended_complexity();
    assert_eq!(beginner_complexity, vec!["Easy"]);
    
    let intermediate_profile = UserProfile {
        cooking_skill_level: SkillLevel::Intermediate,
        ..UserProfile::default()
    };
    let intermediate_complexity = intermediate_profile.get_recommended_complexity();
    assert_eq!(intermediate_complexity, vec!["Easy", "Medium"]);
    
    let advanced_profile = UserProfile {
        cooking_skill_level: SkillLevel::Advanced,
        ..UserProfile::default()
    };
    let advanced_complexity = advanced_profile.get_recommended_complexity();
    assert_eq!(advanced_complexity, vec!["Easy", "Medium", "Hard"]);
}

#[test]
fn test_calculate_portions() {
    let family_of_4 = UserProfile {
        family_size: FamilySize::new(4).unwrap(),
        ..UserProfile::default()
    };
    
    // Base recipe for 2 people, should scale to 4 portions
    assert_eq!(family_of_4.calculate_portions(2), 4);
    
    // Base recipe for 4 people, should remain 4 portions
    assert_eq!(family_of_4.calculate_portions(4), 4);
    
    // Base recipe for 6 people, should scale down to 4 portions
    assert_eq!(family_of_4.calculate_portions(6), 4);
    
    let family_of_1 = UserProfile {
        family_size: FamilySize::new(1).unwrap(),
        ..UserProfile::default()
    };
    
    // Base recipe for 4 people, should scale down to 1 portion
    assert_eq!(family_of_1.calculate_portions(4), 1);
}
