use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

use crate::types::{AccompanimentCategory, Cuisine, DietaryTag};

/// Ingredient structure for recipes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
pub struct Ingredient {
    pub name: String,
    pub quantity: f32,
    pub unit: String, // e.g., "cups", "tbsp", "grams", "oz"
}

/// Instruction step for recipes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
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
/// # Epic 6: Enhanced Meal Planning System
/// Added accompaniment fields (accepts_accompaniment, preferred_accompaniments, accompaniment_category)
/// and metadata fields (cuisine, dietary_tags) for multi-week meal planning algorithm.
///
/// # Backwards Compatibility
/// All Epic 6 fields are Option types to support deserialization of old events without these fields.
/// Old events default to: accepts_accompaniment=false, preferred_accompaniments=[], others=None.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeCreated {
    pub user_id: String,                    // Owner of the recipe
    pub title: String,                      // Recipe title
    pub recipe_type: String,                // Course type: "appetizer", "main_course", or "dessert"
    pub ingredients: Vec<Ingredient>,       // List of ingredients with quantities
    pub instructions: Vec<InstructionStep>, // Step-by-step cooking instructions
    pub prep_time_min: Option<u32>,         // Preparation time in minutes
    pub cook_time_min: Option<u32>,         // Cooking time in minutes
    pub advance_prep_hours: Option<u32>,    // Hours needed for advance prep (e.g., marinating)
    pub serving_size: Option<u32>,          // Number of servings
    pub created_at: String,                 // RFC3339 formatted timestamp

    // Epic 6: Accompaniment support
    #[serde(default)]
    pub accepts_accompaniment: Option<bool>, // Whether this main course accepts an accompaniment (default: false)
    #[serde(default)]
    pub preferred_accompaniments: Option<Vec<AccompanimentCategory>>, // Preferred side categories (default: [])
    #[serde(default)]
    pub accompaniment_category: Option<AccompanimentCategory>, // Category if this is a side dish (default: None)

    // Epic 6: Cuisine and dietary metadata
    #[serde(default)]
    pub cuisine: Option<Cuisine>, // Cuisine type for variety tracking (default: None)
    #[serde(default)]
    pub dietary_tags: Option<Vec<DietaryTag>>, // Dietary tags for filtering (default: [])
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
    pub recipe_type: Option<String>, // Allow updating course type: "appetizer", "main_course", or "dessert"
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

/// RecipeShared event emitted when a recipe's privacy status is toggled
///
/// This event captures changes to the is_shared flag, allowing recipes to be
/// shared with the community discovery feed or made private again.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeShared {
    pub user_id: String, // ID of the user who toggled the share status (ownership verified)
    pub shared: bool,    // true = shared with community, false = private
    pub toggled_at: String, // RFC3339 formatted timestamp
}

/// RecipeRated event emitted when a user rates and optionally reviews a recipe
///
/// This event captures both ratings (1-5 stars) and optional text reviews.
/// One rating per user per recipe - duplicate submissions update existing rating.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeRated {
    pub user_id: String,             // ID of the user who submitted the rating
    pub stars: i32,                  // Rating value (1-5 inclusive)
    pub review_text: Option<String>, // Optional text review (max 500 chars)
    pub rated_at: String,            // RFC3339 formatted timestamp
}

/// RatingUpdated event emitted when a user updates their existing rating/review
///
/// This event allows users to edit their stars and/or review text after initial submission.
/// Only the user who created the rating can update it.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RatingUpdated {
    pub user_id: String,             // ID of the user who updated the rating
    pub stars: i32,                  // New rating value (1-5 inclusive)
    pub review_text: Option<String>, // New review text (max 500 chars)
    pub updated_at: String,          // RFC3339 formatted timestamp
}

/// RatingDeleted event emitted when a user deletes their rating/review
///
/// This event removes the rating from aggregate calculations and hides it from the UI.
/// Only the user who created the rating can delete it.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RatingDeleted {
    pub user_id: String,    // ID of the user who deleted the rating
    pub deleted_at: String, // RFC3339 formatted timestamp
}

