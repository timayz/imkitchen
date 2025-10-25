/// Page-Specific Projection Handlers for Recipe Domain
///
/// This module contains evento projection handlers that populate page-specific read model tables
/// (recipe_list, recipe_detail, recipe_filter_counts, recipe_ratings) from recipe domain events.
///
/// Pattern: One domain event may trigger MULTIPLE projection handlers, each updating a different
/// page-specific table. Handlers are independent and idempotent.
use crate::aggregate::RecipeAggregate;
use crate::events::{
    RecipeCopied, RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeRated, RecipeShared,
    RecipeTagged, RecipeUpdated,
};
use evento::{AggregatorName, Context, EventDetails, Executor};
use sqlx::SqlitePool;

/// Create evento subscription for recipe_list and recipe_filter_counts projections
///
/// Registers handlers that populate recipe_list and recipe_filter_counts tables.
/// This subscription handles the Recipe Library page data.
pub fn recipe_list_projections(pool: SqlitePool) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("recipe-list-projections")
        .aggregator::<RecipeAggregate>()
        .data(pool)
        .handler(handle_recipe_created_for_list())
        .handler(handle_recipe_tagged_for_list())
        .handler(update_recipe_list_favorite())
        .handler(update_recipe_list_shared())
        .handler(handle_recipe_deleted_for_list())
        .skip::<RecipeAggregate, RecipeRated>()
        .skip::<RecipeAggregate, RecipeUpdated>()
        .skip::<RecipeAggregate, RecipeCopied>()
}

/// Create evento subscription for recipe_detail projections
///
/// Registers handlers that populate recipe_detail table.
/// This subscription handles the Recipe Detail page data.
pub fn recipe_detail_projections(
    pool: SqlitePool,
) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("recipe-detail-projections")
        .aggregator::<RecipeAggregate>()
        .data(pool)
        .handler(project_recipe_to_detail_view())
        .handler(update_recipe_detail_tags())
        .handler(update_recipe_detail_favorite())
        .handler(update_recipe_detail_shared())
        .handler(soft_delete_recipe_detail())
        .skip::<RecipeAggregate, RecipeRated>()
        .skip::<RecipeAggregate, RecipeUpdated>()
        .skip::<RecipeAggregate, RecipeCopied>()
}

/// Create evento subscription for recipe_ratings projections
///
/// Registers handlers that populate recipe_ratings table.
/// This subscription handles aggregated rating statistics.
pub fn recipe_ratings_projections(
    pool: SqlitePool,
) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("recipe-ratings-projections")
        .aggregator::<RecipeAggregate>()
        .data(pool)
        .handler(initialize_recipe_ratings())
        .handler(update_recipe_ratings_on_rating())
        .skip::<RecipeAggregate, RecipeTagged>()
        .skip::<RecipeAggregate, RecipeFavorited>()
        .skip::<RecipeAggregate, RecipeShared>()
        .skip::<RecipeAggregate, RecipeDeleted>()
        .skip::<RecipeAggregate, RecipeUpdated>()
        .skip::<RecipeAggregate, RecipeCopied>()
}

// =============================================================================
// RECIPE LIST PAGE PROJECTIONS (recipe_list, recipe_filter_counts)
// =============================================================================

/// Project RecipeCreated event to recipe_list table AND increment filter counts
///
/// This handler combines two operations that both need to run on RecipeCreated:
/// 1. Insert into recipe_list table (for Recipe Library page)
/// 2. Increment recipe_type filter count
///
/// Note: evento only allows ONE handler per event type per subscription, so we must
/// combine multiple operations into a single handler function.
#[evento::handler(RecipeAggregate)]
pub async fn handle_recipe_created_for_list<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Operation 1: Insert into recipe_list
    sqlx::query(
        r#"
        INSERT INTO recipe_list (
            id, user_id, title, recipe_type, complexity, cuisine, dietary_tags,
            prep_time_min, cook_time_min, is_favorite, is_shared,
            avg_rating, rating_count, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, NULL, NULL, NULL, ?5, ?6, 0, 0, 0.0, 0, ?7, ?7)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.title)
    .bind(&event.data.recipe_type)
    .bind(event.data.prep_time_min.map(|v| v as i32))
    .bind(event.data.cook_time_min.map(|v| v as i32))
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    // Operation 2: Increment recipe_type filter count
    sqlx::query(
        r#"
        INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count, updated_at)
        VALUES (?1, 'recipe_type', ?2, 1, ?3)
        ON CONFLICT(user_id, filter_type, filter_value)
        DO UPDATE SET count = count + 1, updated_at = ?3
        "#,
    )
    .bind(&event.data.user_id)
    .bind(&event.data.recipe_type)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Handle RecipeTagged event: update recipe_list AND increment filter counts
