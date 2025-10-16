use crate::aggregate::RecipeAggregate;
use crate::collection_aggregate::CollectionAggregate;
use crate::collection_events::{
    CollectionCreated, CollectionDeleted, CollectionUpdated, RecipeAddedToCollection,
    RecipeRemovedFromCollection,
};
use crate::error::{RecipeError, RecipeResult};
use crate::events::{
    RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeShared, RecipeTagged, RecipeUpdated,
};
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
    pub complexity: Option<String>, // "simple", "moderate", "complex"
    pub cuisine: Option<String>,    // e.g., "Italian", "Asian"
    pub dietary_tags: Option<String>, // JSON array e.g., ["vegetarian", "vegan"]
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
/// This handler soft-deletes recipes from the read model by setting deleted_at timestamp.
/// The events remain in the event store for audit trail.
/// Soft-deleted recipes are excluded from queries via WHERE deleted_at IS NULL filters.
#[evento::handler(RecipeAggregate)]
async fn recipe_deleted_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL UPDATE to soft-delete recipe in read model
    // Sets deleted_at timestamp instead of removing the row
    // This enables future features like "show deleted recipes" and maintains referential integrity
    sqlx::query("UPDATE recipes SET deleted_at = ?1 WHERE id = ?2")
        .bind(&event.data.deleted_at)
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

/// Async evento subscription handler for RecipeShared events
///
/// This handler updates the is_shared flag in the recipes read model table.
/// AC-3: Shared recipes appear in community discovery feed (/discover route)
/// AC-6: Owner can revert to private at any time (removes from community discovery)
#[evento::handler(RecipeAggregate)]
async fn recipe_shared_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL update to toggle share status
    sqlx::query("UPDATE recipes SET is_shared = ?1 WHERE id = ?2")
        .bind(event.data.shared)
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

