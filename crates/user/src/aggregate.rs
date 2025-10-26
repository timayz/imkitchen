use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, NotificationPermissionChanged, PasswordChanged,
    ProfileCompleted, ProfileUpdated, RecipeCreated, RecipeDeleted, RecipeShared,
    SubscriptionUpgraded, UserCreated, UserMealPlanningPreferencesUpdated,
    WeeknightAvailabilitySet,
};
use crate::types::UserPreferences;

/// User aggregate representing the state of a user entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// Fields follow the tech spec requirements including user profile, tier, and Stripe integration.
///
/// Note: user_id, created_at stored as String for bincode compatibility
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct UserAggregate {
    // Core identity
    pub user_id: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: String, // RFC3339 formatted timestamp

    // Profile fields (from tech spec)
    pub dietary_restrictions: Vec<String>,
    pub household_size: Option<u8>,
    pub weeknight_availability: Option<String>, // JSON: {"start":"18:00","duration_minutes":45}
    pub onboarding_completed: bool,

    // Subscription management
    pub tier: String, // "free", "premium"
    pub recipe_count: i32,

    // Stripe integration fields
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,

    // Notification permission fields (Story 4.10)
    pub notification_permission_status: String, // "not_asked", "granted", "denied", "skipped"
    pub last_permission_denial_at: Option<String>, // RFC3339 timestamp for grace period tracking

    // Meal planning preferences (Story 6.4)
    pub preferences: UserPreferences,
}

