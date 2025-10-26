use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    Ingredient, InstructionStep, RatingDeleted, RatingUpdated, RecipeAccompanimentSettingsUpdated,
    RecipeCopied, RecipeCreated, RecipeDeleted, RecipeFavorited, RecipeRated, RecipeShared,
    RecipeTagged, RecipeUpdated,
};
use crate::tagging::{Complexity, RecipeTags};
use crate::types::AccompanimentCategory;

/// Recipe aggregate representing the state of a recipe entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// Fields follow the tech spec requirements including ingredients, instructions, timing, and privacy.
///
/// Note: recipe_id, user_id, created_at stored as String for bincode compatibility
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct RecipeAggregate {
    // Core identity
    pub recipe_id: String,
    pub user_id: String, // Owner of the recipe

    // Recipe details
    pub title: String,
    pub recipe_type: String, // Course type: "appetizer", "main_course", or "dessert"
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<InstructionStep>,

    // Timing information
    pub prep_time_min: Option<u32>,
    pub cook_time_min: Option<u32>,
    pub advance_prep_hours: Option<u32>, // For advance preparation (e.g., marinating)

    // Serving information
    pub serving_size: Option<u32>,

    // Status flags
    pub is_favorite: bool,
    pub is_deleted: bool,
    pub is_shared: bool, // Privacy: false = private, true = shared with community

    // Tags
    pub tags: RecipeTags,

    // Accompaniment fields (Epic 6: Enhanced Meal Planning System)
    /// Whether this main course accepts an accompaniment side dish
    pub accepts_accompaniment: bool,
    /// Preferred accompaniment categories this recipe pairs well with (if accepts_accompaniment)
    pub preferred_accompaniments: Vec<AccompanimentCategory>,
    /// Accompaniment category if this recipe is a side dish (None for main courses)
    pub accompaniment_category: Option<AccompanimentCategory>,

    // Attribution (for copied recipes)
    pub original_recipe_id: Option<String>, // ID of original recipe if this is a copy
    pub original_author: Option<String>,    // User ID of original creator if this is a copy

    // Timestamps
    pub created_at: String, // RFC3339 formatted timestamp
}

