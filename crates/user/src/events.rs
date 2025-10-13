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

/// SkillLevelSet event emitted when user sets cooking skill level (Step 3)
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct SkillLevelSet {
    pub skill_level: String, // "beginner", "intermediate", "expert"
    pub set_at: String,      // RFC3339 formatted timestamp
}

/// WeeknightAvailabilitySet event emitted when user sets weeknight availability (Step 4)
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
    pub skill_level: Option<String>,               // None = no change
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
