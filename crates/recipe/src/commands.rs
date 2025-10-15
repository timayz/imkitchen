use chrono::Utc;
use evento::Sqlite;
use sqlx::{Row, SqlitePool};
use tracing;
use validator::Validate;

use crate::aggregate::RecipeAggregate;
use crate::error::{RecipeError, RecipeResult};
use crate::events::{
    Ingredient, InstructionStep, RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeShared,
    RecipeTagged, RecipeUpdated,
};
use crate::tagging::{CuisineInferenceService, DietaryTagDetector, RecipeComplexityCalculator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Title must be between 3 and 200 characters"
    ))]
    pub title: String,

    #[validate(length(min = 1, message = "At least 1 ingredient is required"))]
    pub ingredients: Vec<Ingredient>,

    #[validate(length(min = 1, message = "At least 1 instruction step is required"))]
    pub instructions: Vec<InstructionStep>,

    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,
}

/// Create a new recipe using evento event sourcing pattern
///
/// 1. Validates command fields
/// 2. Checks user tier and recipe count (free tier limited to 10 recipes)
/// 3. Creates and commits RecipeCreated event to evento event store
/// 4. Event automatically projected to read model via async subscription handler
/// 5. User domain also listens to increment recipe_count
/// 6. Returns recipe ID (evento aggregator_id)
///
/// Free tier enforcement: Queries users table to check tier and recipe_count.
/// Premium users bypass all limits. Free users limited to 10 recipes total.
pub async fn create_recipe(
    command: CreateRecipeCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<String> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    // Validate that ingredients list is not empty (validator doesn't catch empty Vec after deserialization)
    if command.ingredients.is_empty() {
        return Err(RecipeError::ValidationError(
            "At least 1 ingredient is required".to_string(),
        ));
    }

    // Validate that instructions list is not empty
    if command.instructions.is_empty() {
        return Err(RecipeError::ValidationError(
            "At least 1 instruction step is required".to_string(),
        ));
    }

    // Check user tier and recipe count for freemium enforcement
    // AC-11: Count only private recipes (shared recipes don't count toward limit)
    // Query users table to get tier, then count private recipes
    let user_result = sqlx::query("SELECT tier FROM users WHERE id = ?1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    match user_result {
        Some(user_row) => {
            let tier: String = user_row.get("tier");

            // Premium users bypass all limits
            if tier != "premium" {
                // AC-11: Count only private recipes (is_shared = false) that are not deleted
                let private_recipe_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM recipes WHERE user_id = ?1 AND is_shared = 0 AND deleted_at IS NULL"
                )
                .bind(user_id)
                .fetch_one(pool)
                .await?;

                // Free tier users limited to 10 private recipes
                if private_recipe_count >= 10 {
                    return Err(RecipeError::RecipeLimitReached);
                }
            }
        }
        None => {
            return Err(RecipeError::ValidationError("User not found".to_string()));
        }
    }

    let created_at = Utc::now();

    // Create RecipeCreated event and commit to evento event store
    // The async subscription handler will project to read model
    // evento::create() generates a ULID for the aggregator_id (recipe_id)
    let aggregator_id = evento::create::<RecipeAggregate>()
        .data(&RecipeCreated {
            user_id: user_id.to_string(),
            title: command.title,
            ingredients: command.ingredients,
            instructions: command.instructions,
            prep_time_min: command.prep_time_min,
            cook_time_min: command.cook_time_min,
            advance_prep_hours: command.advance_prep_hours,
            serving_size: command.serving_size,
            created_at: created_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Calculate and emit RecipeTagged event for automatic tagging
    // Load the aggregate to access the recipe data
    let load_result = evento::load::<RecipeAggregate, _>(executor, &aggregator_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    emit_recipe_tagged_event(&aggregator_id, &load_result.item, executor, false).await?;

    // Return the generated aggregator_id as the recipe_id
    Ok(aggregator_id)
}

/// Helper function to calculate tags and emit RecipeTagged event
///
/// This is called after RecipeCreated or RecipeUpdated events to automatically tag the recipe.
/// Skips tagging if manual_override flag is set.
async fn emit_recipe_tagged_event(
    recipe_id: &str,
    aggregate: &RecipeAggregate,
    executor: &Sqlite,
    manual_override: bool,
) -> RecipeResult<()> {
    // Skip automatic tagging if manual override is already set
    if aggregate.tags.manual_override && !manual_override {
        return Ok(());
    }

    // Calculate complexity using domain service
    let complexity = RecipeComplexityCalculator::calculate(
        &aggregate.ingredients,
        &aggregate.instructions,
        aggregate.advance_prep_hours,
    );

    // Infer cuisine using domain service
    let cuisine = CuisineInferenceService::infer(&aggregate.ingredients);

    // Detect dietary tags using domain service
    let dietary_tags = DietaryTagDetector::detect(&aggregate.ingredients);

    let tagged_at = Utc::now();

    // Emit RecipeTagged event
    evento::save::<RecipeAggregate>(recipe_id.to_string())
        .data(&RecipeTagged {
            complexity: Some(complexity.as_str().to_string()),
            cuisine,
            dietary_tags,
            manual_override,
            tagged_at: tagged_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRecipeCommand {
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
}

/// Delete a recipe using evento event sourcing pattern
///
/// 1. Verifies recipe ownership by checking read model
/// 2. Creates and commits RecipeDeleted event to evento event store
/// 3. Event automatically projected to read model via async subscription handler (soft delete)
/// 4. User domain also listens to decrement recipe_count
///
/// Permission check: Only the recipe owner can delete their recipe.
/// Note: Ownership is verified via read model query, not aggregate load
pub async fn delete_recipe(
    command: DeleteRecipeCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify recipe exists and check ownership via read model
    let recipe_result = sqlx::query("SELECT user_id FROM recipes WHERE id = ?1")
        .bind(&command.recipe_id)
        .fetch_optional(pool)
        .await?;

    match recipe_result {
        Some(row) => {
            let owner_id: String = row.get("user_id");
            if owner_id != command.user_id {
                return Err(RecipeError::PermissionDenied);
            }
        }
        None => {
            return Err(RecipeError::NotFound);
        }
    }

    let deleted_at = Utc::now();

    // Create RecipeDeleted event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeDeleted {
            user_id: command.user_id,
            deleted_at: deleted_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateRecipeCommand {
    pub recipe_id: String,
    pub user_id: String, // For ownership verification

    #[validate(length(
        min = 3,
        max = 200,
        message = "Title must be between 3 and 200 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(min = 1, message = "At least 1 ingredient is required"))]
    pub ingredients: Option<Vec<Ingredient>>,

    #[validate(length(min = 1, message = "At least 1 instruction step is required"))]
    pub instructions: Option<Vec<InstructionStep>>,

    pub prep_time_min: Option<Option<u32>>,
    pub cook_time_min: Option<Option<u32>>,
    pub advance_prep_hours: Option<Option<u32>>,
    pub serving_size: Option<Option<u32>>,
}

/// Update an existing recipe using evento event sourcing pattern
///
/// 1. Validates command fields
/// 2. Verifies recipe ownership via read model
/// 3. Creates and commits RecipeUpdated event with delta (changed fields only)
/// 4. Event automatically projected to read model via async subscription handler
/// 5. Returns () on success
///
/// Permission check: Only the recipe owner can update their recipe.
/// Note: Ownership is verified via read model query, not aggregate load
pub async fn update_recipe(
    command: UpdateRecipeCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    // Validate that ingredients list is not empty if provided
    if let Some(ref ingredients) = command.ingredients {
        if ingredients.is_empty() {
            return Err(RecipeError::ValidationError(
                "At least 1 ingredient is required".to_string(),
            ));
        }
    }

    // Validate that instructions list is not empty if provided
    if let Some(ref instructions) = command.instructions {
        if instructions.is_empty() {
            return Err(RecipeError::ValidationError(
                "At least 1 instruction step is required".to_string(),
            ));
        }
    }

    // Verify recipe exists and check ownership via read model
    let recipe_result = sqlx::query("SELECT user_id FROM recipes WHERE id = ?1")
        .bind(&command.recipe_id)
        .fetch_optional(pool)
        .await?;

    match recipe_result {
        Some(row) => {
            let owner_id: String = row.get("user_id");
            if owner_id != command.user_id {
                return Err(RecipeError::PermissionDenied);
            }
        }
        None => {
            return Err(RecipeError::NotFound);
        }
    }

    let updated_at = Utc::now();

    // Create RecipeUpdated event with only changed fields (delta pattern)
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeUpdated {
            title: command.title,
            ingredients: command.ingredients,
            instructions: command.instructions,
            prep_time_min: command.prep_time_min,
            cook_time_min: command.cook_time_min,
            advance_prep_hours: command.advance_prep_hours,
            serving_size: command.serving_size,
            updated_at: updated_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Calculate and emit RecipeTagged event for automatic tagging
    // Load the updated aggregate to access the latest recipe data
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    emit_recipe_tagged_event(&command.recipe_id, &load_result.item, executor, false).await?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecipeTagsCommand {
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
    pub complexity: Option<String>,
    pub cuisine: Option<String>,
    pub dietary_tags: Vec<String>,
}

/// Update recipe tags manually (with manual_override flag)
///
/// This allows users to override the automatically assigned tags.
/// Once tags are manually set, automatic tagging will be skipped on subsequent updates.
pub async fn update_recipe_tags(
    command: UpdateRecipeTagsCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify recipe exists and check ownership via read model
    let recipe_result = sqlx::query("SELECT user_id FROM recipes WHERE id = ?1")
        .bind(&command.recipe_id)
        .fetch_optional(pool)
        .await?;

    match recipe_result {
        Some(row) => {
            let owner_id: String = row.get("user_id");
            if owner_id != command.user_id {
                return Err(RecipeError::PermissionDenied);
            }
        }
        None => {
            return Err(RecipeError::NotFound);
        }
    }

    let tagged_at = Utc::now();

    // Emit RecipeTagged event with manual_override=true
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeTagged {
            complexity: command.complexity,
            cuisine: command.cuisine,
            dietary_tags: command.dietary_tags,
            manual_override: true,
            tagged_at: tagged_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteRecipeCommand {
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
}

/// Toggle favorite status of a recipe using evento event sourcing pattern
///
/// 1. Verifies recipe ownership via read model
/// 2. Loads recipe aggregate from event stream to get current is_favorite status
/// 3. Creates and commits RecipeFavorited event with toggled status
/// 4. Event automatically projected to read model via async subscription handler
/// 5. Returns the new favorite status (true/false)
///
/// Permission check: Only the recipe owner can favorite/unfavorite their recipe.
/// Note: Ownership is verified via read model query before loading aggregate
#[tracing::instrument(skip(executor, pool), fields(recipe_id = %command.recipe_id, user_id = %command.user_id))]
pub async fn favorite_recipe(
    command: FavoriteRecipeCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<bool> {
    // Verify recipe exists and check ownership via read model
    let recipe_result = sqlx::query("SELECT user_id FROM recipes WHERE id = ?1")
        .bind(&command.recipe_id)
        .fetch_optional(pool)
        .await?;

    match recipe_result {
        Some(row) => {
            let owner_id: String = row.get("user_id");
            if owner_id != command.user_id {
                return Err(RecipeError::PermissionDenied);
            }
        }
        None => {
            return Err(RecipeError::NotFound);
        }
    }

    // Load recipe aggregate to get current is_favorite status
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Toggle the favorite status
    let new_favorited_status = !load_result.item.is_favorite;

    let toggled_at = Utc::now();

    // Create RecipeFavorited event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeFavorited {
            user_id: command.user_id.clone(),
            favorited: new_favorited_status,
            toggled_at: toggled_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    tracing::info!(
        recipe_id = %command.recipe_id,
        favorited = new_favorited_status,
        "Recipe favorite status toggled"
    );

    // Return the new favorited status for UI updates
    Ok(new_favorited_status)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRecipeCommand {
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
    pub shared: bool,    // true = share with community, false = make private
}

/// Toggle share status of a recipe using evento event sourcing pattern
///
/// 1. Verifies recipe ownership via read model
/// 2. Creates and commits RecipeShared event with shared boolean parameter
/// 3. Event automatically projected to read model via async subscription handler
/// 4. Returns () on success
///
/// Permission check: Only the recipe owner can share/unshare their recipe.
/// Note: Ownership is verified via read model query, not aggregate load
///
/// This command handles both sharing (shared=true) and unsharing (shared=false).
/// AC-2: Toggle changes privacy from "private" to "shared" (RecipeShared event)
/// AC-6: Owner can revert to private at any time (removes from community discovery)
#[tracing::instrument(skip(executor, pool), fields(recipe_id = %command.recipe_id, user_id = %command.user_id, shared = %command.shared))]
pub async fn share_recipe(
    command: ShareRecipeCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify recipe exists and check ownership via read model
    let recipe_result = sqlx::query("SELECT user_id FROM recipes WHERE id = ?1")
        .bind(&command.recipe_id)
        .fetch_optional(pool)
        .await?;

    match recipe_result {
        Some(row) => {
            let owner_id: String = row.get("user_id");
            if owner_id != command.user_id {
                return Err(RecipeError::PermissionDenied);
            }
        }
        None => {
            return Err(RecipeError::NotFound);
        }
    }

    let toggled_at = Utc::now();

    // Create RecipeShared event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeShared {
            user_id: command.user_id.clone(),
            shared: command.shared,
            toggled_at: toggled_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    tracing::info!(
        recipe_id = %command.recipe_id,
        shared = command.shared,
        "Recipe share status toggled"
    );

    Ok(())
}
