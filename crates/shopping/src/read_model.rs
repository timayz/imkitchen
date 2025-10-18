use crate::aggregate::ShoppingListAggregate;
use crate::events::ShoppingListGenerated;
use evento::{AggregatorName, Context, EventDetails, Executor};

/// Project ShoppingListGenerated event to read model tables
///
/// This evento subscription handler updates the shopping_lists and shopping_list_items
/// tables when a ShoppingListGenerated event is emitted.
///
/// Tables:
/// - shopping_lists: Header table (id, user_id, meal_plan_id, week_start_date, generated_at)
/// - shopping_list_items: Line items table (id, shopping_list_id, ingredient_name, quantity, unit, category, is_collected)
#[evento::handler(ShoppingListAggregate)]
pub async fn project_shopping_list_generated<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<ShoppingListGenerated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: sqlx::SqlitePool = context.extract();
    let shopping_list_id = &event.aggregator_id;

    // 1. Insert into shopping_lists table
    sqlx::query(
        r#"
        INSERT INTO shopping_lists (id, user_id, meal_plan_id, week_start_date, generated_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(shopping_list_id)
    .bind(&event.data.user_id)
    .bind(&event.data.meal_plan_id)
    .bind(&event.data.week_start_date)
    .bind(&event.data.generated_at)
    .execute(&pool)
    .await?;

    // 2. Insert all items into shopping_list_items table
    for (index, item) in event.data.items.iter().enumerate() {
        let item_id = format!("{}-{}", shopping_list_id, index);
        sqlx::query(
            r#"
            INSERT INTO shopping_list_items (id, shopping_list_id, ingredient_name, quantity, unit, category, is_collected)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&item_id)
        .bind(shopping_list_id)
        .bind(&item.ingredient_name)
        .bind(item.quantity)
        .bind(&item.unit)
        .bind(&item.category)
        .bind(false) // is_collected defaults to false
        .execute(&pool)
        .await?;
    }

    tracing::info!(
        "Projected ShoppingListGenerated event for shopping_list_id={} with {} items",
        shopping_list_id,
        event.data.items.len()
    );

    Ok(())
}

/// Query shopping list by ID
///
/// Returns the shopping list header and all items.
pub async fn get_shopping_list(
    shopping_list_id: &str,
    pool: &sqlx::SqlitePool,
) -> Result<Option<ShoppingListData>, sqlx::Error> {
    // Query shopping list header
    let header: Option<ShoppingListHeaderRow> = sqlx::query_as::<_, ShoppingListHeaderRow>(
        r#"
        SELECT id, user_id, meal_plan_id, week_start_date, generated_at
        FROM shopping_lists
        WHERE id = ?
        "#,
    )
    .bind(shopping_list_id)
    .fetch_optional(pool)
    .await?;

    if let Some(header) = header {
        // Query all items
        let items: Vec<ShoppingListItemRow> = sqlx::query_as::<_, ShoppingListItemRow>(
            r#"
            SELECT id, shopping_list_id, ingredient_name, quantity, unit, category, is_collected
            FROM shopping_list_items
            WHERE shopping_list_id = ?
            ORDER BY category, ingredient_name
            "#,
        )
        .bind(shopping_list_id)
        .fetch_all(pool)
        .await?;

        Ok(Some(ShoppingListData { header, items }))
    } else {
        Ok(None)
    }
}

/// Query shopping list by user and week
///
/// Returns the shopping list for a specific user and week (if exists).
pub async fn get_shopping_list_by_week(
    user_id: &str,
    week_start_date: &str,
    pool: &sqlx::SqlitePool,
) -> Result<Option<ShoppingListData>, sqlx::Error> {
    // Query shopping list header by user_id and week_start_date
    let header: Option<ShoppingListHeaderRow> = sqlx::query_as::<_, ShoppingListHeaderRow>(
        r#"
        SELECT id, user_id, meal_plan_id, week_start_date, generated_at
        FROM shopping_lists
        WHERE user_id = ? AND week_start_date = ?
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(week_start_date)
    .fetch_optional(pool)
    .await?;

    if let Some(header) = header {
        // Query all items
        let items: Vec<ShoppingListItemRow> = sqlx::query_as::<_, ShoppingListItemRow>(
            r#"
            SELECT id, shopping_list_id, ingredient_name, quantity, unit, category, is_collected
            FROM shopping_list_items
            WHERE shopping_list_id = ?
            ORDER BY category, ingredient_name
            "#,
        )
        .bind(&header.id)
        .fetch_all(pool)
        .await?;

        Ok(Some(ShoppingListData { header, items }))
    } else {
        Ok(None)
    }
}

/// Shopping list data structure (header + items)
#[derive(Debug, Clone)]
pub struct ShoppingListData {
    pub header: ShoppingListHeaderRow,
    pub items: Vec<ShoppingListItemRow>,
}

/// Shopping list header row from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShoppingListHeaderRow {
    pub id: String,
    pub user_id: String,
    pub meal_plan_id: String,
    pub week_start_date: String,
    pub generated_at: String,
}

/// Shopping list item row from database
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ShoppingListItemRow {
    pub id: String,
    pub shopping_list_id: String,
    pub ingredient_name: String,
    pub quantity: f32,
    pub unit: String,
    pub category: String,
    pub is_collected: bool,
}

impl ShoppingListData {
    /// Group items by category for display
    ///
    /// Returns a map of category name -> items in that category
    pub fn group_by_category(
        &self,
    ) -> std::collections::HashMap<String, Vec<&ShoppingListItemRow>> {
        let mut groups: std::collections::HashMap<String, Vec<&ShoppingListItemRow>> =
            std::collections::HashMap::new();

        for item in &self.items {
            groups.entry(item.category.clone()).or_default().push(item);
        }

        groups
    }
}

/// Create shopping list projection subscription
///
/// This sets up the evento subscription to project ShoppingListGenerated events
/// into the shopping_lists and shopping_list_items read model tables.
pub fn shopping_projection(pool: sqlx::SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("shopping-read-model")
        .aggregator::<ShoppingListAggregate>()
        .data(pool)
        .handler(project_shopping_list_generated())
}
