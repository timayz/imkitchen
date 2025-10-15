use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// Ingredient structure for recipes
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Ingredient {
    pub name: String,
    pub quantity: f32,
    pub unit: String, // e.g., "cups", "tbsp", "grams", "oz"
}

/// Instruction step for recipes
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct InstructionStep {
    pub step_number: u32,
    pub instruction_text: String,
    pub timer_minutes: Option<u32>, // Optional timer for this step
}

/// RecipeCreated event emitted when a new recipe is created
///
/// This event is the source of truth for recipe creation in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeCreated {
    pub user_id: String,                    // Owner of the recipe
    pub title: String,                      // Recipe title
    pub ingredients: Vec<Ingredient>,       // List of ingredients with quantities
    pub instructions: Vec<InstructionStep>, // Step-by-step cooking instructions
    pub prep_time_min: Option<u32>,         // Preparation time in minutes
    pub cook_time_min: Option<u32>,         // Cooking time in minutes
    pub advance_prep_hours: Option<u32>,    // Hours needed for advance prep (e.g., marinating)
    pub serving_size: Option<u32>,          // Number of servings
    pub created_at: String,                 // RFC3339 formatted timestamp
}

/// RecipeDeleted event emitted when a recipe is deleted
///
/// This event marks a recipe as deleted (soft delete in event sourcing).
/// The RecipeDeleted event is consumed by the user domain to decrement recipe_count.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeDeleted {
    pub user_id: String,    // ID of the user who deleted the recipe
    pub deleted_at: String, // RFC3339 formatted timestamp
}

/// RecipeFavorited event emitted when a user toggles favorite status
///
/// This event tracks favorite status changes for quick access filtering.
/// User domain subscribes to this event to update favorite_count for performance.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeFavorited {
    pub user_id: String,    // ID of the user who favorited/unfavorited the recipe
    pub favorited: bool,    // true = favorited, false = unfavorited
    pub toggled_at: String, // RFC3339 formatted timestamp
}

/// RecipeUpdated event emitted when a recipe is modified
///
/// This event stores only the changed fields (delta) for efficiency.
/// All fields are Optional - only modified values are included in the event.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeUpdated {
    pub title: Option<String>,
    pub ingredients: Option<Vec<Ingredient>>,
    pub instructions: Option<Vec<InstructionStep>>,
    pub prep_time_min: Option<Option<u32>>, // Option<Option<>> to differentiate between "not changed" and "set to None"
    pub cook_time_min: Option<Option<u32>>,
    pub advance_prep_hours: Option<Option<u32>>,
    pub serving_size: Option<Option<u32>>,
    pub updated_at: String, // RFC3339 formatted timestamp
}

/// RecipeTagged event emitted when recipe tags are automatically assigned
///
/// This event is emitted after RecipeCreated or RecipeUpdated events
/// to store the automatically calculated tags (complexity, cuisine, dietary).
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeTagged {
    pub complexity: Option<String>, // "simple", "moderate", or "complex"
    pub cuisine: Option<String>,    // e.g., "Italian", "Asian", "Mexican", etc.
    pub dietary_tags: Vec<String>,  // e.g., ["vegetarian", "vegan", "gluten-free"]
    pub manual_override: bool,      // true if user manually set tags
    pub tagged_at: String,          // RFC3339 formatted timestamp
}