/// RecipeCopied event emitted when a user copies a community recipe to their library
///
/// This event creates a new Recipe aggregate with full data duplication from the original.
/// The copy is owned by the copying user and defaults to private (is_shared=false).
/// Original recipe attribution is preserved for audit trail.
///
/// Note: new_recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeCopied {
    pub original_recipe_id: String, // ID of the original community recipe
    pub original_author: String,    // User ID of the original recipe creator
    pub copying_user_id: String,    // ID of the user copying the recipe
    pub copied_at: String,          // RFC3339 formatted timestamp
}

/// RecipeAccompanimentSettingsUpdated event emitted when accompaniment settings are changed
///
/// This event allows users to update whether a recipe accepts accompaniments and which
/// accompaniment categories it prefers, without needing to update the entire recipe.
///
/// # Epic 6: Enhanced Meal Planning System
/// Enables flexible accompaniment pairing configuration for the multi-week meal planning algorithm.
///
/// Note: recipe_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeAccompanimentSettingsUpdated {
    pub recipe_id: String,           // ID of the recipe being updated
    pub user_id: String,             // ID of the user updating settings (ownership verification)
    pub accepts_accompaniment: bool, // Whether this recipe accepts an accompaniment
    pub preferred_accompaniments: Vec<AccompanimentCategory>, // Preferred side categories
    pub updated_at: String,          // RFC3339 formatted timestamp
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test RecipeCreated bincode serialization round-trip
    #[test]
    fn test_recipe_created_bincode_roundtrip() {
        let event = RecipeCreated {
            user_id: "user-123".to_string(),
            title: "Tikka Masala".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![Ingredient {
                name: "Chicken".to_string(),
                quantity: 500.0,
                unit: "g".to_string(),
            }],
            instructions: vec![InstructionStep {
                step_number: 1,
                instruction_text: "Cook chicken".to_string(),
                timer_minutes: Some(20),
            }],
            prep_time_min: Some(30),
            cook_time_min: Some(45),
            advance_prep_hours: Some(4),
            serving_size: Some(4),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            accepts_accompaniment: Some(true),
            preferred_accompaniments: Some(vec![AccompanimentCategory::Rice]),
            accompaniment_category: None,
            cuisine: Some(Cuisine::Indian),
            dietary_tags: Some(vec![DietaryTag::GlutenFree]),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeCreated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.user_id, decoded.user_id);
        assert_eq!(event.title, decoded.title);
        assert_eq!(event.recipe_type, decoded.recipe_type);
        assert_eq!(event.accepts_accompaniment, decoded.accepts_accompaniment);
        assert_eq!(
            event.preferred_accompaniments,
            decoded.preferred_accompaniments
        );
        assert_eq!(event.cuisine, decoded.cuisine);
        assert_eq!(event.dietary_tags, decoded.dietary_tags);
    }

    /// Test RecipeDeleted bincode serialization
    #[test]
    fn test_recipe_deleted_bincode_roundtrip() {
        let event = RecipeDeleted {
            user_id: "user-123".to_string(),
            deleted_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeDeleted, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.user_id, decoded.user_id);
        assert_eq!(event.deleted_at, decoded.deleted_at);
    }

    /// Test RecipeFavorited bincode serialization
    #[test]
    fn test_recipe_favorited_bincode_roundtrip() {
        let event = RecipeFavorited {
            user_id: "user-123".to_string(),
            favorited: true,
            toggled_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeFavorited, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.user_id, decoded.user_id);
        assert_eq!(event.favorited, decoded.favorited);
    }

    /// Test RecipeUpdated bincode serialization
    #[test]
    fn test_recipe_updated_bincode_roundtrip() {
        let event = RecipeUpdated {
            title: Some("Updated Title".to_string()),
            recipe_type: Some("dessert".to_string()),
            ingredients: None,
            instructions: None,
            prep_time_min: Some(Some(25)),
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: Some(None),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeUpdated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.title, decoded.title);
        assert_eq!(event.recipe_type, decoded.recipe_type);
        assert_eq!(event.prep_time_min, decoded.prep_time_min);
    }

    /// Test RecipeTagged bincode serialization
    #[test]
    fn test_recipe_tagged_bincode_roundtrip() {
        let event = RecipeTagged {
            complexity: Some("moderate".to_string()),
            cuisine: Some("Mexican".to_string()),
            dietary_tags: vec!["vegetarian".to_string()],
            manual_override: false,
            tagged_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeTagged, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.complexity, decoded.complexity);
        assert_eq!(event.cuisine, decoded.cuisine);
        assert_eq!(event.dietary_tags, decoded.dietary_tags);
    }

    /// Test RecipeShared bincode serialization
    #[test]
    fn test_recipe_shared_bincode_roundtrip() {
        let event = RecipeShared {
            user_id: "user-123".to_string(),
            shared: true,
            toggled_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeShared, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.user_id, decoded.user_id);
        assert_eq!(event.shared, decoded.shared);
    }

    /// Test RecipeRated bincode serialization
    #[test]
    fn test_recipe_rated_bincode_roundtrip() {
        let event = RecipeRated {
            user_id: "user-123".to_string(),
            stars: 5,
            review_text: Some("Excellent!".to_string()),
            rated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeRated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.user_id, decoded.user_id);
        assert_eq!(event.stars, decoded.stars);
        assert_eq!(event.review_text, decoded.review_text);
    }

    /// Test RecipeCopied bincode serialization
    #[test]
    fn test_recipe_copied_bincode_roundtrip() {
        let event = RecipeCopied {
            original_recipe_id: "recipe-orig".to_string(),
            original_author: "user-orig".to_string(),
            copying_user_id: "user-copy".to_string(),
            copied_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeCopied, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.original_recipe_id, decoded.original_recipe_id);
        assert_eq!(event.original_author, decoded.original_author);
    }

    /// Test RecipeAccompanimentSettingsUpdated bincode serialization (Epic 6)
    #[test]
    fn test_recipe_accompaniment_settings_updated_bincode_roundtrip() {
        let event = RecipeAccompanimentSettingsUpdated {
            recipe_id: "recipe-123".to_string(),
            user_id: "user-456".to_string(),
            accepts_accompaniment: true,
            preferred_accompaniments: vec![
                AccompanimentCategory::Pasta,
                AccompanimentCategory::Salad,
            ],
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeAccompanimentSettingsUpdated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(event.recipe_id, decoded.recipe_id);
        assert_eq!(event.accepts_accompaniment, decoded.accepts_accompaniment);
        assert_eq!(
            event.preferred_accompaniments,
            decoded.preferred_accompaniments
        );
    }

    /// Test empty preferred_accompaniments edge case
    #[test]
    fn test_recipe_created_empty_preferred_accompaniments() {
        let event = RecipeCreated {
            user_id: "user-123".to_string(),
            title: "Simple Pasta".to_string(),
            recipe_type: "main_course".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            accepts_accompaniment: Some(false),
            preferred_accompaniments: Some(vec![]), // Empty vector
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: Some(vec![]),
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeCreated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(decoded.preferred_accompaniments, Some(vec![]));
        assert_eq!(decoded.dietary_tags, Some(vec![]));
    }

    /// Test None values for Option<Cuisine> edge case
    #[test]
    fn test_recipe_created_none_cuisine() {
        let event = RecipeCreated {
            user_id: "user-123".to_string(),
            title: "Generic Recipe".to_string(),
            recipe_type: "dessert".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_min: None,
            cook_time_min: None,
            advance_prep_hours: None,
            serving_size: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            accepts_accompaniment: None,
            preferred_accompaniments: None,
            accompaniment_category: None,
            cuisine: None, // None cuisine
            dietary_tags: None,
        };

        let encoded = bincode::encode_to_vec(&event, bincode::config::standard()).unwrap();
        let (decoded, _): (RecipeCreated, _) =
            bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();

        assert_eq!(decoded.cuisine, None);
        assert_eq!(decoded.dietary_tags, None);
    }
}