///
/// This handler combines two operations that both need to run on RecipeTagged:
/// 1. Update recipe_list with complexity, cuisine, dietary_tags
/// 2. Increment filter counts for each tag value
///
/// Note: evento only allows ONE handler per event type per subscription
#[evento::handler(RecipeAggregate)]
pub async fn handle_recipe_tagged_for_list<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeTagged>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let dietary_tags_json = if event.data.dietary_tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&event.data.dietary_tags)?)
    };

    // Operation 1: Update recipe_list
    sqlx::query(
        r#"
        UPDATE recipe_list
        SET complexity = ?1, cuisine = ?2, dietary_tags = ?3, updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(&event.data.complexity)
    .bind(&event.data.cuisine)
    .bind(&dietary_tags_json)
    .bind(&event.data.tagged_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    // Operation 2: Increment filter counts
    // Get user_id from recipe_detail (should always exist since RecipeCreated fires first)
    // Note: We query recipe_detail instead of recipe_list to handle timing between projections
    let user_id: Option<String> = sqlx::query_scalar("SELECT user_id FROM recipe_detail WHERE id = ?")
        .bind(&event.aggregator_id)
        .fetch_optional(&pool)
        .await?;

    // If recipe doesn't exist yet (shouldn't happen since events are ordered), skip gracefully
    let Some(user_id) = user_id else {
        tracing::warn!(
            recipe_id = %event.aggregator_id,
            "RecipeTagged fired but recipe not found in recipe_detail - skipping filter counts"
        );
        return Ok(());
    };

    // Increment complexity filter count (if present)
    if let Some(complexity) = &event.data.complexity {
        sqlx::query(
            r#"
            INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count, updated_at)
            VALUES (?1, 'complexity', ?2, 1, ?3)
            ON CONFLICT(user_id, filter_type, filter_value)
            DO UPDATE SET count = count + 1, updated_at = ?3
            "#,
        )
        .bind(&user_id)
        .bind(complexity)
        .bind(&event.data.tagged_at)
        .execute(&pool)
        .await?;
    }

    // Increment cuisine filter count (if present)
    if let Some(cuisine) = &event.data.cuisine {
        sqlx::query(
            r#"
            INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count, updated_at)
            VALUES (?1, 'cuisine', ?2, 1, ?3)
            ON CONFLICT(user_id, filter_type, filter_value)
            DO UPDATE SET count = count + 1, updated_at = ?3
            "#,
        )
        .bind(&user_id)
        .bind(cuisine)
        .bind(&event.data.tagged_at)
        .execute(&pool)
        .await?;
    }

    // Increment dietary_tag filter counts (for each tag)
    for tag in &event.data.dietary_tags {
        sqlx::query(
            r#"
            INSERT INTO recipe_filter_counts (user_id, filter_type, filter_value, count, updated_at)
            VALUES (?1, 'dietary_tag', ?2, 1, ?3)
            ON CONFLICT(user_id, filter_type, filter_value)
            DO UPDATE SET count = count + 1, updated_at = ?3
            "#,
        )
        .bind(&user_id)
        .bind(tag)
        .bind(&event.data.tagged_at)
        .execute(&pool)
        .await?;
    }

    Ok(())
}

