use imkitchen_shared::{Email, FamilySize, SkillLevel, Difficulty};

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