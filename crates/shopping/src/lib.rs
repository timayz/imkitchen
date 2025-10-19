pub mod aggregate;
pub mod aggregation;
pub mod ambiguous;
pub mod categorization;
pub mod commands;
pub mod events;
pub mod fraction_utils;
pub mod read_model;

// Re-export commonly used types
pub use aggregate::ShoppingListAggregate;
pub use aggregation::IngredientAggregationService;
pub use categorization::{CategorizationService, Category};
pub use commands::{
    generate_shopping_list, recalculate_shopping_list_on_meal_replacement,
    GenerateShoppingListCommand, RecalculateShoppingListCommand, ShoppingListError,
};
pub use events::{
    ShoppingListGenerated, ShoppingListItem, ShoppingListItemCollected, ShoppingListRecalculated,
};
pub use read_model::{shopping_projection, validate_week_date};