/// Update recipe_list when RecipeFavorited event fires
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_list_favorite<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE recipe_list
        SET is_favorite = ?1, updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(if event.data.favorited { 1 } else { 0 })
    .bind(&event.data.toggled_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Update recipe_list when RecipeShared event fires
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_list_shared<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE recipe_list
        SET is_shared = ?1, updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(if event.data.shared { 1 } else { 0 })
    .bind(&event.data.toggled_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Handle RecipeDeleted event: soft-delete from recipe_list AND decrement filter counts
///
/// This handler combines two operations that both need to run on RecipeDeleted:
/// 1. Soft-delete recipe from recipe_list (set deleted_at)
/// 2. Decrement filter counts for recipe's tags
///
/// Note: evento only allows ONE handler per event type per subscription
#[evento::handler(RecipeAggregate)]
pub async fn handle_recipe_deleted_for_list<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

#[allow(clippy::type_complexity)]
    // Operation 1: Fetch recipe data BEFORE soft-delete (for filter count decrements)
    let recipe: Option<(String, Option<String>, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT recipe_type, complexity, cuisine, dietary_tags FROM recipe_list WHERE id = ?",
    )
    .bind(&event.aggregator_id)
    .fetch_optional(&pool)
    .await?;

    // Operation 2: Soft-delete the recipe
    sqlx::query(
        r#"
        UPDATE recipe_list
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(&event.data.deleted_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    // Operation 3: Decrement filter counts
    if let Some((recipe_type, complexity, cuisine, dietary_tags_json)) = recipe {
        // Decrement recipe_type count
        sqlx::query(
            r#"
            UPDATE recipe_filter_counts
            SET count = count - 1, updated_at = ?1
            WHERE user_id = ?2 AND filter_type = 'recipe_type' AND filter_value = ?3
            "#,
        )
        .bind(&event.data.deleted_at)
        .bind(&event.data.user_id)
        .bind(&recipe_type)
        .execute(&pool)
        .await?;

        // Decrement complexity count (if present)
        if let Some(complexity) = complexity {
            sqlx::query(
                r#"
                UPDATE recipe_filter_counts
                SET count = count - 1, updated_at = ?1
                WHERE user_id = ?2 AND filter_type = 'complexity' AND filter_value = ?3
                "#,
            )
            .bind(&event.data.deleted_at)
            .bind(&event.data.user_id)
            .bind(&complexity)
            .execute(&pool)
            .await?;
        }

        // Decrement cuisine count (if present)
        if let Some(cuisine) = cuisine {
            sqlx::query(
                r#"
                UPDATE recipe_filter_counts
                SET count = count - 1, updated_at = ?1
                WHERE user_id = ?2 AND filter_type = 'cuisine' AND filter_value = ?3
                "#,
            )
            .bind(&event.data.deleted_at)
            .bind(&event.data.user_id)
            .bind(&cuisine)
            .execute(&pool)
            .await?;
        }

        // Decrement dietary_tag counts (if present)
        if let Some(dietary_tags_json) = dietary_tags_json {
            let dietary_tags: Vec<String> = serde_json::from_str(&dietary_tags_json)?;
            for tag in dietary_tags {
                sqlx::query(
                    r#"
                    UPDATE recipe_filter_counts
                    SET count = count - 1, updated_at = ?1
                    WHERE user_id = ?2 AND filter_type = 'dietary_tag' AND filter_value = ?3
                    "#,
                )
                .bind(&event.data.deleted_at)
                .bind(&event.data.user_id)
                .bind(&tag)
                .execute(&pool)
                .await?;
            }
        }
    }

    Ok(())
}

// =============================================================================
// RECIPE DETAIL PAGE PROJECTIONS (recipe_detail, recipe_ratings)
// =============================================================================

/// Project RecipeCreated event to recipe_detail table (Recipe Detail page)
///
/// Inserts FULL recipe data needed for detail view (ingredients, instructions, etc.)
#[evento::handler(RecipeAggregate)]
pub async fn project_recipe_to_detail_view<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let ingredients_json = serde_json::to_string(&event.data.ingredients)?;
    let instructions_json = serde_json::to_string(&event.data.instructions)?;

    sqlx::query(
        r#"
        INSERT INTO recipe_detail (
            id, user_id, title, recipe_type, ingredients, instructions,
            prep_time_min, cook_time_min, advance_prep_hours, serving_size,
            complexity, cuisine, dietary_tags, is_favorite, is_shared,
            created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, NULL, NULL, 0, 0, ?11, ?11)
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.user_id)
    .bind(&event.data.title)
    .bind(&event.data.recipe_type)
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

/// Update recipe_detail when RecipeTagged event fires
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_detail_tags<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeTagged>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    let dietary_tags_json = if event.data.dietary_tags.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&event.data.dietary_tags)?)
    };

    sqlx::query(
        r#"
        UPDATE recipe_detail
        SET complexity = ?1, cuisine = ?2, dietary_tags = ?3, updated_at = ?4
        WHERE id = ?5
        "#,
    )
    .bind(&event.data.complexity)
    .bind(&event.data.cuisine)
    .bind(&dietary_tags_json)
    .bind(&event.data.tagged_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Update recipe_detail when RecipeFavorited event fires
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_detail_favorite<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE recipe_detail
        SET is_favorite = ?1, updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(if event.data.favorited { 1 } else { 0 })
    .bind(&event.data.toggled_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Update recipe_detail when RecipeShared event fires
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_detail_shared<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE recipe_detail
        SET is_shared = ?1, updated_at = ?2
        WHERE id = ?3
        "#,
    )
    .bind(if event.data.shared { 1 } else { 0 })
    .bind(&event.data.toggled_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Soft-delete recipe from recipe_detail when RecipeDeleted event fires
#[evento::handler(RecipeAggregate)]
pub async fn soft_delete_recipe_detail<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        UPDATE recipe_detail
        SET deleted_at = ?1
        WHERE id = ?2
        "#,
    )
    .bind(&event.data.deleted_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Initialize recipe_ratings entry when RecipeCreated fires
#[evento::handler(RecipeAggregate)]
pub async fn initialize_recipe_ratings<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    sqlx::query(
        r#"
        INSERT INTO recipe_ratings (
            recipe_id, avg_stars, rating_count,
            five_star_count, four_star_count, three_star_count, two_star_count, one_star_count,
            recent_reviews, updated_at
        )
        VALUES (?1, 0.0, 0, 0, 0, 0, 0, 0, '[]', ?2)
        ON CONFLICT(recipe_id) DO NOTHING
        "#,
    )
    .bind(&event.aggregator_id)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Update recipe_ratings when RecipeRated event fires (aggregate ratings)
#[evento::handler(RecipeAggregate)]
pub async fn update_recipe_ratings_on_rating<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeRated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Increment star bucket count based on stars value
    let star_column = match event.data.stars {
        5 => "five_star_count",
        4 => "four_star_count",
        3 => "three_star_count",
        2 => "two_star_count",
        1 => "one_star_count",
        _ => return Err(anyhow::anyhow!("Invalid star rating: {}", event.data.stars)),
    };

    // Update ratings: increment count, increment star bucket, recalculate avg
    sqlx::query(&format!(
        r#"
        UPDATE recipe_ratings
        SET rating_count = rating_count + 1,
            {} = {} + 1,
            avg_stars = (
                (avg_stars * rating_count) + ?1
            ) / (rating_count + 1),
            updated_at = ?2
        WHERE recipe_id = ?3
        "#,
        star_column, star_column
    ))
    .bind(event.data.stars as f64)
    .bind(&event.data.rated_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    // Also update avg_rating in recipe_list
    let avg_stars: f64 =
        sqlx::query_scalar("SELECT avg_stars FROM recipe_ratings WHERE recipe_id = ?")
            .bind(&event.aggregator_id)
            .fetch_one(&pool)
            .await?;

    let rating_count: i32 =
        sqlx::query_scalar("SELECT rating_count FROM recipe_ratings WHERE recipe_id = ?")
            .bind(&event.aggregator_id)
            .fetch_one(&pool)
            .await?;

    sqlx::query(
        "UPDATE recipe_list SET avg_rating = ?1, rating_count = ?2, updated_at = ?3 WHERE id = ?4",
    )
    .bind(avg_stars)
    .bind(rating_count)
    .bind(&event.data.rated_at)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    // Update recent_reviews in recipe_ratings (add new review, keep top 10 most recent)
    #[derive(serde::Serialize, serde::Deserialize)]
    struct RecentReview {
        user_id: String,
        stars: i32,
        review_text: Option<String>,
        created_at: String,
    }

    // Fetch current recent_reviews
    let current_reviews_json: String = sqlx::query_scalar(
        "SELECT recent_reviews FROM recipe_ratings WHERE recipe_id = ?"
    )
    .bind(&event.aggregator_id)
    .fetch_one(&pool)
    .await?;

    let mut recent_reviews: Vec<RecentReview> = serde_json::from_str(&current_reviews_json)
        .unwrap_or_default();

    // Add new review at the beginning
    recent_reviews.insert(0, RecentReview {
        user_id: event.data.user_id.clone(),
        stars: event.data.stars,
        review_text: event.data.review_text.clone(),
        created_at: event.data.rated_at.clone(),
    });

    // Keep only the 10 most recent
    recent_reviews.truncate(10);

    let recent_reviews_json = serde_json::to_string(&recent_reviews)?;

    sqlx::query(
        "UPDATE recipe_ratings SET recent_reviews = ?1 WHERE recipe_id = ?2"
    )
    .bind(&recent_reviews_json)
    .bind(&event.aggregator_id)
    .execute(&pool)
    .await?;

    Ok(())
}

// =============================================================================
// DASHBOARD METRICS PROJECTIONS
// =============================================================================

/// Create evento subscription for dashboard_metrics projections
///
/// Registers handlers that populate dashboard_metrics table (user recipe counts).
/// This subscription handles the Dashboard page metrics.
pub fn recipe_dashboard_metrics_projections(
    pool: SqlitePool,
) -> evento::SubscribeBuilder<evento::Sqlite> {
    evento::subscribe("recipe-dashboard-metrics-projections")
        .aggregator::<RecipeAggregate>()
        .data(pool)
        .handler(initialize_dashboard_metrics_on_recipe_created())
        .handler(decrement_recipe_count_on_delete())
        .handler(update_favorite_count_on_toggle())
        .handler(update_shared_count_on_toggle())
        .handler(update_cuisine_variety_on_tagged())
        .skip::<RecipeAggregate, RecipeRated>()
        .skip::<RecipeAggregate, RecipeUpdated>()
        .skip::<RecipeAggregate, RecipeCopied>()
}

/// Initialize dashboard_metrics row for user when they create their first recipe
/// Also increment recipe_count
#[evento::handler(RecipeAggregate)]
pub async fn initialize_dashboard_metrics_on_recipe_created<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeCreated>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Insert or update dashboard_metrics
    sqlx::query(
        r#"INSERT INTO dashboard_metrics (user_id, recipe_count, favorite_count, cuisine_variety_count, updated_at)
           VALUES (?1, 1, 0, 0, ?2)
           ON CONFLICT(user_id)
           DO UPDATE SET recipe_count = recipe_count + 1, updated_at = ?2"#,
    )
    .bind(&event.data.user_id)
    .bind(&event.data.created_at)
    .execute(&pool)
    .await?;

    Ok(())
}

/// Decrement recipe_count when recipe is deleted
#[evento::handler(RecipeAggregate)]
pub async fn decrement_recipe_count_on_delete<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeDeleted>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Get user_id from recipe_detail
    let user_id: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM recipe_detail WHERE id = ?")
            .bind(&event.aggregator_id)
            .fetch_optional(&pool)
            .await?;

    if let Some(user_id) = user_id {
        sqlx::query(
            r#"UPDATE dashboard_metrics
               SET recipe_count = MAX(0, recipe_count - 1),
                   updated_at = ?1
               WHERE user_id = ?2"#,
        )
        .bind(&event.data.deleted_at)
        .bind(&user_id)
        .execute(&pool)
        .await?;
    }

    Ok(())
}

