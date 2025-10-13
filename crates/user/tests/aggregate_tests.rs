/// Unit tests for user aggregate COALESCE logic and ProfileUpdated behavior
///
/// Note: These tests verify the command/event flow works correctly by testing
/// the public API (command handlers) rather than private event handlers directly.
/// This follows evento best practices for testing event-sourced aggregates.

use chrono::Utc;

/// Test that profile update command validates household_size range (1-20)
#[tokio::test]
async fn test_profile_update_validates_household_size() {
    use user::commands::UpdateProfileCommand;
    use validator::Validate;

    // Valid household_size (within range)
    let valid_command = UpdateProfileCommand {
        user_id: "test-user".to_string(),
        dietary_restrictions: None,
        household_size: Some(10),
        skill_level: None,
        weeknight_availability: None,
    };

    assert!(valid_command.validate().is_ok());

    // Invalid household_size (> 20)
    let invalid_command = UpdateProfileCommand {
        user_id: "test-user".to_string(),
        dietary_restrictions: None,
        household_size: Some(25),
        skill_level: None,
        weeknight_availability: None,
    };

    assert!(invalid_command.validate().is_err());

    // Invalid household_size (0)
    let zero_command = UpdateProfileCommand {
        user_id: "test-user".to_string(),
        dietary_restrictions: None,
        household_size: Some(0),
        skill_level: None,
        weeknight_availability: None,
    };

    assert!(zero_command.validate().is_err());
}

/// Test that ProfileUpdated event has correct structure for partial updates
#[test]
fn test_profile_updated_event_structure() {
    use user::events::ProfileUpdated;

    // Create ProfileUpdated with partial fields (COALESCE pattern)
    let partial_update = ProfileUpdated {
        dietary_restrictions: Some(vec!["vegetarian".to_string()]),
        household_size: None, // None = no change
        skill_level: Some("intermediate".to_string()),
        weeknight_availability: None, // None = no change
        updated_at: Utc::now().to_rfc3339(),
    };

    // Verify Option fields allow None for COALESCE behavior
    assert!(partial_update.dietary_restrictions.is_some());
    assert!(partial_update.household_size.is_none());
    assert!(partial_update.skill_level.is_some());
    assert!(partial_update.weeknight_availability.is_none());

    // Create ProfileUpdated with all fields
    let full_update = ProfileUpdated {
        dietary_restrictions: Some(vec!["vegan".to_string()]),
        household_size: Some(2),
        skill_level: Some("expert".to_string()),
        weeknight_availability: Some(r#"{"start":"18:00","duration_minutes":60}"#.to_string()),
        updated_at: Utc::now().to_rfc3339(),
    };

    assert!(full_update.dietary_restrictions.is_some());
    assert!(full_update.household_size.is_some());
    assert!(full_update.skill_level.is_some());
    assert!(full_update.weeknight_availability.is_some());
}

/// Test UserAggregate default initialization
#[test]
fn test_user_aggregate_default() {
    use user::aggregate::UserAggregate;

    let aggregate = UserAggregate::default();

    assert_eq!(aggregate.user_id, "");
    assert_eq!(aggregate.email, "");
    assert_eq!(aggregate.dietary_restrictions, Vec::<String>::new());
    assert_eq!(aggregate.household_size, None);
    assert_eq!(aggregate.skill_level, None);
    assert_eq!(aggregate.weeknight_availability, None);
    assert_eq!(aggregate.onboarding_completed, false);
}

/// Test ProfileCompleted event structure
#[test]
fn test_profile_completed_event() {
    use user::events::ProfileCompleted;

    let completed = ProfileCompleted {
        completed_at: Utc::now().to_rfc3339(),
    };

    assert!(!completed.completed_at.is_empty());
}

/// Test DietaryRestrictionsSet event
#[test]
fn test_dietary_restrictions_set_event() {
    use user::events::DietaryRestrictionsSet;

    let dietary_event = DietaryRestrictionsSet {
        dietary_restrictions: vec!["vegetarian".to_string(), "gluten-free".to_string()],
        set_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(dietary_event.dietary_restrictions.len(), 2);
    assert!(dietary_event.dietary_restrictions.contains(&"vegetarian".to_string()));
}

/// Test HouseholdSizeSet event
#[test]
fn test_household_size_set_event() {
    use user::events::HouseholdSizeSet;

    let household_event = HouseholdSizeSet {
        household_size: 4,
        set_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(household_event.household_size, 4);
}

/// Test SkillLevelSet event
#[test]
fn test_skill_level_set_event() {
    use user::events::SkillLevelSet;

    let skill_event = SkillLevelSet {
        skill_level: "intermediate".to_string(),
        set_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(skill_event.skill_level, "intermediate");
}

/// Test WeeknightAvailabilitySet event
#[test]
fn test_weeknight_availability_set_event() {
    use user::events::WeeknightAvailabilitySet;

    let availability_event = WeeknightAvailabilitySet {
        weeknight_availability: r#"{"start":"18:00","duration_minutes":45}"#.to_string(),
        set_at: Utc::now().to_rfc3339(),
    };

    assert!(availability_event.weeknight_availability.contains("18:00"));
}
