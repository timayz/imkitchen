pub mod aggregate;
pub mod aggregation;
pub mod ambiguous;
pub mod categorization;
pub mod commands;
pub mod events;
pub mod fraction_utils;
pub mod page_specific_projections;
pub mod page_specific_queries;
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
pub use page_specific_projections::shopping_list_page_specific_projections;
pub use page_specific_queries::{
    get_category_summaries, get_shopping_list_by_week, get_shopping_list_items,
    get_shopping_list_progress, CategorySummaryData, ShoppingListItemData,
};
pub use read_model::{shopping_projection, validate_week_date};