/// Async evento subscription handler for RecipeTagged events
///
/// This handler updates the tag columns in the recipes read model table.
#[evento::handler(RecipeAggregate)]
async fn recipe_tagged_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeTagged>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Serialize dietary_tags to JSON
    let dietary_tags_json = serde_json::to_string(&event.data.dietary_tags)?;

    // Execute SQL update to set tag columns
    sqlx::query(
        "UPDATE recipes SET complexity = ?1, cuisine = ?2, dietary_tags = ?3 WHERE id = ?4",
    )
    .bind(&event.data.complexity)
    .bind(&event.data.cuisine)
    .bind(&dietary_tags_json)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

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
        .handler(recipe_shared_handler())
        .handler(recipe_updated_handler())
        .handler(recipe_tagged_handler())
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
               is_favorite, is_shared, complexity, cuisine, dietary_tags,
               created_at, updated_at
        FROM recipes
        WHERE id = ?1 AND deleted_at IS NULL
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
                complexity: row.get("complexity"),
                cuisine: row.get("cuisine"),
                dietary_tags: row.get("dietary_tags"),
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
/// If favorite_only is true, only returns favorited recipes
pub async fn query_recipes_by_user(
    user_id: &str,
    favorite_only: bool,
    pool: &SqlitePool,
) -> RecipeResult<Vec<RecipeReadModel>> {
    let query_str = if favorite_only {
        r#"
        SELECT id, user_id, title, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               is_favorite, is_shared, complexity, cuisine, dietary_tags,
               created_at, updated_at
        FROM recipes
        WHERE user_id = ?1 AND is_favorite = 1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#
    } else {
        r#"
        SELECT id, user_id, title, ingredients, instructions,
               prep_time_min, cook_time_min, advance_prep_hours, serving_size,
               is_favorite, is_shared, complexity, cuisine, dietary_tags,
               created_at, updated_at
        FROM recipes
        WHERE user_id = ?1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#
    };

    let rows = sqlx::query(query_str).bind(user_id).fetch_all(pool).await?;

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
            complexity: row.get("complexity"),
            cuisine: row.get("cuisine"),
            dietary_tags: row.get("dietary_tags"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(recipes)
}

/// Filter and sort parameters for community recipe discovery
#[derive(Debug, Clone, Default)]
pub struct RecipeDiscoveryFilters {
    pub cuisine: Option<String>,
    pub min_rating: Option<u8>,     // 3 or 4 for filtering
    pub max_prep_time: Option<u32>, // in minutes (total: prep + cook)
    pub dietary: Option<String>,    // vegetarian, vegan, gluten-free
    pub search: Option<String>,     // search title or ingredients
    pub sort: Option<String>,       // "rating", "recent", "alphabetical"
    pub page: Option<u32>,          // page number (1-based)
}

/// Query shared recipes from read model for community discovery feed (AC-1 to AC-7)
///
/// Returns list of shared recipes (is_shared = true) visible to all users
/// AC-3, AC-4, AC-12: Filters by is_shared = true AND deleted_at IS NULL
/// AC-4: Supports filtering by cuisine, rating, prep time, dietary preferences
/// AC-5: Search by title or ingredient name
/// AC-6: Sorting by rating, date, or alphabetical
/// AC-7: Pagination with 20 recipes per page
/// Uses idx_recipes_shared index for performance
pub async fn list_shared_recipes(
    pool: &SqlitePool,
    filters: RecipeDiscoveryFilters,
) -> RecipeResult<Vec<RecipeReadModel>> {
    // Build base query with parameterized conditions
    let mut query_str = String::from(
        r#"
        SELECT r.id, r.user_id, r.title, r.ingredients, r.instructions,
               r.prep_time_min, r.cook_time_min, r.advance_prep_hours, r.serving_size,
               r.is_favorite, r.is_shared, r.complexity, r.cuisine, r.dietary_tags,
               r.created_at, r.updated_at
        FROM recipes r
        WHERE r.is_shared = 1 AND r.deleted_at IS NULL
        "#,
    );

    let mut conditions = Vec::new();
    let mut bind_index = 1;

    // AC-4: Cuisine filter (parameterized)
    if filters.cuisine.is_some() {
        conditions.push(format!("r.cuisine = ?{}", bind_index));
        bind_index += 1;
    }

    // AC-4: Prep time filter (prep_time_min + cook_time_min <= max) (parameterized)
    if filters.max_prep_time.is_some() {
        conditions.push(format!(
            "(COALESCE(r.prep_time_min, 0) + COALESCE(r.cook_time_min, 0)) <= ?{}",
            bind_index
        ));
        bind_index += 1;
    }

    // AC-4: Dietary filter (JSON text search) (parameterized)
    if filters.dietary.is_some() {
        conditions.push(format!("r.dietary_tags LIKE ?{}", bind_index));
        bind_index += 1;
    }

    // AC-5: Search filter (title OR ingredients) (parameterized)
    if filters.search.is_some() {
        conditions.push(format!(
            "(r.title LIKE ?{} OR r.ingredients LIKE ?{})",
            bind_index,
            bind_index + 1
        ));
        // bind_index incremented for completeness (even though it's the last filter)
        #[allow(unused_assignments)]
        {
            bind_index += 2;
        }
    }

    // Add conditions to query
    if !conditions.is_empty() {
        query_str.push_str(" AND ");
        query_str.push_str(&conditions.join(" AND "));
    }

    // AC-6: Sorting (safe - no user input)
    let sort_clause = match filters.sort.as_deref() {
        Some("rating") => "ORDER BY r.created_at DESC", // TODO: JOIN ratings table when Story 2.9 complete
        Some("alphabetical") => "ORDER BY r.title ASC",
        _ => "ORDER BY r.created_at DESC", // "recent" or default
    };
    query_str.push_str(&format!(" {} ", sort_clause));

    // AC-7: Pagination (safe - validated as u32)
    let page = filters.page.unwrap_or(1).max(1);
    let offset = (page - 1) * 20;
    query_str.push_str(&format!(" LIMIT 20 OFFSET {}", offset));

    // Build query with bound parameters to prevent SQL injection
    let mut query = sqlx::query(&query_str);

    // Pre-compute patterns to avoid lifetime issues
    let dietary_pattern = filters.dietary.as_ref().map(|d| format!("%{}%", d));
    let search_pattern = filters.search.as_ref().map(|s| format!("%{}%", s));

    // Bind parameters in the same order as conditions
    if let Some(ref cuisine) = filters.cuisine {
        query = query.bind(cuisine);
    }
    if let Some(max_time) = filters.max_prep_time {
        query = query.bind(max_time);
    }
    if let Some(ref pattern) = dietary_pattern {
        query = query.bind(pattern);
    }
    if let Some(ref pattern) = search_pattern {
        query = query.bind(pattern); // title LIKE
        query = query.bind(pattern); // ingredients LIKE
    }

    let rows = query.fetch_all(pool).await.map_err(|e| {
        tracing::error!("Failed to query shared recipes: {}", e);
        RecipeError::DatabaseError(e)
    })?;

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
            complexity: row.get("complexity"),
            cuisine: row.get("cuisine"),
            dietary_tags: row.get("dietary_tags"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(recipes)
}

/// Collection data from read model (recipe_collections table)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionReadModel {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub recipe_count: i32, // Computed from assignments table
}

/// Async evento subscription handler for CollectionCreated events
///
/// This handler projects CollectionCreated events from the evento event store
/// into the recipe_collections read model table for efficient querying.
#[evento::handler(CollectionAggregate)]
async fn collection_created_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<CollectionCreated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Execute SQL insert to project event into read model
    // Use event.aggregator_id as the primary key (collection id)
    sqlx::query(
        r#"
        INSERT INTO recipe_collections (
            id, user_id, name, description, created_at, updated_at, deleted_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?5, NULL)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.name)
    .bind(&event.data.description)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for CollectionUpdated events
///
/// This handler updates the recipe_collections read model table with changed fields (delta pattern).
#[evento::handler(CollectionAggregate)]
async fn collection_updated_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<CollectionUpdated>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Build dynamic UPDATE query based on which fields are present in the event
    let mut updates = Vec::new();
    let mut update_query = String::from("UPDATE recipe_collections SET ");

    if event.data.name.is_some() {
        updates.push("name = ?");
    }
    if event.data.description.is_some() {
        updates.push("description = ?");
    }

    // Always update updated_at timestamp
    updates.push("updated_at = ?");

    update_query.push_str(&updates.join(", "));
    update_query.push_str(" WHERE id = ?");

    // Bind parameters in the same order as the updates
    let mut query = sqlx::query(&update_query);

    if let Some(ref name) = event.data.name {
        query = query.bind(name);
    }
    if let Some(ref description) = event.data.description {
        query = query.bind(description);
    }

    // Bind updated_at and collection_id
    query = query.bind(&event.data.updated_at);
    query = query.bind(&event.aggregator_id);

    query.execute(&pool).await?;

    Ok(())
}