/// Update favorite_count when recipe favorite status changes
/// Handles both favoriting (increment) and unfavoriting (decrement)
#[evento::handler(RecipeAggregate)]
pub async fn update_favorite_count_on_toggle<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeFavorited>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Get user_id from recipe_detail
    let user_id: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM recipe_detail WHERE id = ?")
            .bind(&event.aggregator_id)
            .fetch_optional(&pool)
            .await?;

    if let Some(user_id) = user_id {
        if event.data.favorited {
            // Increment when favorited
            sqlx::query(
                r#"UPDATE dashboard_metrics
                   SET favorite_count = favorite_count + 1,
                       updated_at = ?1
                   WHERE user_id = ?2"#,
            )
            .bind(&event.data.toggled_at)
            .bind(&user_id)
            .execute(&pool)
            .await?;
        } else {
            // Decrement when unfavorited
            sqlx::query(
                r#"UPDATE dashboard_metrics
                   SET favorite_count = MAX(0, favorite_count - 1),
                       updated_at = ?1
                   WHERE user_id = ?2"#,
            )
            .bind(&event.data.toggled_at)
            .bind(&user_id)
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}

/// Update cuisine_variety_count when recipe is tagged with cuisine
#[evento::handler(RecipeAggregate)]
pub async fn update_cuisine_variety_on_tagged<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeTagged>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Only process if cuisine is set
    if event.data.cuisine.is_none() {
        return Ok(());
    }

    // Get user_id from recipe_detail
    let user_id: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM recipe_detail WHERE id = ?")
            .bind(&event.aggregator_id)
            .fetch_optional(&pool)
            .await?;

    if let Some(user_id) = user_id {
        // Count distinct cuisines for this user
        let cuisine_count: i32 = sqlx::query_scalar(
            r#"SELECT COUNT(DISTINCT cuisine)
               FROM recipe_detail
               WHERE user_id = ?1
                 AND cuisine IS NOT NULL
                 AND deleted_at IS NULL"#,
        )
        .bind(&user_id)
        .fetch_one(&pool)
        .await?;

        sqlx::query(
            r#"UPDATE dashboard_metrics
               SET cuisine_variety_count = ?1,
                   updated_at = ?2
               WHERE user_id = ?3"#,
        )
        .bind(cuisine_count)
        .bind(&event.data.tagged_at)
        .bind(&user_id)
        .execute(&pool)
        .await?;
    }

    Ok(())
}

