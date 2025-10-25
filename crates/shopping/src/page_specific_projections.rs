/// Page-specific projection handlers for Shopping List page
///
/// Maps shopping domain events to page-specific read model tables:
/// - shopping_list_view: Categorized items with checkoff state
/// - shopping_list_summary: Category totals and completion progress
use evento::{AggregatorName, Context, EventDetails, Executor};
use sqlx::SqlitePool;

use crate::aggregate::ShoppingListAggregate;
use crate::events::{
    ShoppingListGenerated, ShoppingListItemCollected, ShoppingListRecalculated, ShoppingListReset,
};

/// Register all shopping list page-specific projection handlers
pub fn shopping_list_page_specific_projections(
    pool: SqlitePool,
) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("shopping-list-page-specific-projections")
        .aggregator::<ShoppingListAggregate>()
        .data(pool)
        .handler(project_shopping_list_to_view())
        .handler(update_shopping_list_on_recalculation())
        .handler(mark_item_as_collected())
        .handler(reset_all_checkboxes())
}

/// Handler: ShoppingListGenerated → shopping_list_view + shopping_list_summary
///
/// Projects shopping list generation to page-specific tables with denormalized item data.
#[evento::handler(ShoppingListAggregate)]
pub async fn project_shopping_list_to_view<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ShoppingListGenerated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let shopping_list_id = &event.aggregator_id;

    let mut tx = pool.begin().await?;

    // Delete old items for this shopping list (if regenerating)
    sqlx::query("DELETE FROM shopping_list_view WHERE shopping_list_id = ?1")
        .bind(shopping_list_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM shopping_list_summary WHERE shopping_list_id = ?1")
        .bind(shopping_list_id)
        .execute(&mut *tx)
        .await?;

    // Insert items into shopping_list_view
    for item in &event.data.items {
        let item_id = format!("{}:{}", shopping_list_id, item.ingredient_name);

        sqlx::query(
            r#"
            INSERT INTO shopping_list_view (
                id, shopping_list_id, user_id, week_start_date,
                ingredient_name, quantity, unit, category, is_collected,
                source_recipes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, '[]', ?9)
            "#,
        )
        .bind(&item_id)
        .bind(shopping_list_id)
        .bind(&event.data.user_id)
        .bind(&event.data.week_start_date)
        .bind(&item.ingredient_name)
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(&item.category)
        .bind(&event.data.generated_at)
        .execute(&mut *tx)
        .await?;
    }

    // Calculate category summaries
    let categories: Vec<String> = event
        .data
        .items
        .iter()
        .map(|i| i.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for category in categories {
        let total_items = event
            .data
            .items
            .iter()
            .filter(|i| i.category == category)
            .count();

        sqlx::query(
            r#"
            INSERT INTO shopping_list_summary (
                shopping_list_id, category, total_items, collected_items, updated_at
            ) VALUES (?1, ?2, ?3, 0, ?4)
            "#,
        )
        .bind(shopping_list_id)
        .bind(&category)
        .bind(total_items as i32)
        .bind(&event.data.generated_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Handler: ShoppingListRecalculated → Update shopping_list_view + shopping_list_summary
///
/// Updates shopping list when meal is replaced (Story 4.4).
#[evento::handler(ShoppingListAggregate)]
pub async fn update_shopping_list_on_recalculation<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ShoppingListRecalculated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let shopping_list_id = &event.aggregator_id;

    let mut tx = pool.begin().await?;

    // Delete all items and summaries (simpler than diffing)
    sqlx::query("DELETE FROM shopping_list_view WHERE shopping_list_id = ?1")
        .bind(shopping_list_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM shopping_list_summary WHERE shopping_list_id = ?1")
        .bind(shopping_list_id)
        .execute(&mut *tx)
        .await?;

    // Query user_id and week_start_date from shopping_lists table
    let (user_id, week_start_date): (String, String) = sqlx::query_as(
        "SELECT user_id, week_start_date FROM shopping_lists WHERE id = ?1",
    )
    .bind(shopping_list_id)
    .fetch_one(&mut *tx)
    .await?;

    // Re-insert updated items
    for item in &event.data.items {
        let item_id = format!("{}:{}", shopping_list_id, item.ingredient_name);

        sqlx::query(
            r#"
            INSERT INTO shopping_list_view (
                id, shopping_list_id, user_id, week_start_date,
                ingredient_name, quantity, unit, category, is_collected,
                source_recipes, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, '[]', ?9)
            "#,
        )
        .bind(&item_id)
        .bind(shopping_list_id)
        .bind(&user_id)
        .bind(&week_start_date)
        .bind(&item.ingredient_name)
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(&item.category)
        .bind(&event.data.recalculated_at)
        .execute(&mut *tx)
        .await?;
    }

    // Recalculate category summaries
    let categories: Vec<String> = event
        .data
        .items
        .iter()
        .map(|i| i.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for category in categories {
        let total_items = event
            .data
            .items
            .iter()
            .filter(|i| i.category == category)
            .count();

        sqlx::query(
            r#"
            INSERT INTO shopping_list_summary (
                shopping_list_id, category, total_items, collected_items, updated_at
            ) VALUES (?1, ?2, ?3, 0, ?4)
            "#,
        )
        .bind(shopping_list_id)
        .bind(&category)
        .bind(total_items as i32)
        .bind(&event.data.recalculated_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Handler: ShoppingListItemCollected → Update shopping_list_view + shopping_list_summary
///
/// Updates checkbox state for individual item (Story 4.5).
#[evento::handler(ShoppingListAggregate)]
pub async fn mark_item_as_collected<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ShoppingListItemCollected>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let mut tx = pool.begin().await?;

    // Update item checkbox state
    sqlx::query("UPDATE shopping_list_view SET is_collected = ?1 WHERE id = ?2")
        .bind(if event.data.is_collected { 1 } else { 0 })
        .bind(&event.data.item_id)
        .execute(&mut *tx)
        .await?;

    // Query shopping_list_id and category for summary update
    let (shopping_list_id, category): (String, String) = sqlx::query_as(
        "SELECT shopping_list_id, category FROM shopping_list_view WHERE id = ?1",
    )
    .bind(&event.data.item_id)
    .fetch_one(&mut *tx)
    .await?;

    // Recalculate category summary
    let collected_count: i32 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM shopping_list_view
        WHERE shopping_list_id = ?1 AND category = ?2 AND is_collected = 1
        "#,
    )
    .bind(&shopping_list_id)
    .bind(&category)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE shopping_list_summary
        SET collected_items = ?1, updated_at = ?2
        WHERE shopping_list_id = ?3 AND category = ?4
        "#,
    )
    .bind(collected_count)
    .bind(&event.data.collected_at)
    .bind(&shopping_list_id)
    .bind(&category)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Handler: ShoppingListReset → Reset all checkboxes in shopping_list_view + shopping_list_summary
///
/// Unchecks all items for next shopping trip (Story 4.5 AC #8).
#[evento::handler(ShoppingListAggregate)]
pub async fn reset_all_checkboxes<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ShoppingListReset>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();
    let shopping_list_id = &event.aggregator_id;

    let mut tx = pool.begin().await?;

    // Uncheck all items
    sqlx::query("UPDATE shopping_list_view SET is_collected = 0 WHERE shopping_list_id = ?1")
        .bind(shopping_list_id)
        .execute(&mut *tx)
        .await?;

    // Reset all category summaries
    sqlx::query(
        r#"
        UPDATE shopping_list_summary
        SET collected_items = 0, updated_at = ?1
        WHERE shopping_list_id = ?2
        "#,
    )
    .bind(&event.data.reset_at)
    .bind(shopping_list_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
