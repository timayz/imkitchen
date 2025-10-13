use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{PasswordChanged, UserCreated};

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
    pub skill_level: Option<String>, // "beginner", "intermediate", "advanced"

    // Subscription management
    pub tier: String, // "free", "premium"
    pub recipe_count: i32,

    // Stripe integration fields
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
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
        self.stripe_customer_id = None;
        self.stripe_subscription_id = None;
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
}
