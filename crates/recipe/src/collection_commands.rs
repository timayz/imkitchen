use chrono::Utc;
use evento::Sqlite;
use sqlx::{Row, SqlitePool};
use validator::Validate;

use crate::collection_aggregate::CollectionAggregate;
use crate::collection_events::{
    CollectionCreated, CollectionDeleted, CollectionUpdated, RecipeAddedToCollection,
    RecipeRemovedFromCollection,
};
use crate::error::{RecipeError, RecipeResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCollectionCommand {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Collection name must be between 3 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<String>,
}

/// Create a new collection using evento event sourcing pattern
///
/// 1. Validates command fields (name length: 3-100 chars)
/// 2. Creates and commits CollectionCreated event to evento event store
/// 3. Event automatically projected to read model via async subscription handler
/// 4. Returns collection ID (evento aggregator_id)
///
/// All users can create unlimited collections.
pub async fn create_collection(
    command: CreateCollectionCommand,
    user_id: &str,
    executor: &Sqlite,
) -> RecipeResult<String> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    let created_at = Utc::now();

    // Create CollectionCreated event and commit to evento event store
    // The async subscription handler will project to read model
    // evento::create() generates a ULID for the aggregator_id (collection_id)
    let aggregator_id = evento::create::<CollectionAggregate>()
        .data(&CollectionCreated {
            user_id: user_id.to_string(),
            name: command.name,
            description: command.description,
            created_at: created_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Return the generated aggregator_id as the collection_id
    Ok(aggregator_id)
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCollectionCommand {
    pub collection_id: String,
    pub user_id: String, // For ownership verification

    #[validate(length(
        min = 3,
        max = 100,
        message = "Collection name must be between 3 and 100 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(max = 500, message = "Description must be at most 500 characters"))]
    pub description: Option<Option<String>>,
}

/// Update an existing collection using evento event sourcing pattern
///
/// 1. Validates command fields
/// 2. Verifies collection ownership via read model
/// 3. Creates and commits CollectionUpdated event with delta (changed fields only)
/// 4. Event automatically projected to read model via async subscription handler
/// 5. Returns () on success
///
/// Permission check: Only the collection owner can update their collection.
pub async fn update_collection(
    command: UpdateCollectionCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    // Verify collection exists and check ownership via read model
    let collection_result =
        sqlx::query("SELECT user_id FROM recipe_collections WHERE id = ?1 AND deleted_at IS NULL")
            .bind(&command.collection_id)
            .fetch_optional(pool)
            .await?;

    match collection_result {
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

    // Create CollectionUpdated event with only changed fields (delta pattern)
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<CollectionAggregate>(command.collection_id.clone())
        .data(&CollectionUpdated {
            name: command.name,
            description: command.description,
            updated_at: updated_at.to_rfc3339(),
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
pub struct DeleteCollectionCommand {
    pub collection_id: String,
    pub user_id: String, // For ownership verification
}

/// Delete a collection using evento event sourcing pattern
///
/// 1. Verifies collection ownership by checking read model
/// 2. Creates and commits CollectionDeleted event to evento event store
/// 3. Event automatically projected to read model via async subscription handler (soft delete)
/// 4. All recipe assignments are removed, but recipes themselves are preserved
///
/// Permission check: Only the collection owner can delete their collection.
pub async fn delete_collection(
    command: DeleteCollectionCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify collection exists and check ownership via read model
    let collection_result =
        sqlx::query("SELECT user_id FROM recipe_collections WHERE id = ?1 AND deleted_at IS NULL")
            .bind(&command.collection_id)
            .fetch_optional(pool)
            .await?;

    match collection_result {
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

    // Create CollectionDeleted event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<CollectionAggregate>(command.collection_id.clone())
        .data(&CollectionDeleted {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddRecipeToCollectionCommand {
    pub collection_id: String,
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
}

/// Add a recipe to a collection using evento event sourcing pattern
///
/// 1. Verifies collection and recipe ownership via read model
/// 2. Checks if recipe is already in collection (idempotent operation)
/// 3. Creates and commits RecipeAddedToCollection event to evento event store
/// 4. Event automatically projected to read model via async subscription handler
///
/// Permission check: User must own both the collection and the recipe.
/// Idempotent: Adding the same recipe twice has no effect.
pub async fn add_recipe_to_collection(
    command: AddRecipeToCollectionCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify collection exists and check ownership
    let collection_result =
        sqlx::query("SELECT user_id FROM recipe_collections WHERE id = ?1 AND deleted_at IS NULL")
            .bind(&command.collection_id)
            .fetch_optional(pool)
            .await?;

    match collection_result {
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

    // Verify recipe exists and check ownership
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

    // Check if recipe is already in collection (idempotent operation)
    let existing = sqlx::query(
        "SELECT 1 FROM recipe_collection_assignments WHERE collection_id = ?1 AND recipe_id = ?2",
    )
    .bind(&command.collection_id)
    .bind(&command.recipe_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        // Already in collection, return success (idempotent)
        return Ok(());
    }

    let assigned_at = Utc::now();

    // Create RecipeAddedToCollection event and commit to evento event store
    evento::save::<CollectionAggregate>(command.collection_id.clone())
        .data(&RecipeAddedToCollection {
            recipe_id: command.recipe_id,
            assigned_at: assigned_at.to_rfc3339(),
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
pub struct RemoveRecipeFromCollectionCommand {
    pub collection_id: String,
    pub recipe_id: String,
    pub user_id: String, // For ownership verification
}

/// Remove a recipe from a collection using evento event sourcing pattern
///
/// 1. Verifies collection ownership via read model
/// 2. Checks if recipe is in collection
/// 3. Creates and commits RecipeRemovedFromCollection event to evento event store
/// 4. Event automatically projected to read model via async subscription handler
/// 5. Recipe itself is not deleted, only the assignment is removed
///
/// Permission check: User must own the collection.
/// Idempotent: Removing a recipe that's not in the collection has no effect.
pub async fn remove_recipe_from_collection(
    command: RemoveRecipeFromCollectionCommand,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Verify collection exists and check ownership
    let collection_result =
        sqlx::query("SELECT user_id FROM recipe_collections WHERE id = ?1 AND deleted_at IS NULL")
            .bind(&command.collection_id)
            .fetch_optional(pool)
            .await?;

    match collection_result {
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

    // Check if recipe is in collection
    let existing = sqlx::query(
        "SELECT 1 FROM recipe_collection_assignments WHERE collection_id = ?1 AND recipe_id = ?2",
    )
    .bind(&command.collection_id)
    .bind(&command.recipe_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_none() {
        // Not in collection, return success (idempotent)
        return Ok(());
    }

    let removed_at = Utc::now();

    // Create RecipeRemovedFromCollection event and commit to evento event store
    evento::save::<CollectionAggregate>(command.collection_id.clone())
        .data(&RecipeRemovedFromCollection {
            recipe_id: command.recipe_id,
            removed_at: removed_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    Ok(())
}
