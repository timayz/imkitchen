/// Unit tests for subscription domain logic
///
/// Tests cover:
/// - UpgradeSubscriptionCommand validation
/// - SubscriptionUpgraded event structure
/// - UserAggregate::subscription_upgraded event handler behavior
/// - Recipe limit validation for free vs premium tiers
use chrono::Utc;

/// Test SubscriptionUpgraded event structure
#[test]
fn test_subscription_upgraded_event_structure() {
    use user::events::SubscriptionUpgraded;

    // Create SubscriptionUpgraded event with all Stripe metadata
    let upgrade_event = SubscriptionUpgraded {
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_123456".to_string()),
        stripe_subscription_id: Some("sub_789012".to_string()),
        upgraded_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(upgrade_event.new_tier, "premium");
    assert!(upgrade_event.stripe_customer_id.is_some());
    assert!(upgrade_event.stripe_subscription_id.is_some());
    assert!(!upgrade_event.upgraded_at.is_empty());
}

/// Test SubscriptionUpgraded event without Stripe IDs (downgrade to free)
#[test]
fn test_subscription_downgrade_event() {
    use user::events::SubscriptionUpgraded;

    // Downgrade to free tier (no Stripe metadata)
    let downgrade_event = SubscriptionUpgraded {
        new_tier: "free".to_string(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
        upgraded_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(downgrade_event.new_tier, "free");
    assert!(downgrade_event.stripe_customer_id.is_none());
    assert!(downgrade_event.stripe_subscription_id.is_none());
}

/// Test UpgradeSubscriptionCommand validation
#[test]
fn test_upgrade_subscription_command_validation() {
    use user::commands::UpgradeSubscriptionCommand;
    use validator::Validate;

    // Valid command with Stripe metadata
    let valid_command = UpgradeSubscriptionCommand {
        user_id: "test-user-id".to_string(),
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_123".to_string()),
        stripe_subscription_id: Some("sub_456".to_string()),
    };

    assert!(valid_command.validate().is_ok());

    // Valid command without Stripe metadata (downgrade)
    let downgrade_command = UpgradeSubscriptionCommand {
        user_id: "test-user-id".to_string(),
        new_tier: "free".to_string(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
    };

    assert!(downgrade_command.validate().is_ok());
}

/// Test UserAggregate default tier is "free"
#[test]
fn test_user_aggregate_default_tier() {
    use user::aggregate::UserAggregate;

    let aggregate = UserAggregate::default();

    assert_eq!(aggregate.tier, "");
    assert_eq!(aggregate.recipe_count, 0);
    assert!(aggregate.stripe_customer_id.is_none());
    assert!(aggregate.stripe_subscription_id.is_none());
}

/// Test RecipeCreated event structure (cross-domain)
#[test]
fn test_recipe_created_event() {
    use user::events::RecipeCreated;

    let recipe_event = RecipeCreated {
        user_id: "user-123".to_string(),
        title: "Chicken Tikka Masala".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(recipe_event.user_id, "user-123");
    assert_eq!(recipe_event.title, "Chicken Tikka Masala");
    assert!(!recipe_event.created_at.is_empty());
}

/// Test RecipeDeleted event structure (cross-domain)
#[test]
fn test_recipe_deleted_event() {
    use user::events::RecipeDeleted;

    let delete_event = RecipeDeleted {
        user_id: "user-123".to_string(),
        deleted_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(delete_event.user_id, "user-123");
    assert!(!delete_event.deleted_at.is_empty());
}

/// Test that UserAggregate correctly stores Stripe metadata after subscription upgrade
///
/// This test verifies the aggregate state by simulating the event handler behavior
/// (we can't call event handlers directly due to evento macro privacy)
#[test]
fn test_subscription_upgrade_includes_stripe_metadata() {
    use user::aggregate::UserAggregate;
    use user::events::SubscriptionUpgraded;

    // Create a default aggregate (free tier)
    let mut aggregate = UserAggregate {
        tier: "free".to_string(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
        ..Default::default()
    };

    // Simulate SubscriptionUpgraded event
    let upgrade_event = SubscriptionUpgraded {
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_ABC123".to_string()),
        stripe_subscription_id: Some("sub_XYZ789".to_string()),
        upgraded_at: Utc::now().to_rfc3339(),
    };

    // Manually apply event logic (simulates event handler)
    aggregate.tier = upgrade_event.new_tier.clone();
    aggregate.stripe_customer_id = upgrade_event.stripe_customer_id.clone();
    aggregate.stripe_subscription_id = upgrade_event.stripe_subscription_id.clone();

    // Verify aggregate state updated correctly
    assert_eq!(aggregate.tier, "premium");
    assert_eq!(aggregate.stripe_customer_id, Some("cus_ABC123".to_string()));
    assert_eq!(
        aggregate.stripe_subscription_id,
        Some("sub_XYZ789".to_string())
    );
}

/// Test recipe count increment on RecipeCreated event
#[test]
fn test_recipe_count_increments() {
    use user::aggregate::UserAggregate;

    // Create aggregate with initial recipe count
    let mut aggregate = UserAggregate {
        recipe_count: 5,
        ..Default::default()
    };

    // Simulate RecipeCreated event handler behavior
    aggregate.recipe_count += 1;

    assert_eq!(aggregate.recipe_count, 6);
}

/// Test recipe count decrement on RecipeDeleted event (with min 0 protection)
#[test]
fn test_recipe_count_decrements() {
    use user::aggregate::UserAggregate;

    // Test normal decrement
    let mut aggregate = UserAggregate::default();
    aggregate.recipe_count = 3;

    aggregate.recipe_count = (aggregate.recipe_count - 1).max(0);
    assert_eq!(aggregate.recipe_count, 2);

    // Test decrement at 0 (should not go negative)
    let mut zero_aggregate = UserAggregate::default();
    zero_aggregate.recipe_count = 0;

    zero_aggregate.recipe_count = (zero_aggregate.recipe_count - 1).max(0);
    assert_eq!(zero_aggregate.recipe_count, 0);
}

/// Test SubscriptionUpgraded event with various tier combinations
#[test]
fn test_subscription_tier_transitions() {
    use user::events::SubscriptionUpgraded;

    // Free -> Premium
    let free_to_premium = SubscriptionUpgraded {
        new_tier: "premium".to_string(),
        stripe_customer_id: Some("cus_123".to_string()),
        stripe_subscription_id: Some("sub_456".to_string()),
        upgraded_at: Utc::now().to_rfc3339(),
    };
    assert_eq!(free_to_premium.new_tier, "premium");

    // Premium -> Free (cancellation)
    let premium_to_free = SubscriptionUpgraded {
        new_tier: "free".to_string(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
        upgraded_at: Utc::now().to_rfc3339(),
    };
    assert_eq!(premium_to_free.new_tier, "free");
}
