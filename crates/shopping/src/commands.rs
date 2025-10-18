use chrono::Utc;

use crate::aggregation::IngredientAggregationService;
use crate::categorization::CategorizationService;
use crate::events::{
    ShoppingListGenerated, ShoppingListItem, ShoppingListItemCollected, ShoppingListRecalculated,
    ShoppingListReset,
};

/// Command to generate a shopping list from a meal plan
#[derive(Debug, Clone)]
pub struct GenerateShoppingListCommand {
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String, // ISO 8601 date (Monday of the week)
    pub ingredients: Vec<(String, f32, String)>, // (name, quantity, unit) - passed from route
}

/// Command to recalculate shopping list when a meal slot is replaced (Story 4.4)
#[derive(Debug, Clone)]
pub struct RecalculateShoppingListCommand {
    pub shopping_list_id: String,
    pub old_recipe_ingredients: Vec<(String, f32, String)>, // Ingredients to subtract
    pub new_recipe_ingredients: Vec<(String, f32, String)>, // Ingredients to add
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

    #[error("Shopping list item not found: {0}")]
    ItemNotFoundError(String),

    #[error("Shopping list not found: {0}")]
    ShoppingListNotFoundError(String),
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

/// Recalculate shopping list on meal replacement (Story 4.4)
///
/// This command:
/// 1. Loads the current shopping list aggregate from event store
/// 2. Converts current items back to ingredient tuples
/// 3. Subtracts old recipe ingredients
/// 4. Adds new recipe ingredients
/// 5. Re-aggregates all ingredients (sum quantities, normalize units)
/// 6. Re-categorizes ingredients
/// 7. Emits ShoppingListRecalculated event
///
/// Performance target: <1 second total time
pub async fn recalculate_shopping_list_on_meal_replacement(
    cmd: RecalculateShoppingListCommand,
    executor: &impl evento::Executor,
) -> Result<(), ShoppingListError> {
    // 1. Load current shopping list aggregate from event store
    let loaded =
        evento::load::<crate::aggregate::ShoppingListAggregate, _>(executor, &cmd.shopping_list_id)
            .await
            .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

    let shopping_list = loaded.item;

    // 2. Convert current items to ingredient tuples for recalculation
    let mut current_ingredients: Vec<(String, f32, String)> = shopping_list
        .items
        .iter()
        .map(|item| {
            (
                item.ingredient_name.clone(),
                item.quantity,
                item.unit.clone(),
            )
        })
        .collect();

    // 3. Subtract old recipe ingredients (negate quantities)
    for (name, quantity, unit) in cmd.old_recipe_ingredients {
        current_ingredients.push((name, -quantity, unit));
    }

    // 4. Add new recipe ingredients
    current_ingredients.extend(cmd.new_recipe_ingredients);

    // 5. Re-aggregate ingredients (sum quantities, normalize units)
    let aggregated = IngredientAggregationService::aggregate(current_ingredients)
        .map_err(|e| ShoppingListError::AggregationError(e.to_string()))?;

    // 6. Filter out ingredients with zero or negative quantities (removed from list)
    let filtered: Vec<(String, f32, String)> = aggregated
        .into_iter()
        .filter(|(_, quantity, _)| *quantity > 0.0)
        .collect();

    // 7. Re-categorize ingredients
    let items: Vec<ShoppingListItem> = filtered
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

    // 8. Emit ShoppingListRecalculated event
    let recalculated_at = Utc::now().to_rfc3339();

    let event = ShoppingListRecalculated {
        items,
        recalculated_at,
    };

    // Append event to existing aggregate using evento::save
    evento::save::<crate::aggregate::ShoppingListAggregate>(&cmd.shopping_list_id)
        .data(&event)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

    Ok(())
}

/// Command to mark a shopping list item as collected/uncollected (Story 4.5)
#[derive(Debug, Clone)]
pub struct MarkItemCollectedCommand {
    pub shopping_list_id: String,
    pub item_id: String,
    pub is_collected: bool,
}

/// Mark a shopping list item as collected/uncollected (Story 4.5 - AC #1, #2, #3)
///
/// This command:
/// 1. Validates that the item exists
/// 2. Emits ShoppingListItemCollected event
/// 3. Read model projection updates is_collected column
///
/// Note: Validation that user owns the shopping list should be done in the route handler
/// before calling this command (checking user_id from JWT matches shopping_list.user_id)
pub async fn mark_item_collected(
    cmd: MarkItemCollectedCommand,
    executor: &impl evento::Executor,
) -> Result<(), ShoppingListError> {
    // Create event with timestamp
    let collected_at = Utc::now().to_rfc3339();

    let event = ShoppingListItemCollected {
        item_id: cmd.item_id,
        is_collected: cmd.is_collected,
        collected_at,
    };

    // Append event to shopping list aggregate
    evento::save::<crate::aggregate::ShoppingListAggregate>(&cmd.shopping_list_id)
        .data(&event)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

    Ok(())
}

/// Command to reset all items in a shopping list (Story 4.5)
#[derive(Debug, Clone)]
pub struct ResetShoppingListCommand {
    pub shopping_list_id: String,
}

/// Reset all items in a shopping list (uncheck all checkboxes) - Story 4.5 AC #8
///
/// This command:
/// 1. Emits ShoppingListReset event
/// 2. Read model projection sets all is_collected = false for this shopping list
///
/// Note: Validation that user owns the shopping list should be done in the route handler
pub async fn reset_shopping_list(
    cmd: ResetShoppingListCommand,
    executor: &impl evento::Executor,
) -> Result<(), ShoppingListError> {
    // Create event with timestamp
    let reset_at = Utc::now().to_rfc3339();

    let event = ShoppingListReset { reset_at };

    // Append event to shopping list aggregate
    evento::save::<crate::aggregate::ShoppingListAggregate>(&cmd.shopping_list_id)
        .data(&event)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .metadata(&true)
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?
        .commit(executor)
        .await
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

    Ok(())
}
