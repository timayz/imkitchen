use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// Shopping list item structure
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, PartialEq)]
pub struct ShoppingListItem {
    pub ingredient_name: String,
    pub quantity: f32,
    pub unit: String,
    pub category: String, // Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
}

/// ShoppingListGenerated event emitted when a shopping list is generated
///
/// This event is the source of truth for shopping list creation in the event sourced system.
/// Uses String types for bincode compatibility (UUID and timestamps serialized as strings).
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ShoppingListGenerated {
    pub user_id: String,              // Owner of the shopping list
    pub meal_plan_id: String,         // Associated meal plan
    pub week_start_date: String,      // ISO 8601 date (Monday of the week)
    pub items: Vec<ShoppingListItem>, // Aggregated and categorized shopping items
    pub generated_at: String,         // RFC3339 formatted timestamp
}

/// ShoppingListItemCollected event emitted when a user marks an item as collected
///
/// Story 4.5: This event captures checkbox state changes for shopping list items.
/// The event maintains full audit trail of checkbox toggles in the event store.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ShoppingListItemCollected {
    pub item_id: String,      // Shopping list item ID
    pub is_collected: bool,   // true = collected, false = uncollected
    pub collected_at: String, // RFC3339 formatted timestamp
}

/// ShoppingListRecalculated event emitted when shopping list is recalculated due to meal replacement
///
/// This event is emitted in Story 4.4 when a meal slot is replaced, triggering recalculation
/// of the shopping list by subtracting old recipe ingredients and adding new recipe ingredients.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ShoppingListRecalculated {
    pub items: Vec<ShoppingListItem>, // Updated aggregated and categorized shopping items
    pub recalculated_at: String,      // RFC3339 formatted timestamp
}

/// ShoppingListReset event emitted when user resets all checkboxes for next shopping trip
///
/// Story 4.5: This event unchecks all items in a shopping list (AC #8).
/// Used when the user has completed shopping and wants to prepare the list for the next trip.
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct ShoppingListReset {
    pub reset_at: String, // RFC3339 formatted timestamp
}
