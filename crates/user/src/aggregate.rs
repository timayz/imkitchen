use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    DietaryRestrictionsSet, HouseholdSizeSet, NotificationPermissionChanged, PasswordChanged,
    ProfileCompleted, ProfileUpdated, RecipeCreated, RecipeDeleted, SkillLevelSet,
    SubscriptionUpgraded, UserCreated, WeeknightAvailabilitySet,
};

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
    pub skill_level: Option<String>, // "beginner", "intermediate", "expert"
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
        self.user_id = event.aggregator_id.clone();
        self.email = event.data.email;
        self.password_hash = event.data.password_hash;
        self.created_at = event.data.created_at;
        self.tier = "free".to_string();
        self.recipe_count = 0;
        self.dietary_restrictions = Vec::new();
        self.household_size = None;
        self.skill_level = None;
        self.weeknight_availability = None;
        self.onboarding_completed = false;
        self.stripe_customer_id = None;
        self.stripe_subscription_id = None;
        self.notification_permission_status = "not_asked".to_string();
        self.last_permission_denial_at = None;
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

    /// Handle SkillLevelSet event (Step 3)
    async fn skill_level_set(
        &mut self,
        event: evento::EventDetails<SkillLevelSet>,
    ) -> anyhow::Result<()> {
        self.skill_level = Some(event.data.skill_level);
        Ok(())
    }

    /// Handle WeeknightAvailabilitySet event (Step 4)
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
        if let Some(skill_level) = event.data.skill_level {
            self.skill_level = Some(skill_level);
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
}
