use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{ShoppingListGenerated, ShoppingListItem, ShoppingListItemCollected};

/// ShoppingListAggregate representing the state of a shopping list entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// It stores the complete state of a weekly shopping list including all items.
///
/// Note: All fields are String types for bincode compatibility (follows evento best practices)
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct ShoppingListAggregate {
    // Core identity
    pub shopping_list_id: String,
    pub user_id: String,
    pub meal_plan_id: String,

    // Shopping list metadata
    pub week_start_date: String, // ISO 8601 date (Monday of the week)
    pub items: Vec<ShoppingListItem>,

    // Timestamps
    pub generated_at: String, // RFC3339 formatted timestamp
}

/// Implement evento aggregator pattern for ShoppingListAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl ShoppingListAggregate {
    /// Handle ShoppingListGenerated event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    async fn shopping_list_generated(
        &mut self,
        event: evento::EventDetails<ShoppingListGenerated>,
    ) -> anyhow::Result<()> {
        self.shopping_list_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.meal_plan_id = event.data.meal_plan_id;
        self.week_start_date = event.data.week_start_date;
        self.items = event.data.items;
        self.generated_at = event.data.generated_at;
        Ok(())
    }

    /// Handle ShoppingListItemCollected event to update item collected status
    ///
    /// Note: This is for Story 4.2 (future), but we define the handler here for completeness.
    /// In the current story (4.1), we don't persist collected status in the aggregate.
    async fn shopping_list_item_collected(
        &mut self,
        event: evento::EventDetails<ShoppingListItemCollected>,
    ) -> anyhow::Result<()> {
        // For Story 4.2: Update item collected status in read model
        // This handler exists for evento framework but performs no aggregate state changes
        // (collected status managed in read model, not in aggregate)
        let _ = event; // Suppress unused warning
        Ok(())
    }
}
