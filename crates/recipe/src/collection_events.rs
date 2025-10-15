use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// CollectionCreated event emitted when a new collection is created
///
/// This event is the source of truth for collection creation in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
///
/// Note: collection_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct CollectionCreated {
    pub user_id: String,             // Owner of the collection
    pub name: String,                // Collection name
    pub description: Option<String>, // Optional description
    pub created_at: String,          // RFC3339 formatted timestamp
}

/// CollectionUpdated event emitted when a collection is modified
///
/// This event stores only the changed fields (delta) for efficiency.
/// Fields that are None are not modified in the aggregate.
///
/// Note: collection_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct CollectionUpdated {
    pub name: Option<String>,                // Updated collection name
    pub description: Option<Option<String>>, // Option<Option<>> to differentiate "not changed" vs "set to None"
    pub updated_at: String,                  // RFC3339 formatted timestamp
}

/// CollectionDeleted event emitted when a collection is deleted
///
/// This event marks a collection as deleted (soft delete in event sourcing).
/// All recipe assignments are removed, but recipes themselves are preserved.
///
/// Note: collection_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct CollectionDeleted {
    pub user_id: String,    // ID of the user who deleted the collection
    pub deleted_at: String, // RFC3339 formatted timestamp
}

/// RecipeAddedToCollection event emitted when a recipe is added to a collection
///
/// This event creates a many-to-many relationship between recipe and collection.
/// Recipes can belong to multiple collections simultaneously.
///
/// Note: collection_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeAddedToCollection {
    pub recipe_id: String,   // ID of the recipe being added
    pub assigned_at: String, // RFC3339 formatted timestamp
}

/// RecipeRemovedFromCollection event emitted when a recipe is removed from a collection
///
/// This event deletes the many-to-many relationship between recipe and collection.
/// The recipe itself is not deleted, only the assignment is removed.
///
/// Note: collection_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeRemovedFromCollection {
    pub recipe_id: String,  // ID of the recipe being removed
    pub removed_at: String, // RFC3339 formatted timestamp
}
