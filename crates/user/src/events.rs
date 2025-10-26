use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// UserCreated event emitted when a new user registers
///
/// This event is the source of truth for user creation in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct UserCreated {
    pub email: String,
    pub password_hash: String,
    pub created_at: String, // RFC3339 formatted timestamp
}

/// PasswordChanged event emitted when a user successfully resets their password
///
/// This event records password changes in the audit trail. The old password is NOT stored
/// for security reasons - only the new hashed password is recorded.
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct PasswordChanged {
    pub password_hash: String, // New Argon2 hashed password
    pub changed_at: String,    // RFC3339 formatted timestamp
}

/// DietaryRestrictionsSet event emitted when user sets dietary restrictions (Step 1)
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct DietaryRestrictionsSet {
    pub dietary_restrictions: Vec<String>, // e.g., ["vegetarian", "gluten-free", "peanuts"]
    pub set_at: String,                    // RFC3339 formatted timestamp
}

/// HouseholdSizeSet event emitted when user sets household size (Step 2)
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct HouseholdSizeSet {
    pub household_size: u8, // 1-10 people
    pub set_at: String,     // RFC3339 formatted timestamp
}

/// WeeknightAvailabilitySet event emitted when user sets weeknight availability (Step 3)
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct WeeknightAvailabilitySet {
    pub weeknight_availability: String, // JSON: {"start":"18:00","duration_minutes":45}
    pub set_at: String,                 // RFC3339 formatted timestamp
}

/// ProfileCompleted event emitted when a user completes onboarding
///
/// This event marks the onboarding as complete. Individual profile data is already
/// set via the step events above.
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ProfileCompleted {
    pub completed_at: String, // RFC3339 formatted timestamp
}

/// ProfileUpdated event emitted when a user updates their profile
///
/// This event supports partial updates - only changed fields are included (Option types).
/// Used for post-onboarding profile editing. Includes timestamp for audit trail (AC-7).
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ProfileUpdated {
    pub dietary_restrictions: Option<Vec<String>>, // None = no change
    pub household_size: Option<u8>,                // None = no change
    pub weeknight_availability: Option<String>, // None = no change, JSON: {"start":"18:00","duration_minutes":45}
    pub updated_at: String,                     // RFC3339 formatted timestamp
}

/// RecipeCreated event (cross-domain event from recipe domain)
///
/// User domain listens to this event to increment recipe_count for freemium enforcement.
/// This event is emitted by the recipe domain when a user creates a new recipe.
///
/// Note: user_id stored in metadata, recipe_id in aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeCreated {
    pub user_id: String,    // ID of the user who created the recipe
    pub title: String,      // Recipe title for audit
    pub created_at: String, // RFC3339 formatted timestamp
}

/// RecipeDeleted event (cross-domain event from recipe domain)
///
/// User domain listens to this event to decrement recipe_count for freemium enforcement.
/// This event is emitted by the recipe domain when a user deletes a recipe.
///
/// Note: user_id stored in metadata, recipe_id in aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeDeleted {
    pub user_id: String,    // ID of the user who deleted the recipe
    pub deleted_at: String, // RFC3339 formatted timestamp
}

/// RecipeShared event (cross-domain event from recipe domain)
///
/// User domain listens to this event to decrement recipe_count when a recipe is shared.
/// Shared recipes do NOT count toward the freemium limit.
/// This event is emitted by the recipe domain when a user shares a recipe.
///
/// Note: user_id stored in event data, recipe_id in aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeShared {
    pub user_id: String,   // ID of the user who shared the recipe
    pub shared: bool,      // true = shared (decrement count), false = unshared (increment count)
    pub shared_at: String, // RFC3339 formatted timestamp
}

/// RecipeFavorited event (cross-domain event from recipe domain)
///
/// User domain listens to this event to update favorite_count for performance optimization.
/// This event is emitted by the recipe domain when a user toggles favorite status on a recipe.
///
/// Note: user_id stored in event data, recipe_id in aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeFavorited {
    pub user_id: String,    // ID of the user who favorited the recipe
    pub favorited: bool,    // true = favorited, false = unfavorited
    pub toggled_at: String, // RFC3339 formatted timestamp
}

/// SubscriptionUpgraded event emitted when a user upgrades/downgrades subscription tier
///
/// This event captures subscription changes including Stripe metadata for future management.
/// Emitted when Stripe webhook confirms successful payment or subscription update.
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct SubscriptionUpgraded {
    pub new_tier: String,                       // "free" or "premium"
    pub stripe_customer_id: Option<String>,     // Stripe Customer ID for billing management
    pub stripe_subscription_id: Option<String>, // Stripe Subscription ID for cancellation
    pub upgraded_at: String,                    // RFC3339 formatted timestamp
}

/// NotificationPermissionChanged event emitted when user changes notification permission
///
/// This event tracks notification permission status and denial timestamp for grace period (AC #8).
/// Emitted when user grants, denies, or skips notification permission in onboarding or settings.
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct NotificationPermissionChanged {
    pub permission_status: String, // "granted", "denied", "skipped"
    pub last_permission_denial_at: Option<String>, // RFC3339 timestamp when denied (for grace period)
    pub changed_at: String,                        // RFC3339 formatted timestamp
}

/// UserMealPlanningPreferencesUpdated event emitted when user updates meal planning preferences
///
/// This event captures all user preferences used by the meal planning algorithm for personalization.
/// Emitted when user configures preferences in profile settings or onboarding.
///
/// Design note: All preference fields serialized with bincode for evento compatibility.
/// Uses String for timestamps and structured types for preferences.
///
/// Note: user_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct UserMealPlanningPreferencesUpdated {
    /// Dietary restrictions (serialized as JSON array)
    pub dietary_restrictions: String, // JSON: [{"type":"Vegetarian"},{"type":"Custom","value":"shellfish"}]
    /// Number of people in household
    pub household_size: u32,
    /// Cooking skill level: "Beginner", "Intermediate", "Advanced"
    pub skill_level: String,
    /// Weeknight availability (JSON)
    pub weeknight_availability: String, // JSON: {"start":"18:00","duration_minutes":45}
    /// Maximum prep time for weeknight meals (minutes)
    pub max_prep_time_weeknight: u32,
    /// Maximum prep time for weekend meals (minutes)
    pub max_prep_time_weekend: u32,
    /// Avoid scheduling complex meals back-to-back
    pub avoid_consecutive_complex: bool,
    /// Cuisine variety weight (0.0-1.0)
    pub cuisine_variety_weight: f32,
    /// Timestamp when preferences were updated
    pub updated_at: String, // RFC3339 formatted timestamp
}
