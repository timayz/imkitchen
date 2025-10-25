pub mod aggregate;
pub mod collection_aggregate;
pub mod collection_commands;
pub mod collection_events;
pub mod commands;
pub mod error;
pub mod events;
pub mod page_specific_projections;
pub mod page_specific_queries;
pub mod read_model;
pub mod tagging;

pub use aggregate::RecipeAggregate;
pub use collection_aggregate::CollectionAggregate;
pub use collection_commands::*;
pub use collection_events::*;
pub use commands::*;
pub use error::{RecipeError, RecipeResult};
pub use events::*;
pub use page_specific_projections::{
    recipe_dashboard_metrics_projections, recipe_detail_projections, recipe_list_projections,
    recipe_ratings_projections,
};
// Export page-specific query functions and types
// Note: get_recipe_detail and get_recipe_list conflict with read_model functions,
// so they must be imported with a module prefix (page_specific_queries::)
pub use page_specific_queries::{
    get_filter_counts, get_recipe_counts, get_recipe_list_filtered, get_recipe_ratings,
    get_shared_recipes, get_shared_recipes_filtered, FilterCount, RecipeDetailData,
    RecipeListCard, RecipeRatingsData,
};

// Re-export page-specific functions with explicit naming to avoid conflicts
pub use page_specific_queries::{
    get_recipe_detail as page_get_recipe_detail, get_recipe_list as page_get_recipe_list,
};
pub use read_model::*;
pub use tagging::*;

// Re-export batch import types for convenience
pub use commands::{
    batch_import_recipes, BatchImportRecipe, BatchImportRecipesCommand, BatchImportResult,
};
