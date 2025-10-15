use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::collection_events::{
    CollectionCreated, CollectionDeleted, CollectionUpdated, RecipeAddedToCollection,
    RecipeRemovedFromCollection,
};

/// Collection aggregate representing the state of a collection entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// Collections organize recipes into groups for easier discovery and filtering.
///
/// Note: collection_id, user_id, created_at stored as String for bincode compatibility
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct CollectionAggregate {
    // Core identity
    pub collection_id: String,
    pub user_id: String, // Owner of the collection

    // Collection details
    pub name: String,
    pub description: Option<String>,

    // Recipe assignments (many-to-many)
    // Stores recipe IDs that belong to this collection
    pub recipe_ids: HashSet<String>,

    // Status flags
    pub is_deleted: bool,

    // Timestamps
    pub created_at: String, // RFC3339 formatted timestamp
}

/// Implement evento aggregator pattern for CollectionAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl CollectionAggregate {
    /// Handle CollectionCreated event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    async fn collection_created(
        &mut self,
        event: evento::EventDetails<CollectionCreated>,
    ) -> anyhow::Result<()> {
        self.collection_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.name = event.data.name;
        self.description = event.data.description;
        self.created_at = event.data.created_at;
        self.is_deleted = false;
        self.recipe_ids = HashSet::new();
        Ok(())
    }

    /// Handle CollectionUpdated event to apply changes to aggregate state
    ///
    /// This event handler updates only the fields that were changed (delta pattern).
    /// Fields that are None in the event are not modified in the aggregate.
    async fn collection_updated(
        &mut self,
        event: evento::EventDetails<CollectionUpdated>,
    ) -> anyhow::Result<()> {
        if let Some(name) = event.data.name {
            self.name = name;
        }
        if let Some(description) = event.data.description {
            self.description = description;
        }
        Ok(())
    }

    /// Handle CollectionDeleted event to mark collection as deleted
    ///
    /// This is a soft delete - the collection remains in the event store for audit trail,
    /// but is marked as deleted and won't be returned in queries.
    async fn collection_deleted(
        &mut self,
        _event: evento::EventDetails<CollectionDeleted>,
    ) -> anyhow::Result<()> {
        self.is_deleted = true;
        Ok(())
    }

    /// Handle RecipeAddedToCollection event to add recipe to collection
    ///
    /// This event handler adds a recipe ID to the collection's recipe_ids set.
    /// Recipes can belong to multiple collections simultaneously.
    async fn recipe_added_to_collection(
        &mut self,
        event: evento::EventDetails<RecipeAddedToCollection>,
    ) -> anyhow::Result<()> {
        self.recipe_ids.insert(event.data.recipe_id);
        Ok(())
    }

    /// Handle RecipeRemovedFromCollection event to remove recipe from collection
    ///
    /// This event handler removes a recipe ID from the collection's recipe_ids set.
    /// The recipe itself is not deleted, only the assignment is removed.
    async fn recipe_removed_from_collection(
        &mut self,
        event: evento::EventDetails<RecipeRemovedFromCollection>,
    ) -> anyhow::Result<()> {
        self.recipe_ids.remove(&event.data.recipe_id);
        Ok(())
    }
}
