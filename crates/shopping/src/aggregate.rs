use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    ShoppingListGenerated, ShoppingListItem, ShoppingListItemCollected, ShoppingListRecalculated,
    ShoppingListReset,
};

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

    /// Handle ShoppingListItemCollected event (Story 4.5)
    ///
    /// This handler exists for evento framework but performs no aggregate state changes.
    /// The checkbox state (is_collected) is managed in the read model, not in the aggregate.
    /// The aggregate doesn't need to track checkbox state for business logic.
    async fn shopping_list_item_collected(
        &mut self,
        event: evento::EventDetails<ShoppingListItemCollected>,
    ) -> anyhow::Result<()> {
        // Checkbox state is managed in read model via projection
        // No aggregate state changes needed
        let _ = event; // Suppress unused warning
        Ok(())
    }

    /// Handle ShoppingListRecalculated event to update aggregate state
    ///
    /// This is called when a meal replacement triggers shopping list recalculation (Story 4.4).
    /// The event contains the newly calculated shopping list items after subtracting old recipe
    /// ingredients and adding new recipe ingredients.
    async fn shopping_list_recalculated(
        &mut self,
        event: evento::EventDetails<ShoppingListRecalculated>,
    ) -> anyhow::Result<()> {
        self.items = event.data.items;
        Ok(())
    }

    /// Handle ShoppingListReset event (Story 4.5)
    ///
    /// This handler exists for evento framework but performs no aggregate state changes.
    /// The reset operation (unchecking all items) is managed in the read model via projection.
    async fn shopping_list_reset(
        &mut self,
        event: evento::EventDetails<ShoppingListReset>,
    ) -> anyhow::Result<()> {
        // Reset state is managed in read model via projection
        // No aggregate state changes needed
        let _ = event; // Suppress unused warning
        Ok(())
    }
}