/// Update shared_count when recipe share status changes
/// Handles both sharing (increment) and unsharing (decrement)
#[evento::handler(RecipeAggregate)]
pub async fn update_shared_count_on_toggle<E: Executor>(
    context: &Context<'_, E>,
    event: EventDetails<RecipeShared>,
) -> anyhow::Result<()> {
    let pool: SqlitePool = context.extract();

    // Get user_id from recipe_detail
    let user_id: Option<String> =
        sqlx::query_scalar("SELECT user_id FROM recipe_detail WHERE id = ?")
            .bind(&event.aggregator_id)
            .fetch_optional(&pool)
            .await?;

    if let Some(user_id) = user_id {
        if event.data.shared {
            // Increment when shared
            sqlx::query(
                r#"UPDATE dashboard_metrics
                   SET shared_count = shared_count + 1,
                       updated_at = ?1
                   WHERE user_id = ?2"#,
            )
            .bind(&event.data.toggled_at)
            .bind(&user_id)
            .execute(&pool)
            .await?;
        } else {
            // Decrement when unshared
            sqlx::query(
                r#"UPDATE dashboard_metrics
                   SET shared_count = MAX(0, shared_count - 1),
                       updated_at = ?1
                   WHERE user_id = ?2"#,
            )
            .bind(&event.data.toggled_at)
            .bind(&user_id)
            .execute(&pool)
            .await?;
        }
    }

    Ok(())
}
