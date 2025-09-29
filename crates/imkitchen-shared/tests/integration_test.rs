use imkitchen_shared::{Difficulty, Email, FamilySize, SkillLevel, DietaryRestriction};

#[test]
fn test_valid_email_creation() {
    let email = Email::new("user@example.com".to_string());
    assert!(email.is_ok());
    assert_eq!(email.unwrap().value, "user@example.com");
}

#[test]
fn test_invalid_email_creation() {
    let email = Email::new("invalid-email".to_string());
    assert!(email.is_err());
}

#[test]
fn test_valid_family_size_creation() {
    for size in 1..=8 {
        let family_size = FamilySize::new(size);
        assert!(family_size.is_ok());
        assert_eq!(family_size.unwrap().value, size);
    }
}

#[test]
fn test_family_size_validation_errors() {
    // Test too small
    let family_size = FamilySize::new(0);
    assert!(family_size.is_err());

    // Test too large
    let family_size = FamilySize::new(9);
    assert!(family_size.is_err());
}

#[test]
fn test_skill_level_serialization() {
    let beginner = SkillLevel::Beginner;
    let json = serde_json::to_string(&beginner).unwrap();
    assert!(json.contains("Beginner"));
}

#[test]
fn test_difficulty_serialization() {
    let easy = Difficulty::Easy;
    let json = serde_json::to_string(&easy).unwrap();
    assert!(json.contains("Easy"));
}

#[test]
fn test_dietary_restriction_serialization() {
    let vegetarian = DietaryRestriction::Vegetarian;
    let json = serde_json::to_string(&vegetarian).unwrap();
    assert!(json.contains("Vegetarian"));

    let vegan = DietaryRestriction::Vegan;
    let json = serde_json::to_string(&vegan).unwrap();
    assert!(json.contains("Vegan"));
}

#[test]
fn test_dietary_restriction_all_variants() {
    let restrictions = vec![
        DietaryRestriction::Vegetarian,
        DietaryRestriction::Vegan,
        DietaryRestriction::GlutenFree,
        DietaryRestriction::DairyFree,
        DietaryRestriction::NutFree,
        DietaryRestriction::SoyFree,
        DietaryRestriction::LowSodium,
        DietaryRestriction::LowCarb,
        DietaryRestriction::Keto,
        DietaryRestriction::Paleo,
    ];
    
    // All should serialize and deserialize correctly
    for restriction in restrictions {
        let json = serde_json::to_string(&restriction).unwrap();
        let deserialized: DietaryRestriction = serde_json::from_str(&json).unwrap();
        assert_eq!(restriction, deserialized);
    }
}

#[test]
fn test_family_size_constants() {
    assert_eq!(FamilySize::FAMILY1.value, 1);
    assert_eq!(FamilySize::FAMILY2.value, 2);
    assert_eq!(FamilySize::FAMILY3.value, 3);
    assert_eq!(FamilySize::FAMILY4.value, 4);
    assert_eq!(FamilySize::FAMILY5.value, 5);
    assert_eq!(FamilySize::FAMILY6.value, 6);
    assert_eq!(FamilySize::FAMILY7.value, 7);
    assert_eq!(FamilySize::FAMILY8.value, 8);
}

#[test]
fn test_skill_level_display() {
    assert_eq!(SkillLevel::Beginner.to_string(), "Beginner");
    assert_eq!(SkillLevel::Intermediate.to_string(), "Intermediate");
    assert_eq!(SkillLevel::Advanced.to_string(), "Advanced");
}
