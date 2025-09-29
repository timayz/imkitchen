// Basic domain model tests for User aggregate and value objects

use imkitchen_shared::{Email, FamilySize, Password, SkillLevel};
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
