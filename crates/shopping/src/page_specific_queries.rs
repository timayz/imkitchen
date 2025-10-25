/// Query functions for page-specific read models (Shopping List page)
///
/// These functions query the new page-specific tables (shopping_list_view, shopping_list_summary)
/// instead of the old domain-centric table (shopping_list_items).
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Shopping list item data (denormalized, no JOINs needed)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ShoppingListItemData {
    pub id: String,
    pub shopping_list_id: String,
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit: String,
    pub category: String,
    pub is_collected: i32,      // SQLite boolean
    pub source_recipes: String, // JSON array
}

/// Shopping list category summary data
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CategorySummaryData {
    pub category: String,
    pub total_items: i32,
    pub collected_items: i32,
}

/// Query shopping list items for a specific shopping list (Shopping List page)
///
/// Returns items grouped by category with checkbox states.
pub async fn get_shopping_list_items(
    shopping_list_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<ShoppingListItemData>, sqlx::Error> {
    sqlx::query_as::<_, ShoppingListItemData>(
        r#"
        SELECT id, shopping_list_id, ingredient_name, quantity, unit, category,
               is_collected, source_recipes
        FROM shopping_list_view
        WHERE shopping_list_id = ?1
        ORDER BY category, ingredient_name
        "#,
    )
    .bind(shopping_list_id)
    .fetch_all(pool)
    .await
}

/// Query shopping list category summaries (Shopping List page)
///
/// Returns category-level completion progress (e.g., "Produce: 5/10 collected").
pub async fn get_category_summaries(
    shopping_list_id: &str,
    pool: &SqlitePool,
) -> Result<Vec<CategorySummaryData>, sqlx::Error> {
    sqlx::query_as::<_, CategorySummaryData>(
        r#"
        SELECT category, total_items, collected_items
        FROM shopping_list_summary
        WHERE shopping_list_id = ?1
        ORDER BY category
        "#,
    )
    .bind(shopping_list_id)
    .fetch_all(pool)
    .await
}

/// Query shopping list by user and week (Shopping List page)
///
/// Returns the shopping list ID for a specific user and week.
pub async fn get_shopping_list_by_week(
    user_id: &str,
    week_start_date: &str,
    pool: &SqlitePool,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT shopping_list_id
        FROM shopping_list_view
        WHERE user_id = ?1 AND week_start_date = ?2
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(week_start_date)
    .fetch_optional(pool)
    .await
}

/// Query overall completion progress for shopping list
///
/// Returns (total_items, collected_items).
pub async fn get_shopping_list_progress(
    shopping_list_id: &str,
    pool: &SqlitePool,
) -> Result<(i32, i32), sqlx::Error> {
    let result: (i32, i32) = sqlx::query_as(
        r#"
        SELECT
            SUM(total_items) as total,
            SUM(collected_items) as collected
        FROM shopping_list_summary
        WHERE shopping_list_id = ?1
        "#,
    )
    .bind(shopping_list_id)
    .fetch_one(pool)
    .await?;

    Ok(result)
}
