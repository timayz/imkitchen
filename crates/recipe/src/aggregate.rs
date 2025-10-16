use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    Ingredient, InstructionStep, RatingDeleted, RatingUpdated, RecipeCreated, RecipeDeleted,
    RecipeFavorited, RecipeRated, RecipeShared, RecipeTagged, RecipeUpdated,
};
use crate::tagging::{Complexity, RecipeTags};

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
    async fn recipe_created(
        &mut self,
        event: evento::EventDetails<RecipeCreated>,
    ) -> anyhow::Result<()> {
        self.recipe_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.title = event.data.title;
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
    async fn recipe_updated(
        &mut self,
        event: evento::EventDetails<RecipeUpdated>,
    ) -> anyhow::Result<()> {
        if let Some(title) = event.data.title {
            self.title = title;
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
}