/// Implement evento aggregator pattern for RecipeAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl RecipeAggregate {
    /// Handle RecipeCreated event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    ///
    /// AC-9: Backward compatibility - old events without recipe_type default to "main_course"
    async fn recipe_created(
        &mut self,
        event: evento::EventDetails<RecipeCreated>,
    ) -> anyhow::Result<()> {
        self.recipe_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.title = event.data.title;
        // AC-9: Handle old events without recipe_type field (backward compatibility)
        self.recipe_type = if event.data.recipe_type.is_empty() {
            "main_course".to_string() // Default for old events
        } else {
            event.data.recipe_type
        };
        self.ingredients = event.data.ingredients;
        self.instructions = event.data.instructions;
        self.prep_time_min = event.data.prep_time_min;
        self.cook_time_min = event.data.cook_time_min;
        self.advance_prep_hours = event.data.advance_prep_hours;
        self.serving_size = event.data.serving_size;
        self.created_at = event.data.created_at;
        self.is_favorite = false;
        self.is_deleted = false;
        self.is_shared = false; // Default to private per AC-9
        self.tags = RecipeTags::default();
        // Epic 6 accompaniment fields (with backwards compatibility defaults)
        self.accepts_accompaniment = event.data.accepts_accompaniment.unwrap_or(false);
        self.preferred_accompaniments = event
            .data
            .preferred_accompaniments
            .clone()
            .unwrap_or_default();
        self.accompaniment_category = event.data.accompaniment_category;
        self.original_recipe_id = None;
        self.original_author = None;
        Ok(())
    }

    /// Handle RecipeDeleted event to mark recipe as deleted
    ///
    /// This is a soft delete - the recipe remains in the event store for audit trail,
    /// but is marked as deleted and won't be returned in queries.
    async fn recipe_deleted(
        &mut self,
        _event: evento::EventDetails<RecipeDeleted>,
    ) -> anyhow::Result<()> {
        self.is_deleted = true;
        Ok(())
    }

    /// Handle RecipeFavorited event to toggle favorite status
    ///
    /// This event handler updates the aggregate state when a user toggles
    /// the favorite status of a recipe.
    async fn recipe_favorited(
        &mut self,
        event: evento::EventDetails<RecipeFavorited>,
    ) -> anyhow::Result<()> {
        self.is_favorite = event.data.favorited;
        Ok(())
    }

    /// Handle RecipeUpdated event to apply changes to aggregate state
    ///
    /// This event handler updates only the fields that were changed (delta pattern).
    /// Fields that are None in the event are not modified in the aggregate.
    ///
    /// AC-3: Support updating recipe_type
    async fn recipe_updated(
        &mut self,
        event: evento::EventDetails<RecipeUpdated>,
    ) -> anyhow::Result<()> {
        if let Some(title) = event.data.title {
            self.title = title;
        }
        // AC-3: Allow updating recipe_type
        if let Some(recipe_type) = event.data.recipe_type {
            self.recipe_type = recipe_type;
        }
        if let Some(ingredients) = event.data.ingredients {
            self.ingredients = ingredients;
        }
        if let Some(instructions) = event.data.instructions {
            self.instructions = instructions;
        }
        if let Some(prep_time) = event.data.prep_time_min {
            self.prep_time_min = prep_time;
        }
        if let Some(cook_time) = event.data.cook_time_min {
            self.cook_time_min = cook_time;
        }
        if let Some(advance_prep) = event.data.advance_prep_hours {
            self.advance_prep_hours = advance_prep;
        }
        if let Some(serving_size) = event.data.serving_size {
            self.serving_size = serving_size;
        }
        Ok(())
    }

    /// Handle RecipeTagged event to update recipe tags
    ///
    /// This event handler updates the tags field when the recipe is automatically tagged.
    async fn recipe_tagged(
        &mut self,
        event: evento::EventDetails<RecipeTagged>,
    ) -> anyhow::Result<()> {
        // Parse complexity from string to enum
        self.tags.complexity = event
            .data
            .complexity
            .as_ref()
            .and_then(|c| match c.as_str() {
                "simple" => Some(Complexity::Simple),
                "moderate" => Some(Complexity::Moderate),
                "complex" => Some(Complexity::Complex),
                _ => None,
            });
        self.tags.cuisine = event.data.cuisine.clone();
        self.tags.dietary_tags = event.data.dietary_tags.clone();
        self.tags.manual_override = event.data.manual_override;
        Ok(())
    }

    /// Handle RecipeShared event to toggle share status
    ///
    /// This event handler updates the aggregate state when a user toggles
    /// the privacy status of a recipe (shared with community vs. private).
    async fn recipe_shared(
        &mut self,
        event: evento::EventDetails<RecipeShared>,
    ) -> anyhow::Result<()> {
        self.is_shared = event.data.shared;
        Ok(())
    }

    /// Handle RecipeRated event
    ///
    /// Note: Recipe aggregate does not store individual ratings - they are managed
    /// in the ratings read model table. This handler exists for evento framework
    /// but performs no aggregate state changes.
    async fn recipe_rated(
        &mut self,
        _event: evento::EventDetails<RecipeRated>,
    ) -> anyhow::Result<()> {
        // No-op: ratings managed in separate read model, not in recipe aggregate
        Ok(())
    }

    /// Handle RatingUpdated event
    ///
    /// Note: Recipe aggregate does not store individual ratings - they are managed
    /// in the ratings read model table. This handler exists for evento framework
    /// but performs no aggregate state changes.
    async fn rating_updated(
        &mut self,
        _event: evento::EventDetails<RatingUpdated>,
    ) -> anyhow::Result<()> {
        // No-op: ratings managed in separate read model, not in recipe aggregate
        Ok(())
    }

    /// Handle RatingDeleted event
    ///
    /// Note: Recipe aggregate does not store individual ratings - they are managed
    /// in the ratings read model table. This handler exists for evento framework
    /// but performs no aggregate state changes.
    async fn rating_deleted(
        &mut self,
        _event: evento::EventDetails<RatingDeleted>,
    ) -> anyhow::Result<()> {
        // No-op: ratings managed in separate read model, not in recipe aggregate
        Ok(())
    }

    /// Handle RecipeCopied event to store attribution metadata
    ///
    /// This event is emitted after RecipeCreated when a recipe is copied.
    /// It stores the original recipe ID and author for audit trail.
    async fn recipe_copied(
        &mut self,
        event: evento::EventDetails<RecipeCopied>,
    ) -> anyhow::Result<()> {
        self.original_recipe_id = Some(event.data.original_recipe_id);
        self.original_author = Some(event.data.original_author);
        Ok(())
    }

    /// Handle RecipeAccompanimentSettingsUpdated event to update accompaniment preferences
    ///
    /// This event handler updates the accompaniment settings when a user modifies
    /// whether the recipe accepts sides and which accompaniment categories it prefers.
    ///
    /// Epic 6: Enhanced Meal Planning System
    async fn recipe_accompaniment_settings_updated(
        &mut self,
        event: evento::EventDetails<RecipeAccompanimentSettingsUpdated>,
    ) -> anyhow::Result<()> {
        self.accepts_accompaniment = event.data.accepts_accompaniment;
        self.preferred_accompaniments = event.data.preferred_accompaniments;
        Ok(())
    }
}
