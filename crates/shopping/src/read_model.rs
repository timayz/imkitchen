use crate::aggregate::ShoppingListAggregate;
use crate::commands::ShoppingListError;
use crate::events::ShoppingListGenerated;
use chrono::{Datelike, NaiveDate, Utc};
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

/// Validate week_start_date parameter for shopping list queries
///
/// Validates that:
/// 1. The date is a valid ISO 8601 date
/// 2. The date is a Monday (ISO week start)
/// 3. The week is not in the past (current week or future only)
/// 4. The week is not more than 4 weeks in the future
///
/// Returns Ok(NaiveDate) if valid, otherwise ShoppingListError
pub fn validate_week_date(week_start_date: &str) -> Result<NaiveDate, ShoppingListError> {
    // Parse ISO 8601 date
    let date = NaiveDate::parse_from_str(week_start_date, "%Y-%m-%d")
        .map_err(|e| ShoppingListError::InvalidWeekError(format!("Invalid date format: {}", e)))?;

    // Check if date is a Monday (ISO week start)
    if date.weekday().num_days_from_monday() != 0 {
        return Err(ShoppingListError::InvalidWeekError(
            "Week start must be a Monday".to_string(),
        ));
    }

    // Get current week's Monday
    let today = Utc::now().date_naive();
    let current_week_monday =
        today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

    // Calculate week difference
    let days_diff = (date - current_week_monday).num_days();
    let weeks_diff = days_diff / 7;

    // Reject past weeks (MVP limitation per AC #7)
    if weeks_diff < 0 {
        return Err(ShoppingListError::PastWeekNotAccessibleError);
    }

    // Reject weeks more than 4 weeks in the future (AC #5)
    if weeks_diff > 4 {
        return Err(ShoppingListError::FutureWeekOutOfRangeError);
    }

    Ok(date)
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
///
/// This function validates the week_start_date before querying the database:
/// - Must be a valid ISO 8601 date (YYYY-MM-DD)
/// - Must be a Monday (ISO week start)
/// - Must not be in the past (current week or future only)
/// - Must not be more than 4 weeks in the future
///
/// Returns:
/// - Ok(Some(ShoppingListData)) if shopping list exists for the week
/// - Ok(None) if no shopping list exists for the week (but week is valid)
/// - Err if week validation fails (see ShoppingListError variants)
pub async fn get_shopping_list_by_week(
    user_id: &str,
    week_start_date: &str,
    pool: &sqlx::SqlitePool,
) -> Result<Option<ShoppingListData>, ShoppingListError> {
    // Validate week_start_date (AC #3, #5, #7)
    validate_week_date(week_start_date)?;

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
    .await
    .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

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
        .await
        .map_err(|e| ShoppingListError::EventStoreError(e.into()))?;

        Ok(Some(ShoppingListData { header, items }))
    } else {
        // No shopping list exists for this week (but week is valid) - AC #4
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
    /// Returns items grouped by category in a sorted vector where:
    /// - Categories are ordered by grocery store layout (Produce first, Other last)
    /// - Items within each category are sorted alphabetically by ingredient_name
    /// - Empty categories are excluded from the result
    ///
    /// Returns: Vec<(category_name, Vec<ShoppingListItemRow>)>
    pub fn group_by_category(&self) -> Vec<(String, Vec<&ShoppingListItemRow>)> {
        use std::collections::HashMap;

        // Group items by category
        let mut groups: HashMap<String, Vec<&ShoppingListItemRow>> = HashMap::new();
        for item in &self.items {
            groups.entry(item.category.clone()).or_default().push(item);
        }

        // Convert to vector and sort by category order
        let mut result: Vec<(String, Vec<&ShoppingListItemRow>)> = groups.into_iter().collect();

        // Sort categories by grocery store layout order
        result.sort_by(|a, b| {
            let order_a = Self::category_order(&a.0);
            let order_b = Self::category_order(&b.0);
            order_a.cmp(&order_b)
        });

        // Sort items within each category alphabetically
        for (_, items) in &mut result {
            items.sort_by(|a, b| a.ingredient_name.cmp(&b.ingredient_name));
        }

        result
    }

    /// Get the sort order for a category (lower = displayed first)
    ///
    /// Order matches typical grocery store layout:
    /// Produce(0), Dairy(1), Meat(2), Frozen(3), Pantry(4), Bakery(5), Other(6)
    fn category_order(category: &str) -> usize {
        match category {
            "Produce" => 0,
            "Dairy" => 1,
            "Meat" => 2,
            "Frozen" => 3,
            "Pantry" => 4,
            "Bakery" => 5,
            "Other" => 6,
            _ => 999, // Unknown categories go to the end
        }
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