/// Implement evento aggregator pattern for UserAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl UserAggregate {
    /// Handle UserCreated event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    async fn user_created(
        &mut self,
        event: evento::EventDetails<UserCreated>,
    ) -> anyhow::Result<()> {
        let user_id = event.aggregator_id.clone();
        self.user_id = user_id.clone();
        self.email = event.data.email;
        self.password_hash = event.data.password_hash;
        self.created_at = event.data.created_at;
        self.tier = "free".to_string();
        self.recipe_count = 0;
        self.dietary_restrictions = Vec::new();
        self.household_size = None;
        self.weeknight_availability = None;
        self.onboarding_completed = false;
        self.stripe_customer_id = None;
        self.stripe_subscription_id = None;
        self.notification_permission_status = "not_asked".to_string();
        self.last_permission_denial_at = None;
        self.preferences = UserPreferences {
            user_id,
            ..Default::default()
        };
        Ok(())
    }

    /// Handle PasswordChanged event to update password hash
    ///
    /// This event handler updates the aggregate state when a user successfully
    /// resets their password. The old password is invalidated automatically
    /// by replacing the password_hash field.
    async fn password_changed(
        &mut self,
        event: evento::EventDetails<PasswordChanged>,
    ) -> anyhow::Result<()> {
        self.password_hash = event.data.password_hash;
        Ok(())
    }

    /// Handle DietaryRestrictionsSet event (Step 1)
    async fn dietary_restrictions_set(
        &mut self,
        event: evento::EventDetails<DietaryRestrictionsSet>,
    ) -> anyhow::Result<()> {
        self.dietary_restrictions = event.data.dietary_restrictions;
        Ok(())
    }

    /// Handle HouseholdSizeSet event (Step 2)
    async fn household_size_set(
        &mut self,
        event: evento::EventDetails<HouseholdSizeSet>,
    ) -> anyhow::Result<()> {
        self.household_size = Some(event.data.household_size);
        Ok(())
    }

    /// Handle WeeknightAvailabilitySet event (Step 3)
    async fn weeknight_availability_set(
        &mut self,
        event: evento::EventDetails<WeeknightAvailabilitySet>,
    ) -> anyhow::Result<()> {
        self.weeknight_availability = Some(event.data.weeknight_availability);
        Ok(())
    }

    /// Handle ProfileCompleted event - marks onboarding as complete
    ///
    /// This is emitted after all step events, simply marking onboarding as done.
    async fn profile_completed(
        &mut self,
        _event: evento::EventDetails<ProfileCompleted>,
    ) -> anyhow::Result<()> {
        self.onboarding_completed = true;
        Ok(())
    }

    /// Handle ProfileUpdated event - updates profile fields with COALESCE logic
    ///
    /// This event supports partial updates. Only non-None fields are updated,
    /// preserving existing values for None fields (COALESCE behavior).
    /// Used for post-onboarding profile editing.
    async fn profile_updated(
        &mut self,
        event: evento::EventDetails<ProfileUpdated>,
    ) -> anyhow::Result<()> {
        // COALESCE logic: only update non-None fields
        if let Some(dietary_restrictions) = event.data.dietary_restrictions {
            self.dietary_restrictions = dietary_restrictions;
        }
        if let Some(household_size) = event.data.household_size {
            self.household_size = Some(household_size);
        }
        if let Some(weeknight_availability) = event.data.weeknight_availability {
            self.weeknight_availability = Some(weeknight_availability);
        }
        Ok(())
    }

    /// Handle RecipeCreated event (cross-domain) - increment recipe_count
    ///
    /// This handler is called when a recipe is created in the recipe domain.
    /// It increments the recipe_count to track freemium tier limits (10 recipe max for free tier).
    async fn recipe_created(
        &mut self,
        _event: evento::EventDetails<RecipeCreated>,
    ) -> anyhow::Result<()> {
        self.recipe_count += 1;
        Ok(())
    }

    /// Handle RecipeDeleted event (cross-domain) - decrement recipe_count
    ///
    /// This handler is called when a recipe is deleted in the recipe domain.
    /// It decrements the recipe_count to free up a slot for free tier users.
    async fn recipe_deleted(
        &mut self,
        _event: evento::EventDetails<RecipeDeleted>,
    ) -> anyhow::Result<()> {
        self.recipe_count = (self.recipe_count - 1).max(0); // Prevent negative counts
        Ok(())
    }

    /// Handle RecipeShared event (cross-domain) - adjust recipe_count
    ///
    /// This handler is called when a recipe is shared/unshared in the recipe domain.
    /// Shared recipes do NOT count toward the freemium limit.
    /// - When shared=true: decrement count (recipe no longer counts toward limit)
    /// - When shared=false: increment count (recipe now counts toward limit)
    async fn recipe_shared(
        &mut self,
        event: evento::EventDetails<RecipeShared>,
    ) -> anyhow::Result<()> {
        if event.data.shared {
            // Recipe was shared - decrement count (shared recipes don't count)
            self.recipe_count = (self.recipe_count - 1).max(0);
        } else {
            // Recipe was unshared - increment count (now counts toward limit)
            self.recipe_count += 1;
        }
        Ok(())
    }

    /// Handle SubscriptionUpgraded event - update subscription tier and Stripe metadata
    ///
    /// This handler is called when a user upgrades or downgrades their subscription tier.
    /// It updates the tier field and stores Stripe Customer ID and Subscription ID for
    /// future subscription management (cancellation, billing updates).
    async fn subscription_upgraded(
        &mut self,
        event: evento::EventDetails<SubscriptionUpgraded>,
    ) -> anyhow::Result<()> {
        self.tier = event.data.new_tier;
        self.stripe_customer_id = event.data.stripe_customer_id;
        self.stripe_subscription_id = event.data.stripe_subscription_id;
        Ok(())
    }

    /// Handle NotificationPermissionChanged event - update notification permission status
    ///
    /// This handler is called when a user grants, denies, or skips notification permission.
    /// It tracks the permission status and denial timestamp for grace period enforcement (AC #8).
    async fn notification_permission_changed(
        &mut self,
        event: evento::EventDetails<NotificationPermissionChanged>,
    ) -> anyhow::Result<()> {
        self.notification_permission_status = event.data.permission_status;
        self.last_permission_denial_at = event.data.last_permission_denial_at;
        Ok(())
    }

    /// Handle UserMealPlanningPreferencesUpdated event - update meal planning preferences
    ///
    /// This handler is called when a user updates their meal planning preferences.
    /// It deserializes JSON fields (dietary_restrictions, weeknight_availability) and updates
    /// the UserPreferences struct within the aggregate.
    ///
    /// Story 6.4 - AC #6: User aggregate integrates preferences
    async fn user_meal_planning_preferences_updated(
        &mut self,
        event: evento::EventDetails<UserMealPlanningPreferencesUpdated>,
    ) -> anyhow::Result<()> {
        use crate::types::{DietaryRestriction, SkillLevel, TimeRange};

        // Deserialize JSON fields
        let dietary_restrictions: Vec<DietaryRestriction> =
            serde_json::from_str(&event.data.dietary_restrictions)?;
        let skill_level: SkillLevel =
            serde_json::from_str(&format!("\"{}\"", event.data.skill_level))?;
        let weeknight_availability: TimeRange =
            serde_json::from_str(&event.data.weeknight_availability)?;

        // Update preferences
        self.preferences.dietary_restrictions = dietary_restrictions;
        self.preferences.household_size = event.data.household_size;
        self.preferences.skill_level = skill_level;
        self.preferences.weeknight_availability = weeknight_availability;
        self.preferences.max_prep_time_weeknight = event.data.max_prep_time_weeknight;
        self.preferences.max_prep_time_weekend = event.data.max_prep_time_weekend;
        self.preferences.avoid_consecutive_complex = event.data.avoid_consecutive_complex;
        self.preferences.cuisine_variety_weight = event.data.cuisine_variety_weight;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::UserMealPlanningPreferencesUpdated;
    use crate::types::{DietaryRestriction, SkillLevel, TimeRange};
    use sqlx::SqlitePool;

    /// Helper to create evento executor for testing
    async fn create_test_executor() -> anyhow::Result<evento::Sqlite> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;

        // Run evento migrations for event store tables
        use evento::migrator::{Migrate, Plan};
        let mut conn = pool.acquire().await?;
        evento::sql_migrator::new_migrator::<sqlx::Sqlite>()?
            .run(&mut *conn, &Plan::apply_all())
            .await?;
        drop(conn);

        Ok(pool.into())
    }

    #[tokio::test]
    async fn test_user_created_with_default_preferences() -> anyhow::Result<()> {
        let executor = create_test_executor().await?;

        // Create user via evento
        let user_id = evento::create::<UserAggregate>()
            .metadata(&true)?
            .data(&UserCreated {
                email: "test@example.com".to_string(),
                password_hash: "hash123".to_string(),
                created_at: "2025-10-25T12:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Load user and verify default preferences
        let user = evento::load::<UserAggregate, _>(&executor, &user_id)
            .await?
            .item;

        // AC #6: User aggregate integrates preferences
        assert_eq!(user.preferences.user_id, user_id);

        // AC #8: Default values match specification
        assert_eq!(user.preferences.max_prep_time_weeknight, 30);
        assert_eq!(user.preferences.max_prep_time_weekend, 90);
        assert_eq!(user.preferences.cuisine_variety_weight, 0.7);
        assert!(user.preferences.avoid_consecutive_complex);

        Ok(())
    }

    #[tokio::test]
    async fn test_preferences_updated_via_event() -> anyhow::Result<()> {
        let executor = create_test_executor().await?;

        // Create user
        let user_id = evento::create::<UserAggregate>()
            .metadata(&true)?
            .data(&UserCreated {
                email: "test2@example.com".to_string(),
                password_hash: "hash456".to_string(),
                created_at: "2025-10-25T12:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Update preferences
        let dietary_restrictions = vec![
            DietaryRestriction::Vegetarian,
            DietaryRestriction::Custom("shellfish".to_string()),
        ];
        let dietary_json = serde_json::to_string(&dietary_restrictions)?;

        let weeknight_availability = TimeRange {
            start: "19:00".to_string(),
            duration_minutes: 60,
        };
        let weeknight_json = serde_json::to_string(&weeknight_availability)?;

        evento::save::<UserAggregate>(&user_id)
            .metadata(&true)?
            .data(&UserMealPlanningPreferencesUpdated {
                dietary_restrictions: dietary_json,
                household_size: 4,
                skill_level: "Advanced".to_string(),
                weeknight_availability: weeknight_json,
                max_prep_time_weeknight: 45,
                max_prep_time_weekend: 120,
                avoid_consecutive_complex: false,
                cuisine_variety_weight: 0.5,
                updated_at: "2025-10-25T13:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Load user and verify all fields updated
        let user = evento::load::<UserAggregate, _>(&executor, &user_id)
            .await?
            .item;

        // AC #7: Test updating preferences via event - verify all fields updated
        assert_eq!(user.preferences.dietary_restrictions.len(), 2);
        assert_eq!(
            user.preferences.dietary_restrictions[0],
            DietaryRestriction::Vegetarian
        );
        assert_eq!(user.preferences.household_size, 4);
        assert_eq!(user.preferences.skill_level, SkillLevel::Advanced);
        assert_eq!(user.preferences.weeknight_availability.start, "19:00");
        assert_eq!(user.preferences.weeknight_availability.duration_minutes, 60);
        assert_eq!(user.preferences.max_prep_time_weeknight, 45);
        assert_eq!(user.preferences.max_prep_time_weekend, 120);
        assert!(!user.preferences.avoid_consecutive_complex);
        assert_eq!(user.preferences.cuisine_variety_weight, 0.5);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_preference_updates_preserve_user_id() -> anyhow::Result<()> {
        let executor = create_test_executor().await?;

        // Create user
        let user_id = evento::create::<UserAggregate>()
            .metadata(&true)?
            .data(&UserCreated {
                email: "test3@example.com".to_string(),
                password_hash: "hash789".to_string(),
                created_at: "2025-10-25T12:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // First update
        evento::save::<UserAggregate>(&user_id)
            .metadata(&true)?
            .data(&UserMealPlanningPreferencesUpdated {
                dietary_restrictions: "[]".to_string(),
                household_size: 2,
                skill_level: "Beginner".to_string(),
                weeknight_availability: r#"{"start":"18:00","duration_minutes":30}"#.to_string(),
                max_prep_time_weeknight: 20,
                max_prep_time_weekend: 60,
                avoid_consecutive_complex: true,
                cuisine_variety_weight: 0.3,
                updated_at: "2025-10-25T13:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Second update
        evento::save::<UserAggregate>(&user_id)
            .metadata(&true)?
            .data(&UserMealPlanningPreferencesUpdated {
                dietary_restrictions: r#"[{"type":"Vegan"}]"#.to_string(),
                household_size: 3,
                skill_level: "Intermediate".to_string(),
                weeknight_availability: r#"{"start":"18:30","duration_minutes":45}"#.to_string(),
                max_prep_time_weeknight: 35,
                max_prep_time_weekend: 100,
                avoid_consecutive_complex: false,
                cuisine_variety_weight: 0.8,
                updated_at: "2025-10-25T14:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Load user and verify final state
        let user = evento::load::<UserAggregate, _>(&executor, &user_id)
            .await?
            .item;

        // AC #7: Multiple preference updates preserve user_id
        assert_eq!(user.preferences.user_id, user_id);
        assert_eq!(user.preferences.household_size, 3);
        assert_eq!(user.preferences.skill_level, SkillLevel::Intermediate);
        assert_eq!(user.preferences.cuisine_variety_weight, 0.8);

        Ok(())
    }

    #[tokio::test]
    async fn test_custom_dietary_restriction_serialization() -> anyhow::Result<()> {
        let executor = create_test_executor().await?;

        let user_id = evento::create::<UserAggregate>()
            .metadata(&true)?
            .data(&UserCreated {
                email: "test4@example.com".to_string(),
                password_hash: "hash101".to_string(),
                created_at: "2025-10-25T12:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        // Update with custom dietary restriction
        let custom_restriction = DietaryRestriction::Custom("soy allergy".to_string());
        let restrictions = vec![custom_restriction];
        let dietary_json = serde_json::to_string(&restrictions)?;

        evento::save::<UserAggregate>(&user_id)
            .metadata(&true)?
            .data(&UserMealPlanningPreferencesUpdated {
                dietary_restrictions: dietary_json,
                household_size: 1,
                skill_level: "Beginner".to_string(),
                weeknight_availability: r#"{"start":"18:00","duration_minutes":45}"#.to_string(),
                max_prep_time_weeknight: 30,
                max_prep_time_weekend: 90,
                avoid_consecutive_complex: true,
                cuisine_variety_weight: 0.7,
                updated_at: "2025-10-25T13:00:00Z".to_string(),
            })?
            .commit(&executor)
            .await?;

        let user = evento::load::<UserAggregate, _>(&executor, &user_id)
            .await?
            .item;

        // AC #7: Custom dietary restriction serialization
        assert_eq!(user.preferences.dietary_restrictions.len(), 1);
        match &user.preferences.dietary_restrictions[0] {
            DietaryRestriction::Custom(s) => assert_eq!(s, "soy allergy"),
            _ => panic!("Expected Custom variant"),
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_event_serialization() -> anyhow::Result<()> {
        // AC #5: Test UserMealPlanningPreferencesUpdated event serialization
        let event = UserMealPlanningPreferencesUpdated {
            dietary_restrictions: r#"[{"type":"GlutenFree"}]"#.to_string(),
            household_size: 2,
            skill_level: "Intermediate".to_string(),
            weeknight_availability: r#"{"start":"18:00","duration_minutes":45}"#.to_string(),
            max_prep_time_weeknight: 30,
            max_prep_time_weekend: 90,
            avoid_consecutive_complex: true,
            cuisine_variety_weight: 0.7,
            updated_at: "2025-10-25T13:00:00Z".to_string(),
        };

        // Test serde JSON serialization
        let json = serde_json::to_string(&event)?;
        let decoded: UserMealPlanningPreferencesUpdated = serde_json::from_str(&json)?;
        assert_eq!(decoded.household_size, 2);
        assert_eq!(decoded.max_prep_time_weeknight, 30);

        // Test bincode serialization
        let encoded = bincode::encode_to_vec(&event, bincode::config::standard())?;
        let (decoded, _): (UserMealPlanningPreferencesUpdated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard())?;
        assert_eq!(decoded.cuisine_variety_weight, 0.7);

        Ok(())
    }
}