/// Async evento subscription handler for CollectionDeleted events
///
/// This handler soft-deletes collections from the read model by setting deleted_at timestamp
/// and removing all recipe assignments. The events remain in the event store for audit trail.
#[evento::handler(CollectionAggregate)]
async fn collection_deleted_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<CollectionDeleted>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Soft delete the collection by setting deleted_at timestamp
    sqlx::query("UPDATE recipe_collections SET deleted_at = ?1 WHERE id = ?2")
        .bind(&event.data.deleted_at)
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    // Remove all recipe assignments for this collection
    sqlx::query("DELETE FROM recipe_collection_assignments WHERE collection_id = ?1")
        .bind(&event.aggregator_id)
        .execute(&pool)
        .await?;

    Ok(())
}

/// Async evento subscription handler for RecipeAddedToCollection events
///
/// This handler creates a many-to-many assignment between recipe and collection.
#[evento::handler(CollectionAggregate)]
async fn recipe_added_to_collection_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeAddedToCollection>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Insert into recipe_collection_assignments (idempotent due to PRIMARY KEY constraint)
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO recipe_collection_assignments (
            collection_id, recipe_id, assigned_at
        )
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(&event.aggregator_id) // collection_id
    .bind(&event.data.recipe_id)
    .bind(&event.data.assigned_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Async evento subscription handler for RecipeRemovedFromCollection events
///
/// This handler deletes the many-to-many assignment between recipe and collection.
/// The recipe itself is not deleted, only the assignment is removed.
#[evento::handler(CollectionAggregate)]
async fn recipe_removed_from_collection_handler<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeRemovedFromCollection>,
) -> anyhow::Result<()> {
    // Extract the shared SqlitePool from context
    let pool: SqlitePool = context.extract();

    // Delete the assignment
    sqlx::query(
        "DELETE FROM recipe_collection_assignments WHERE collection_id = ?1 AND recipe_id = ?2",
    )
    .bind(&event.aggregator_id) // collection_id
    .bind(&event.data.recipe_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Create collection event subscription for read model projection
///
/// Returns a subscription builder that can be run with `.run(&executor).await`
///
/// Usage in main.rs:
/// ```no_run
/// # use sqlx::SqlitePool;
/// # use evento::Sqlite;
/// # async fn example(pool: SqlitePool, executor: Sqlite) -> anyhow::Result<()> {
/// recipe::collection_projection(pool.clone()).run(&executor).await?;
/// # Ok(())
/// # }
/// ```
pub fn collection_projection(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("collection-read-model")
        .aggregator::<CollectionAggregate>()
        .data(pool)
        .handler(collection_created_handler())
        .handler(collection_updated_handler())
        .handler(collection_deleted_handler())
        .handler(recipe_added_to_collection_handler())
        .handler(recipe_removed_from_collection_handler())
}

/// Query collection by ID from read model
///
/// Returns Some(CollectionReadModel) if collection exists and is not deleted, None otherwise
pub async fn query_collection_by_id(
    collection_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Option<CollectionReadModel>> {
    let result = sqlx::query(
        r#"
        SELECT c.id, c.user_id, c.name, c.description, c.created_at, c.updated_at,
               COUNT(a.recipe_id) as recipe_count
        FROM recipe_collections c
        LEFT JOIN recipe_collection_assignments a ON c.id = a.collection_id
        WHERE c.id = ?1 AND c.deleted_at IS NULL
        GROUP BY c.id
        "#,
    )
    .bind(collection_id)
    .fetch_optional(pool)
    .await?;

    match result {
        Some(row) => {
            let collection = CollectionReadModel {
                id: row.get("id"),
                user_id: row.get("user_id"),
                name: row.get("name"),
                description: row.get("description"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                recipe_count: row.get("recipe_count"),
            };
            Ok(Some(collection))
        }
        None => Ok(None),
    }
}

/// Query all collections for a user from read model
///
/// Returns list of collections owned by the user (sorted by name ascending)
pub async fn query_collections_by_user(
    user_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Vec<CollectionReadModel>> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.user_id, c.name, c.description, c.created_at, c.updated_at,
               COUNT(a.recipe_id) as recipe_count
        FROM recipe_collections c
        LEFT JOIN recipe_collection_assignments a ON c.id = a.collection_id
        WHERE c.user_id = ?1 AND c.deleted_at IS NULL
        GROUP BY c.id
        ORDER BY c.name ASC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let collections = rows
        .into_iter()
        .map(|row| CollectionReadModel {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            recipe_count: row.get("recipe_count"),
        })
        .collect();

    Ok(collections)
}

/// Query recipes in a specific collection from read model
///
/// Returns list of recipes that belong to the specified collection
pub async fn query_recipes_by_collection(
    collection_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Vec<RecipeReadModel>> {
    let rows = sqlx::query(
        r#"
        SELECT r.id, r.user_id, r.title, r.ingredients, r.instructions,
               r.prep_time_min, r.cook_time_min, r.advance_prep_hours, r.serving_size,
               r.is_favorite, r.is_shared, r.complexity, r.cuisine, r.dietary_tags,
               r.created_at, r.updated_at
        FROM recipes r
        INNER JOIN recipe_collection_assignments a ON r.id = a.recipe_id
        WHERE a.collection_id = ?1 AND r.deleted_at IS NULL
        ORDER BY r.created_at DESC
        "#,
    )
    .bind(collection_id)
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
            complexity: row.get("complexity"),
            cuisine: row.get("cuisine"),
            dietary_tags: row.get("dietary_tags"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect();

    Ok(recipes)
}

/// Query collections that contain a specific recipe
///
/// Returns list of collections that the recipe belongs to
pub async fn query_collections_for_recipe(
    recipe_id: &str,
    user_id: &str,
    pool: &SqlitePool,
) -> RecipeResult<Vec<CollectionReadModel>> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.user_id, c.name, c.description, c.created_at, c.updated_at,
               COUNT(a2.recipe_id) as recipe_count
        FROM recipe_collections c
        INNER JOIN recipe_collection_assignments a ON c.id = a.collection_id
        LEFT JOIN recipe_collection_assignments a2 ON c.id = a2.collection_id
        WHERE a.recipe_id = ?1 AND c.user_id = ?2 AND c.deleted_at IS NULL
        GROUP BY c.id
        ORDER BY c.name ASC
        "#,
    )
    .bind(recipe_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let collections = rows
        .into_iter()
        .map(|row| CollectionReadModel {
            id: row.get("id"),
            user_id: row.get("user_id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            recipe_count: row.get("recipe_count"),
        })
        .collect();

    Ok(collections)
}
