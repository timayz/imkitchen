use chrono::Utc;
use evento::Sqlite;
use sqlx::{Row, SqlitePool};
use validator::Validate;

use crate::aggregate::RecipeAggregate;
use crate::error::{RecipeError, RecipeResult};
use crate::events::{Ingredient, InstructionStep, RecipeCreated, RecipeDeleted};
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
    // Query users table to get tier and recipe_count
    let user_result = sqlx::query("SELECT tier, recipe_count FROM users WHERE id = ?1")
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    match user_result {
        Some(user_row) => {
            let tier: String = user_row.get("tier");
            let recipe_count: i32 = user_row.get("recipe_count");

            // Premium users bypass all limits
            if tier != "premium" {
                // Free tier users limited to 10 recipes
                if recipe_count >= 10 {
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

    // Return the generated aggregator_id as the recipe_id
    Ok(aggregator_id)
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
