use chrono::Utc;
use evento::Sqlite;
use sqlx::SqlitePool;
use tracing;
use validator::Validate;

use crate::aggregate::RecipeAggregate;
use crate::error::{RecipeError, RecipeResult};
use crate::events::{
    Ingredient, InstructionStep, RatingDeleted, RatingUpdated, RecipeCopied, RecipeCreated,
    RecipeDeleted, RecipeFavorited, RecipeRated, RecipeShared, RecipeTagged, RecipeUpdated,
};
use crate::tagging::{CuisineInferenceService, RecipeComplexityCalculator};
use crate::types::{AccompanimentCategory, Cuisine, DietaryTag};
use serde::{Deserialize, Serialize};

// Import UserAggregate for recipe limit checks
use user::aggregate::UserAggregate;

/// Helper function to parse accompaniment category from form string
fn parse_accompaniment_category(s: &str) -> Option<AccompanimentCategory> {
    match s {
        "Pasta" => Some(AccompanimentCategory::Pasta),
        "Rice" => Some(AccompanimentCategory::Rice),
        "Fries" => Some(AccompanimentCategory::Fries),
        "Salad" => Some(AccompanimentCategory::Salad),
        "Bread" => Some(AccompanimentCategory::Bread),
        "Vegetable" => Some(AccompanimentCategory::Vegetable),
        "Other" => Some(AccompanimentCategory::Other),
        _ => {
            tracing::warn!("Failed to parse accompaniment category: '{}'", s);
            None
        }
    }
}

/// Helper function to parse cuisine from form string
fn parse_cuisine(s: &str) -> Option<Cuisine> {
    match s {
        "Italian" => Some(Cuisine::Italian),
        "Indian" => Some(Cuisine::Indian),
        "Mexican" => Some(Cuisine::Mexican),
        "Chinese" => Some(Cuisine::Chinese),
        "Japanese" => Some(Cuisine::Japanese),
        "French" => Some(Cuisine::French),
        "American" => Some(Cuisine::American),
        "Mediterranean" => Some(Cuisine::Mediterranean),
        "Thai" => Some(Cuisine::Thai),
        "Korean" => Some(Cuisine::Korean),
        "Vietnamese" => Some(Cuisine::Vietnamese),
        "Greek" => Some(Cuisine::Greek),
        "Spanish" => Some(Cuisine::Spanish),
        custom if !custom.is_empty() => Some(Cuisine::Custom(custom.to_string())),
        _ => {
            tracing::warn!("Failed to parse cuisine: empty string provided");
            None
        }
    }
}

/// Helper function to parse dietary tag from form string
fn parse_dietary_tag(s: &str) -> Option<DietaryTag> {
    match s {
        "Vegetarian" => Some(DietaryTag::Vegetarian),
        "Vegan" => Some(DietaryTag::Vegan),
        "Gluten-Free" => Some(DietaryTag::GlutenFree),
        "Dairy-Free" => Some(DietaryTag::DairyFree),
        "Nut-Free" => Some(DietaryTag::NutFree),
        "Halal" => Some(DietaryTag::Halal),
        "Kosher" => Some(DietaryTag::Kosher),
        _ => {
            tracing::warn!("Failed to parse dietary tag: '{}'", s);
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateRecipeCommand {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Title must be between 3 and 200 characters"
    ))]
    pub title: String,

    #[validate(custom(function = "validate_recipe_type"))]
    pub recipe_type: String, // AC 9.4.2: Must be "appetizer", "main_course", "dessert", or "accompaniment"

    #[validate(length(min = 1, message = "At least 1 ingredient is required"))]
    pub ingredients: Vec<Ingredient>,

    #[validate(length(min = 1, message = "At least 1 instruction step is required"))]
    pub instructions: Vec<InstructionStep>,

    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,

    // AC 9.4.3: Main course accepts accompaniment
    pub accepts_accompaniment: bool,
    // AC 9.4.4: Preferred accompaniment categories
    pub preferred_accompaniments: Vec<String>,
    // AC 9.4.5: Accompaniment category (required if recipe_type = "accompaniment")
    pub accompaniment_category: Option<String>,
    // AC 9.4.6: Cuisine selection (custom cuisines stored as Cuisine::Custom variant)
    pub cuisine: Option<String>,
    // AC 9.4.7: Dietary tags
    pub dietary_tags: Vec<String>,
}

