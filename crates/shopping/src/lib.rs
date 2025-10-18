pub mod aggregate;
pub mod aggregation;
pub mod categorization;
pub mod commands;
pub mod events;
pub mod read_model;

// Re-export commonly used types
pub use aggregate::ShoppingListAggregate;
pub use aggregation::IngredientAggregationService;
pub use categorization::{CategorizationService, Category};
pub use commands::{generate_shopping_list, GenerateShoppingListCommand, ShoppingListError};
pub use events::{ShoppingListGenerated, ShoppingListItem, ShoppingListItemCollected};
pub use read_model::{shopping_projection, validate_week_date};
