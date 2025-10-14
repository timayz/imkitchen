use crate::aggregate::RecipeAggregate;
use crate::error::RecipeResult;
use crate::events::{RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeUpdated};
use evento::{AggregatorName, Context, EventDetails, Executor};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Recipe data from read model (recipes table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeReadModel {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub ingredients: String,  // JSON
    pub instructions: String, // JSON
    pub prep_time_min: Option<i32>,
    pub cook_time_min: Option<i32>,
    pub advance_prep_hours: Option<i32>,
    pub serving_size: Option<i32>,
    pub is_favorite: bool,
    pub is_shared: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Async evento subscription handler for RecipeCreated events
///
/// This handler projects RecipeCreated events from the evento event store
/// into the recipes read model table for efficient querying.
#[evento::handler(RecipeAggregate)]
async fn recipe_created_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Serialize ingredients and instructions to JSON
    let ingredients_json = serde_json::to_string(&event.data.ingredients)?;
    let instructions_json = serde_json::to_string(&event.data.instructions)?;

    // Execute SQL insert to project event into read model
    // Use event.aggregator_id as the primary key (recipe id)
    // Default is_shared to false (private) per AC-10
    sqlx::query(
        r#"
        INSERT INTO recipes (
            id, user_id, title, ingredients, instructions,
            prep_time_min, cook_time_min, advance_prep_hours, serving_size,
            is_favorite, is_shared, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, 0, ?10, ?10)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.title)
    .bind(&ingredients_json)
    .bind(&instructions_json)
    .bind(event.data.prep_time_min.map(|v| v as i32))
    .bind(event.data.cook_time_min.map(|v| v as i32))
    .bind(event.data.advance_prep_hours.map(|v| v as i32))
    .bind(event.data.serving_size.map(|v| v as i32))
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for RecipeDeleted events
///
/// This handler soft-deletes recipes from the read model by removing them from the table.
/// The events remain in the event store for audit trail.
#[evento::handler(RecipeAggregate)]
async fn recipe_deleted_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL delete to remove recipe from read model
    // This is a soft delete - events remain in the event store
    sqlx::query("DELETE FROM recipes WHERE id = ?1")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Async evento subscription handler for RecipeFavorited events
///
/// This handler updates the is_favorite flag in the recipes read model table.
#[evento::handler(RecipeAggregate)]
async fn recipe_favorited_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL update to toggle favorite status
    sqlx::query("UPDATE recipes SET is_favorite = ?1 WHERE id = ?2")
        .bind(event.data.favorited)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Async evento subscription handler for RecipeUpdated events
///
/// This handler updates the recipes read model table with changed fields (delta pattern).
/// Only fields that are present in the event (Some value) are updated in the read model.
///
/// Note: Uses dynamic SQL construction with parameterized bindings. The column names are
/// hardcoded (not user input), and all values are bound via SQLx parameters, making this
/// approach safe from SQL injection while maintaining optimal performance (single UPDATE).
#[evento::handler(RecipeAggregate)]
async fn recipe_updated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeUpdated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Build dynamic UPDATE query based on which fields are present in the event
    let mut updates = Vec::new();
    let mut update_query = String::from("UPDATE recipes SET ");

    if event.data.title.is_some() {
        updates.push("title = ?");
    }
    if event.data.ingredients.is_some() {
        updates.push("ingredients = ?");
    }
    if event.data.instructions.is_some() {
        updates.push("instructions = ?");
    }
    if event.data.prep_time_min.is_some() {
        updates.push("prep_time_min = ?");
    }
    if event.data.cook_time_min.is_some() {
        updates.push("cook_time_min = ?");
    }
    if event.data.advance_prep_hours.is_some() {
        updates.push("advance_prep_hours = ?");
    }
    if event.data.serving_size.is_some() {
        updates.push("serving_size = ?");
    }

    // Always update updated_at timestamp
    updates.push("updated_at = ?");

    update_query.push_str(&updates.join(", "));
    update_query.push_str(" WHERE id = ?");

    // Bind parameters in the same order as the updates
    let mut query = sqlx::query(&update_query);

    if let Some(ref title) = event.data.title {
        query = query.bind(title);
    }
    if let Some(ref ingredients) = event.data.ingredients {
        let ingredients_json = serde_json::to_string(ingredients)?;
        query = query.bind(ingredients_json);
    }
    if let Some(ref instructions) = event.data.instructions {
        let instructions_json = serde_json::to_string(instructions)?;
        query = query.bind(instructions_json);
    }
    if let Some(prep_time) = event.data.prep_time_min {
        query = query.bind(prep_time.map(|v| v as i32));
    }
    if let Some(cook_time) = event.data.cook_time_min {
        query = query.bind(cook_time.map(|v| v as i32));
    }
    if let Some(advance_prep) = event.data.advance_prep_hours {
        query = query.bind(advance_prep.map(|v| v as i32));
    }
    if let Some(serving_size) = event.data.serving_size {
        query = query.bind(serving_size.map(|v| v as i32));
    }

    // Bind updated_at and recipe_id
    query = query.bind(&event.data.updated_at);
    query = query.bind(&event.aggregator_id);

    query.execute(&pool).await?;

    Ok(())
}

/// Create recipe event subscription for read model projection
///
/// Returns a subscription builder that can be run with `.run(&executor).await`
///
/// Usage in main.rs:
/// ```no_run
/// # use sqlx::SqlitePool;
/// # use evento::Sqlite;
/// # async fn example(pool: SqlitePool, executor: Sqlite) -> anyhow::Result<()> {
/// recipe::recipe_projection(pool.clone()).run(&executor).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Cross-Domain Integration
///
/// **TODO (AC-6): meal_planning crate integration**
///
/// When the `meal_planning` crate is implemented, it should subscribe to `RecipeUpdated` events
/// to cascade changes to active meal plans that reference this recipe. This ensures that meal
/// plans immediately reflect recipe modifications (title, timing, etc.) without manual refresh.
///
/// Recommended implementation:
/// 1. Create `meal_planning::recipe_updated_cascade_handler()` subscription
/// 2. Listen for `RecipeUpdated` events (cross-aggregator subscription)
/// 3. Query `meal_assignments` table for recipes matching `event.aggregator_id`
/// 4. Update meal plan read model with refreshed recipe metadata
///
/// Reference: Story 2.2 AC-6 - "Updated recipe immediately reflects in meal plans (if currently scheduled)"
pub fn recipe_projection(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("recipe-read-model")
        .aggregator::<RecipeAggregate>()
        .data(pool)
        .handler(recipe_created_handler())
        .handler(recipe_deleted_handler())
        .handler(recipe_favorited_handler())
        .handler(recipe_updated_handler())
}

/// Query recipe by ID from read model
///
/// Returns Some(RecipeReadModel) if recipe exists and is not deleted, None otherwise
pub async fn query_recipe_by_id(
    recipe_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Option<RecipeReadModel>> {
    let result = sqlx::query(
        r#"
        SELECT id, user_id, title, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               is_favorite, is_shared, created_at, updated_at
        FROM recipes
        WHERE id = ?1
        "#,
    )
    .bind(recipe_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => {
            let recipe = RecipeReadModel {
                id: row.get("id"),
                user_id: row.get("user_id"),
                title: row.get("title"),
                ingredients: row.get("ingredients"),
                instructions: row.get("instructions"),
                prep_time_min: row.get("prep_time_min"),
                cook_time_min: row.get("cook_time_min"),
                advance_prep_hours: row.get("advance_prep_hours"),
                serving_size: row.get("serving_size"),
                is_favorite: row.get("is_favorite"),
                is_shared: row.get("is_shared"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(recipe))
        }
        None => Ok(None),
    }
}

/// Query all recipes for a user from read model
///
/// Returns list of recipes owned by the user (sorted by created_at descending)
pub async fn query_recipes_by_user(
    user_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Vec<RecipeReadModel>> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, title, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               is_favorite, is_shared, created_at, updated_at
        FROM recipes
        WHERE user_id = ?1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let recipes = rows
        .into_iter()
        .map(|row| RecipeReadModel {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            ingredients: row.get("ingredients"),
            instructions: row.get("instructions"),
            prep_time_min: row.get("prep_time_min"),
            cook_time_min: row.get("cook_time_min"),
            advance_prep_hours: row.get("advance_prep_hours"),
            serving_size: row.get("serving_size"),
            is_favorite: row.get("is_favorite"),
            is_shared: row.get("is_shared"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(recipes)
}