/// AC 9.4.2: Validate recipe_type field
/// Only accepts: "appetizer", "main_course", "dessert", or "accompaniment"
fn validate_recipe_type(recipe_type: &str) -> Result<(), validator::ValidationError> {
    match recipe_type {
        "appetizer" | "main_course" | "dessert" | "accompaniment" => Ok(()),
        _ => {
            let mut error = validator::ValidationError::new("invalid_recipe_type");
            error.message = Some(std::borrow::Cow::from(
                "Recipe type must be 'appetizer', 'main_course', 'dessert', or 'accompaniment'",
            ));
            Err(error)
        }
    }
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
    _pool: &SqlitePool,
    bypass_premium: bool,
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

    // AC 9.4.9: Validate that accompaniment category is provided if recipe_type is "accompaniment"
    if command.recipe_type == "accompaniment" && command.accompaniment_category.is_none() {
        return Err(RecipeError::ValidationError(
            "Accompaniment category is required when recipe type is 'accompaniment'".to_string(),
        ));
    }

    // Check user tier and recipe count for freemium enforcement using evento::load
    // AC-11: recipe_count is tracked in UserAggregate via RecipeCreated/RecipeDeleted events
    let user_load_result = evento::load::<UserAggregate, _>(executor, user_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if user exists
    if user_load_result.item.user_id.is_empty() {
        return Err(RecipeError::ValidationError("User not found".to_string()));
    }

    // Premium users bypass all limits (or MVP/demo mode bypass)
    if !bypass_premium && user_load_result.item.tier != "premium" {
        // Free tier users limited to 10 recipes
        // recipe_count is tracked in UserAggregate and represents all non-deleted recipes
        if user_load_result.item.recipe_count >= 10 {
            return Err(RecipeError::RecipeLimitReached);
        }
    }

    let created_at = Utc::now();

    // Create RecipeCreated event and commit to evento event store
    // The async subscription handler will project to read model
    // evento::create() generates a ULID for the aggregator_id (recipe_id)
    // AC 9.4.2: Include recipe_type in event (now supports "accompaniment")

    // Parse cuisine and dietary tags from command
    let parsed_cuisine = command.cuisine.and_then(|s| parse_cuisine(&s));

    let parsed_dietary_tags: Vec<DietaryTag> = command
        .dietary_tags
        .iter()
        .filter_map(|s| parse_dietary_tag(s))
        .collect();

    let aggregator_id = evento::create::<RecipeAggregate>()
        .data(&RecipeCreated {
            user_id: user_id.to_string(),
            title: command.title,
            recipe_type: command.recipe_type, // AC 9.4.2: Course type (appetizer, main_course, dessert, accompaniment)
            ingredients: command.ingredients,
            instructions: command.instructions,
            prep_time_min: command.prep_time_min,
            cook_time_min: command.cook_time_min,
            advance_prep_hours: command.advance_prep_hours,
            serving_size: command.serving_size,
            created_at: created_at.to_rfc3339(),
            // AC 9.4.3: Main course accepts accompaniment
            accepts_accompaniment: Some(command.accepts_accompaniment),
            // AC 9.4.4: Preferred accompaniment categories (convert strings to enum)
            preferred_accompaniments: Some(
                command
                    .preferred_accompaniments
                    .iter()
                    .filter_map(|s| parse_accompaniment_category(s))
                    .collect(),
            ),
            // AC 9.4.5: Accompaniment category
            accompaniment_category: command
                .accompaniment_category
                .and_then(|s| parse_accompaniment_category(&s)),
            // AC 9.4.6: Cuisine (convert string to enum) - use pre-parsed value
            cuisine: parsed_cuisine,
            // AC 9.4.7: Dietary tags (convert strings to enum) - use pre-parsed value
            dietary_tags: Some(parsed_dietary_tags),
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

    // Emit user::events::RecipeCreated to update UserAggregate.recipe_count
    evento::save::<user::aggregate::UserAggregate>(user_id.to_string())
        .data(&user::events::RecipeCreated {
            user_id: user_id.to_string(),
            title: load_result.item.title.clone(),
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

    // If user provided explicit cuisine/dietary_tags, skip the RecipeTagged event
    // to avoid overwriting user input with auto-inferred values
    let has_user_provided_metadata =
        aggregate.tags.cuisine.is_some() || !aggregate.tags.dietary_tags.is_empty();
    if has_user_provided_metadata {
        // User provided metadata explicitly - don't emit RecipeTagged to avoid overwriting
        return Ok(());
    }

    // Auto-infer cuisine from ingredients
    let cuisine = CuisineInferenceService::infer(&aggregate.ingredients);
    // Dietary tags are no longer auto-detected - must be set manually
    let dietary_tags = Vec::new();

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
/// 1. Loads recipe aggregate from event store to verify ownership
/// 2. Creates and commits RecipeDeleted event to evento event store
/// 3. Event automatically projected to read model via async subscription handler (soft delete)
/// 4. User domain also listens to decrement recipe_count
///
/// Permission check: Only the recipe owner can delete their recipe.
/// Ownership is verified by loading the aggregate from the event store (consistent data).
pub async fn delete_recipe(
    command: DeleteRecipeCommand,
    executor: &Sqlite,
    _pool: &SqlitePool,
) -> RecipeResult<()> {
    // Load recipe aggregate from event store to verify ownership
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists (aggregate has data)
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::NotFound);
    }

    // Check ownership
    if load_result.item.user_id != command.user_id {
        return Err(RecipeError::PermissionDenied);
    }

    let deleted_at = Utc::now();

    // Create RecipeDeleted event and commit to evento event store
    // evento::save() automatically loads the aggregate before appending the event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeDeleted {
            user_id: command.user_id.clone(),
            deleted_at: deleted_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Emit user::events::RecipeDeleted to update UserAggregate.recipe_count
    evento::save::<user::aggregate::UserAggregate>(command.user_id.clone())
        .data(&user::events::RecipeDeleted {
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

    #[validate(custom(function = "validate_optional_recipe_type"))]
    pub recipe_type: Option<String>, // AC-3: Allow updating recipe type

    #[validate(length(min = 1, message = "At least 1 ingredient is required"))]
    pub ingredients: Option<Vec<Ingredient>>,

    #[validate(length(min = 1, message = "At least 1 instruction step is required"))]
    pub instructions: Option<Vec<InstructionStep>>,

    pub prep_time_min: Option<Option<u32>>,
    pub cook_time_min: Option<Option<u32>>,
    pub advance_prep_hours: Option<Option<u32>>,
    pub serving_size: Option<Option<u32>>,

    // Metadata fields (AC 9.4.3-9.4.7)
    pub accepts_accompaniment: Option<bool>,
    pub preferred_accompaniments: Option<Vec<AccompanimentCategory>>,
    pub accompaniment_category: Option<Option<AccompanimentCategory>>,
    pub cuisine: Option<Option<Cuisine>>,
    pub dietary_tags: Option<Vec<DietaryTag>>,
}

/// AC-3: Validate optional recipe_type field in updates
/// Note: validator automatically unwraps Option fields, so this receives &String
fn validate_optional_recipe_type(recipe_type: &str) -> Result<(), validator::ValidationError> {
    validate_recipe_type(recipe_type)
}

/// Update an existing recipe using evento event sourcing pattern
///
/// 1. Validates command fields
/// 2. Loads recipe aggregate from event store to verify ownership
/// 3. Creates and commits RecipeUpdated event with delta (changed fields only)
/// 4. Event automatically projected to read model via async subscription handler
/// 5. Returns () on success
///
/// Permission check: Only the recipe owner can update their recipe.
/// Ownership is verified by loading the aggregate from the event store (consistent data).
pub async fn update_recipe(
    command: UpdateRecipeCommand,
    executor: &Sqlite,
    _pool: &SqlitePool,
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

    // Load recipe aggregate from event store to verify ownership
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::NotFound);
    }

    // Check ownership
    if load_result.item.user_id != command.user_id {
        return Err(RecipeError::PermissionDenied);
    }

    let updated_at = Utc::now();

    // Create RecipeUpdated event with only changed fields (delta pattern)
    // evento::save() automatically loads the aggregate before appending the event
    // AC-3: Include recipe_type in update event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeUpdated {
            title: command.title,
            recipe_type: command.recipe_type, // AC-3: Allow updating recipe type
            ingredients: command.ingredients,
            instructions: command.instructions,
            prep_time_min: command.prep_time_min,
            cook_time_min: command.cook_time_min,
            advance_prep_hours: command.advance_prep_hours,
            serving_size: command.serving_size,
            // Metadata fields (AC 9.4.3-9.4.7)
            accepts_accompaniment: command.accepts_accompaniment,
            preferred_accompaniments: command.preferred_accompaniments,
            accompaniment_category: command.accompaniment_category,
            cuisine: command.cuisine,
            dietary_tags: command.dietary_tags,
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
    _pool: &SqlitePool,
) -> RecipeResult<()> {
    // Load recipe aggregate from event store to verify ownership
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::NotFound);
    }

    // Check ownership
    if load_result.item.user_id != command.user_id {
        return Err(RecipeError::PermissionDenied);
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
/// 1. Loads recipe aggregate from event store to verify ownership and get current is_favorite status
/// 2. Creates and commits RecipeFavorited event with toggled status
/// 3. Event automatically projected to read model via async subscription handler
/// 4. Returns the new favorite status (true/false)
///
/// Permission check: Only the recipe owner can favorite/unfavorite their recipe.
/// Ownership is verified by loading the aggregate from the event store (consistent data).
#[tracing::instrument(skip(executor, _pool), fields(recipe_id = %command.recipe_id, user_id = %command.user_id))]
pub async fn favorite_recipe(
    command: FavoriteRecipeCommand,
    executor: &Sqlite,
    _pool: &SqlitePool,
) -> RecipeResult<bool> {
    // Load recipe aggregate to verify ownership and get current is_favorite status
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::NotFound);
    }

    // Check ownership
    if load_result.item.user_id != command.user_id {
        return Err(RecipeError::PermissionDenied);
    }

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
/// 1. Loads recipe aggregate from event store to verify ownership
/// 2. Creates and commits RecipeShared event with shared boolean parameter
/// 3. Event automatically projected to read model via async subscription handler
/// 4. Returns () on success
///
/// Permission check: Only the recipe owner can share/unshare their recipe.
/// Ownership is verified by loading the aggregate from the event store (consistent data).
///
/// This command handles both sharing (shared=true) and unsharing (shared=false).
/// AC-2: Toggle changes privacy from "private" to "shared" (RecipeShared event)
/// AC-6: Owner can revert to private at any time (removes from community discovery)
#[tracing::instrument(skip(executor, _pool), fields(recipe_id = %command.recipe_id, user_id = %command.user_id, shared = %command.shared))]
pub async fn share_recipe(
    command: ShareRecipeCommand,
    executor: &Sqlite,
    _pool: &SqlitePool,
) -> RecipeResult<()> {
    // Load recipe aggregate from event store to verify ownership
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::NotFound);
    }

    // Check ownership
    if load_result.item.user_id != command.user_id {
        return Err(RecipeError::PermissionDenied);
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

    // Emit user::events::RecipeShared to update UserAggregate.recipe_count
    // Shared recipes do NOT count toward the freemium limit
    evento::save::<user::aggregate::UserAggregate>(command.user_id.clone())
        .data(&user::events::RecipeShared {
            user_id: command.user_id.clone(),
            shared: command.shared,
            shared_at: toggled_at.to_rfc3339(),
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

/// Command to rate a recipe (create or update rating)
///
/// AC-1, AC-2, AC-3, AC-10, AC-11
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateRecipeCommand {
    pub recipe_id: String, // Recipe being rated

    #[validate(range(min = 1, max = 5, message = "Stars must be between 1 and 5"))]
    pub stars: i32, // Rating value (1-5)

    #[validate(length(max = 500, message = "Review text must not exceed 500 characters"))]
    pub review_text: Option<String>, // Optional review text
}

/// Rate a recipe using evento event sourcing pattern
///
/// AC-1, AC-2, AC-3, AC-10, AC-11, AC-12:
/// 1. Validates command fields (stars 1-5, review_text <= 500 chars)
/// 2. Verifies recipe exists and is shared (public recipes only)
/// 3. Emits RecipeRated event (evento handles UPSERT via projection)
/// 4. Event automatically projected to ratings read model via async subscription handler
/// 5. AC-2, AC-12: Duplicate detection handled in projection layer (UNIQUE constraint + UPSERT)
///
/// Authentication: User must be authenticated (enforced by route middleware)
pub async fn rate_recipe(
    command: RateRecipeCommand,
    user_id: &str,
    executor: &Sqlite,
    _pool: &SqlitePool,
) -> RecipeResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    // AC-10: Load recipe aggregate to verify it exists and is shared (ratings only on shared/community recipes)
    let load_result = evento::load::<RecipeAggregate, _>(executor, &command.recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::ValidationError("Recipe not found".to_string()));
    }

    // Check if recipe is deleted
    if load_result.item.is_deleted {
        return Err(RecipeError::ValidationError("Recipe not found".to_string()));
    }

    // Check if recipe is shared (only shared recipes can be rated)
    if !load_result.item.is_shared {
        return Err(RecipeError::ValidationError(
            "Only shared recipes can be rated".to_string(),
        ));
    }

    let rated_at = Utc::now();

    // Emit RecipeRated event
    // AC-2, AC-12: Projection handler will UPSERT into ratings table (INSERT or UPDATE existing)
    // The aggregator_id is the recipe_id (the recipe being rated, not a rating aggregate)
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RecipeRated {
            user_id: user_id.to_string(),
            stars: command.stars,
            review_text: command.review_text.clone(),
            rated_at: rated_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    tracing::info!(
        recipe_id = %command.recipe_id,
        user_id = %user_id,
        stars = command.stars,
        "Recipe rated"
    );

    Ok(())
}

/// Command to update an existing rating
///
/// AC-6: User can edit their own review
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateRatingCommand {
    pub recipe_id: String, // Recipe being rated

    #[validate(range(min = 1, max = 5, message = "Stars must be between 1 and 5"))]
    pub stars: i32, // New rating value (1-5)

    #[validate(length(max = 500, message = "Review text must not exceed 500 characters"))]
    pub review_text: Option<String>, // New review text
}

/// Update an existing rating using evento event sourcing pattern
///
/// AC-6: User can edit their own review
/// 1. Validates command fields
/// 2. Verifies rating exists and belongs to the user (ownership check)
/// 3. Emits RatingUpdated event
/// 4. Event automatically projected to ratings read model via async subscription handler
///
/// Returns 403 Forbidden if user attempts to edit another user's rating (enforced in route layer)
pub async fn update_rating(
    command: UpdateRatingCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // Validate command
    command
        .validate()
        .map_err(|e| RecipeError::ValidationError(e.to_string()))?;

    // AC-6: Verify rating exists and belongs to the user
    let rating_result =
        sqlx::query("SELECT user_id FROM ratings WHERE recipe_id = ?1 AND user_id = ?2")
            .bind(&command.recipe_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if rating_result.is_none() {
        return Err(RecipeError::ValidationError(
            "Rating not found or access denied".to_string(),
        ));
    }

    let updated_at = Utc::now();

    // Emit RatingUpdated event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RatingUpdated {
            user_id: user_id.to_string(),
            stars: command.stars,
            review_text: command.review_text.clone(),
            updated_at: updated_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    tracing::info!(
        recipe_id = %command.recipe_id,
        user_id = %user_id,
        "Rating updated"
    );

    Ok(())
}

/// Command to delete a rating
///
/// AC-7: User can delete their own review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRatingCommand {
    pub recipe_id: String, // Recipe being rated
}

/// Delete a rating using evento event sourcing pattern
///
/// AC-7: User can delete their own review
/// 1. Verifies rating exists and belongs to the user (ownership check)
/// 2. Emits RatingDeleted event
/// 3. Event automatically projected to ratings read model via async subscription handler (DELETE)
///
/// Returns 403 Forbidden if user attempts to delete another user's rating (enforced in route layer)
pub async fn delete_rating(
    command: DeleteRatingCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
) -> RecipeResult<()> {
    // AC-7: Verify rating exists and belongs to the user
    let rating_result =
        sqlx::query("SELECT user_id FROM ratings WHERE recipe_id = ?1 AND user_id = ?2")
            .bind(&command.recipe_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    if rating_result.is_none() {
        return Err(RecipeError::ValidationError(
            "Rating not found or access denied".to_string(),
        ));
    }

    let deleted_at = Utc::now();

    // Emit RatingDeleted event
    evento::save::<RecipeAggregate>(command.recipe_id.clone())
        .data(&RatingDeleted {
            user_id: user_id.to_string(),
            deleted_at: deleted_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    tracing::info!(
        recipe_id = %command.recipe_id,
        user_id = %user_id,
        "Rating deleted"
    );

    Ok(())
}

/// Command to copy a community recipe to user's personal library
///
/// AC-2, AC-3, AC-4, AC-5, AC-6, AC-7, AC-10, AC-11
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyRecipeCommand {
    pub original_recipe_id: String, // ID of the original community recipe to copy
}

/// BatchImportRecipe represents a single recipe in a batch import
///
/// Identical structure to CreateRecipeCommand but used for JSON deserialization
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BatchImportRecipe {
    #[validate(length(
        min = 3,
        max = 200,
        message = "Title must be between 3 and 200 characters"
    ))]
    pub title: String,

    #[validate(custom(function = "validate_recipe_type"))]
    pub recipe_type: String,

    #[validate(length(min = 1, message = "At least 1 ingredient is required"))]
    pub ingredients: Vec<Ingredient>,

    #[validate(length(min = 1, message = "At least 1 instruction step is required"))]
    pub instructions: Vec<InstructionStep>,

    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>,
    pub serving_size: Option<u32>,
}

/// BatchImportRecipesCommand represents a batch import operation
///
/// AC-5: Contains array of recipes (not single recipe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchImportRecipesCommand {
    pub recipes: Vec<BatchImportRecipe>,
}

/// BatchImportResult contains the outcome of a batch import operation
///
/// AC-9, AC-10: Tracks successful imports and failures with error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchImportResult {
    pub successful_recipe_ids: Vec<String>, // Recipe IDs that were successfully imported
    pub failed_imports: Vec<(usize, String)>, // (index, error_message) for failed recipes
    pub total_attempted: usize,             // Total number of recipes in the batch
}

/// Copy a community recipe to user's personal library using evento event sourcing pattern
///
/// AC-2: Copies recipe to user's personal library with full recipe data duplicated
/// AC-3: Copied recipe becomes owned by user (user_id set to current user)
/// AC-4: Original creator attribution maintained in metadata (original_recipe_id, original_author)
/// AC-5: Copy counts as new recipe toward free tier limit (10 recipe maximum)
/// AC-6: Copied recipe defaults to private (is_shared = false)
/// AC-7: Modifications to copy don't affect original (independent Recipe aggregate created)
/// AC-10: Prevents duplicate copies (check if user already copied this recipe)
/// AC-11: Enforces free tier limit (returns RecipeLimitReached error)
///
/// Flow:
/// 1. Verifies original recipe exists and is shared (only community recipes can be copied)
/// 2. Checks if user already copied this recipe (prevent duplicates)
/// 3. Checks user tier and recipe count (free tier limited to 10 recipes)
/// 4. Loads original recipe aggregate from event stream to get full data
/// 5. Creates new Recipe aggregate with RecipeCreated event (full data duplication)
/// 6. Emits RecipeCopied event to store attribution metadata
/// 7. Returns new recipe ID
pub async fn copy_recipe(
    command: CopyRecipeCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
    bypass_premium: bool,
) -> RecipeResult<String> {
    // AC-10: Load original recipe aggregate to verify it exists and is shared
    let original_load_result =
        evento::load::<RecipeAggregate, _>(executor, &command.original_recipe_id)
            .await
            .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if recipe exists
    if original_load_result.item.recipe_id.is_empty() {
        return Err(RecipeError::ValidationError("Recipe not found".to_string()));
    }

    // Check if recipe is deleted
    if original_load_result.item.is_deleted {
        return Err(RecipeError::ValidationError("Recipe not found".to_string()));
    }

    // Check if recipe is shared (only shared recipes can be copied)
    if !original_load_result.item.is_shared {
        return Err(RecipeError::ValidationError(
            "Only shared recipes can be copied".to_string(),
        ));
    }

    let original_author = original_load_result.item.user_id.clone();

    // AC-10: Check if user already copied this recipe (prevent duplicates)
    let duplicate_check: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM recipes WHERE user_id = ?1 AND original_recipe_id = ?2 AND deleted_at IS NULL"
    )
    .bind(user_id)
    .bind(&command.original_recipe_id)
    .fetch_optional(pool)
    .await?;

    if let Some(count) = duplicate_check {
        if count > 0 {
            return Err(RecipeError::AlreadyCopied);
        }
    }

    // AC-5, AC-11: Load user aggregate to check tier and recipe count for freemium enforcement
    let user_load_result = evento::load::<UserAggregate, _>(executor, user_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if user exists
    if user_load_result.item.user_id.is_empty() {
        return Err(RecipeError::ValidationError("User not found".to_string()));
    }

    // Premium users bypass all limits (or MVP/demo mode bypass)
    if !bypass_premium && user_load_result.item.tier != "premium" {
        // Free tier users limited to 10 private recipes
        // recipe_count is tracked in UserAggregate via RecipeCreated/RecipeDeleted events
        if user_load_result.item.recipe_count >= 10 {
            return Err(RecipeError::RecipeLimitReached);
        }
    }

    // AC-2, AC-7: Load original recipe aggregate to get full recipe data
    let original_aggregate =
        evento::load::<RecipeAggregate, _>(executor, &command.original_recipe_id)
            .await
            .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    let copied_at = Utc::now();

    // AC-2, AC-3, AC-6, AC-7: Create new Recipe aggregate with RecipeCreated event
    // AC-4: Also emit RecipeCopied event in same transaction for attribution metadata
    // Copy all data from original recipe, but set user_id to copying user
    // Default to private (is_shared = false) per AC-6
    // AC-2: Include recipe_type from original recipe
    let new_recipe_id = evento::create::<RecipeAggregate>()
        .data(&RecipeCreated {
            user_id: user_id.to_string(),
            title: original_aggregate.item.title.clone(),
            recipe_type: original_aggregate.item.recipe_type.clone(), // AC-2: Copy recipe type
            ingredients: original_aggregate.item.ingredients.clone(),
            instructions: original_aggregate.item.instructions.clone(),
            prep_time_min: original_aggregate.item.prep_time_min,
            cook_time_min: original_aggregate.item.cook_time_min,
            advance_prep_hours: original_aggregate.item.advance_prep_hours,
            serving_size: original_aggregate.item.serving_size,
            created_at: copied_at.to_rfc3339(),
            // Epic 6 fields - copy from original recipe
            accepts_accompaniment: Some(original_aggregate.item.accepts_accompaniment),
            preferred_accompaniments: Some(
                original_aggregate.item.preferred_accompaniments.clone(),
            ),
            accompaniment_category: original_aggregate.item.accompaniment_category,
            cuisine: None, // Cuisine not copied yet (will be in tagging domain)
            dietary_tags: Some(vec![]), // Dietary tags not copied yet
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .data(&RecipeCopied {
            original_recipe_id: command.original_recipe_id.clone(),
            original_author: original_author.clone(),
            copying_user_id: user_id.to_string(),
            copied_at: copied_at.to_rfc3339(),
        })
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .metadata(&true)
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?
        .commit(executor)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Calculate and emit RecipeTagged event for automatic tagging
    // Load the new aggregate to access the recipe data
    let load_result = evento::load::<RecipeAggregate, _>(executor, &new_recipe_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    emit_recipe_tagged_event(&new_recipe_id, &load_result.item, executor, false).await?;

    tracing::info!(
        original_recipe_id = %command.original_recipe_id,
        new_recipe_id = %new_recipe_id,
        user_id = %user_id,
        "Recipe copied to user library"
    );

    // Return the new recipe ID
    Ok(new_recipe_id)
}

/// Batch import multiple recipes using evento event sourcing pattern
///
/// AC-5, AC-6, AC-7, AC-8, AC-9, AC-12:
/// 1. Validates array is non-empty
/// 2. Checks free tier limit upfront (current_count + import_count <= 10)
/// 3. Loops through recipes, validates each, calls create_recipe()
/// 4. Collects successful recipe IDs and failures with error messages
/// 5. Returns BatchImportResult with counts and details
/// 6. Partial success: some recipes can succeed while others fail (no rollback)
///
/// Free tier enforcement: Checks total count upfront to prevent partial success with limit exceeded.
/// If the batch would exceed the limit, the entire batch is rejected with RecipeLimitReached error.
pub async fn batch_import_recipes(
    command: BatchImportRecipesCommand,
    user_id: &str,
    executor: &Sqlite,
    pool: &SqlitePool,
    bypass_premium: bool,
) -> RecipeResult<BatchImportResult> {
    // AC-5: Validate array is non-empty
    if command.recipes.is_empty() {
        return Err(RecipeError::ValidationError(
            "No recipes found in file".to_string(),
        ));
    }

    // AC-8: Check free tier limit upfront
    // Load user aggregate to check tier and recipe count
    let user_load_result = evento::load::<UserAggregate, _>(executor, user_id)
        .await
        .map_err(|e| RecipeError::EventStoreError(e.to_string()))?;

    // Check if user exists
    if user_load_result.item.user_id.is_empty() {
        return Err(RecipeError::ValidationError("User not found".to_string()));
    }

    // Premium users bypass all limits (or MVP/demo mode bypass)
    if !bypass_premium && user_load_result.item.tier != "premium" {
        // Free tier users limited to 10 recipes
        // Check if (current_count + import_count) > 10
        let current_count = user_load_result.item.recipe_count;
        let import_count = command.recipes.len() as i32;

        if current_count + import_count > 10 {
            return Err(RecipeError::RecipeLimitReached);
        }
    }

    let total_attempted = command.recipes.len();
    let mut successful_recipe_ids = Vec::new();
    let mut failed_imports: Vec<(usize, String)> = Vec::new();

    // AC-6, AC-9, AC-12: Loop through recipes and import each
    for (index, recipe) in command.recipes.into_iter().enumerate() {
        // Convert BatchImportRecipe to CreateRecipeCommand
        let create_cmd = CreateRecipeCommand {
            title: recipe.title,
            recipe_type: recipe.recipe_type,
            ingredients: recipe.ingredients,
            instructions: recipe.instructions,
            prep_time_min: recipe.prep_time_min,
            cook_time_min: recipe.cook_time_min,
            advance_prep_hours: recipe.advance_prep_hours,
            serving_size: recipe.serving_size,
            // AC 9.4.3-9.4.7: Default values for batch imports (can be enhanced later)
            accepts_accompaniment: false,
            preferred_accompaniments: vec![],
            accompaniment_category: None,
            cuisine: None,
            dietary_tags: vec![],
        };

        // Attempt to create recipe using existing create_recipe function
        // Pass bypass_premium flag through (already checked upfront in batch)
        match create_recipe(create_cmd, user_id, executor, pool, true).await {
            Ok(recipe_id) => {
                successful_recipe_ids.push(recipe_id);
            }
            Err(e) => {
                // Collect error with recipe index for user feedback
                failed_imports.push((index, e.to_string()));
            }
        }
    }

    // AC-9: Return result with success/failure counts
    Ok(BatchImportResult {
        successful_recipe_ids,
        failed_imports,
        total_attempted,
    })
}
