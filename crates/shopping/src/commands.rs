use chrono::Utc;

use crate::aggregation::IngredientAggregationService;
use crate::categorization::CategorizationService;
use crate::events::{ShoppingListGenerated, ShoppingListItem};

/// Command to generate a shopping list from a meal plan
#[derive(Debug, Clone)]
pub struct GenerateShoppingListCommand {
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String, // ISO 8601 date (Monday of the week)
    pub ingredients: Vec<(String, f32, String)>, // (name, quantity, unit) - passed from route
}

/// Error types for shopping list commands
#[derive(Debug, thiserror::Error)]
pub enum ShoppingListError {
    #[error("Failed to aggregate ingredients: {0}")]
    AggregationError(String),

    #[error("Event store error: {0}")]
    EventStoreError(#[from] anyhow::Error),

    #[error("Invalid week: {0}")]
    InvalidWeekError(String),

    #[error("Past weeks are not accessible (out of scope for MVP)")]
    PastWeekNotAccessibleError,

    #[error("Future week out of range: Maximum 4 weeks ahead allowed")]
    FutureWeekOutOfRangeError,
}

/// Generate a shopping list from a meal plan
///
/// This command:
/// 1. Aggregates all ingredients (summing quantities, normalizing units)
/// 2. Categorizes ingredients by grocery store category
/// 3. Creates ShoppingList aggregate and emits ShoppingListGenerated event
///
/// Performance target: <2 seconds for 14 recipes (140 ingredients)
///
/// Note: Ingredients are passed in from the route handler (which queries recipes from meal plan)
pub async fn generate_shopping_list(
    cmd: GenerateShoppingListCommand,
    executor: &impl evento::Executor,
) -> Result<String, ShoppingListError> {
    // 1. Aggregate ingredients (sum quantities, normalize units)
    let aggregated = IngredientAggregationService::aggregate(cmd.ingredients)
        .map_err(|e| ShoppingListError::AggregationError(e.to_string()))?;

    // 2. Categorize ingredients
    let items: Vec<ShoppingListItem> = aggregated
        .into_iter()
        .map(|(name, quantity, unit)| {
            let category = CategorizationService::categorize(&name);
            ShoppingListItem {
                ingredient_name: name,
                quantity,
                unit,
                category: category.as_str().to_string(),
            }
        })
        .collect();

    // 3. Create ShoppingList aggregate and emit event
    let generated_at = Utc::now().to_rfc3339();

    let event = ShoppingListGenerated {
        user_id: cmd.user_id,
        meal_plan_id: cmd.meal_plan_id,
        week_start_date: cmd.week_start_date,
        items,
        generated_at,
    };

    // Use evento::create to emit the event (generates ULID for shopping_list_id)
    let shopping_list_id = evento::create::<crate::aggregate::ShoppingListAggregate>()
        .data(&event)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

    Ok(shopping_list_id)
}
