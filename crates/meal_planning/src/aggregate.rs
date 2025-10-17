use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    MealAssignment, MealPlanArchived, MealPlanGenerated, MealPlanRegenerated, MealReplaced,
    RecipeUsedInRotation, RotationCycleReset,
};

/// MealPlanAggregate representing the state of a meal plan entity
///
/// This aggregate is rebuilt from events using the evento event sourcing framework.
/// It stores the complete state of a weekly meal plan including all meal assignments
/// and rotation tracking.
///
/// Note: All fields are String types for bincode compatibility (follows evento best practices)
#[derive(Debug, Default, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MealPlanAggregate {
    // Core identity
    pub meal_plan_id: String,
    pub user_id: String,

    // Meal plan metadata
    pub start_date: String,          // ISO 8601 date (Monday of the week)
    pub status: String,              // "active" or "archived"
    pub rotation_state_json: String, // JSON serialized RotationState

    // Meal assignments (7 days × 3 meals = 21 assignments)
    pub meal_assignments: Vec<MealAssignment>,

    // Timestamps
    pub created_at: String,          // RFC3339 formatted timestamp
    pub archived_at: Option<String>, // RFC3339 formatted timestamp if archived
}

/// Status of a meal plan (helper enum for type safety)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum MealPlanStatus {
    #[default]
    Active,
    Archived,
}

impl MealPlanStatus {
    pub fn as_str(&self) -> &str {
        match self {
            MealPlanStatus::Active => "active",
            MealPlanStatus::Archived => "archived",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "active" => Ok(MealPlanStatus::Active),
            "archived" => Ok(MealPlanStatus::Archived),
            _ => Err(format!("Invalid meal plan status: {}", s)),
        }
    }
}

/// Implement evento aggregator pattern for MealPlanAggregate
///
/// The #[evento::aggregator] macro generates:
/// - Aggregator trait implementation with event dispatching
/// - AggregatorName trait implementation
/// - Event replay functionality
#[evento::aggregator]
impl MealPlanAggregate {
    /// Handle MealPlanGenerated event to initialize aggregate state
    ///
    /// This is called when replaying events from the event store to rebuild
    /// the aggregate's current state.
    ///
    /// **Critical Fix 1.2:** Added validation for rotation state JSON
    async fn meal_plan_generated(
        &mut self,
        event: evento::EventDetails<MealPlanGenerated>,
    ) -> anyhow::Result<()> {
        // Validate rotation state JSON is parseable
        use crate::rotation::RotationState;
        let _rotation_state =
            RotationState::from_json(&event.data.rotation_state_json).map_err(|e| {
                anyhow::anyhow!(
                    "Invalid rotation state in MealPlanGenerated event for meal_plan_id={}: {}",
                    event.aggregator_id,
                    e
                )
            })?;

        self.meal_plan_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.start_date = event.data.start_date;
        self.meal_assignments = event.data.meal_assignments;
        self.rotation_state_json = event.data.rotation_state_json;
        self.created_at = event.data.generated_at.clone();
        self.status = MealPlanStatus::Active.as_str().to_string();
        self.archived_at = None;
        Ok(())
    }

    /// Handle RecipeUsedInRotation event to track rotation state
    ///
    /// This event is emitted for each recipe used during generation.
    /// The rotation state is already captured in MealPlanGenerated event,
    /// so this handler is primarily for cross-domain subscriptions.
    async fn recipe_used_in_rotation(
        &mut self,
        _event: evento::EventDetails<RecipeUsedInRotation>,
    ) -> anyhow::Result<()> {
        // Rotation state already updated in meal_plan_generated handler
        // This handler exists for potential cross-domain event subscriptions
        Ok(())
    }

    /// Handle RotationCycleReset event to reset rotation cycle
    ///
    /// This event is emitted when all favorite recipes have been used once,
    /// triggering a new rotation cycle. The rotation state in the aggregate
    /// is updated via the rotation_state_json field.
    ///
    /// **Critical Fix 1.2:** Replaced silent error handling with explicit error propagation
    async fn rotation_cycle_reset(
        &mut self,
        event: evento::EventDetails<RotationCycleReset>,
    ) -> anyhow::Result<()> {
        // Parse current rotation state with explicit error handling
        use crate::rotation::RotationState;
        let mut rotation_state =
            RotationState::from_json(&self.rotation_state_json).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse rotation state for meal_plan_id={}: {}",
                    self.meal_plan_id,
                    e
                )
            })?;

        // Reset the cycle
        rotation_state.reset_cycle();
        rotation_state.total_favorite_count = event.data.favorite_count;

        // Update aggregate state
        self.rotation_state_json = rotation_state.to_json()?;

        Ok(())
    }

    /// Handle MealPlanArchived event to mark meal plan as archived
    ///
    /// Archiving a meal plan makes it inactive so a new plan can be generated.
    /// Only one meal plan can be active per user at a time.
    async fn meal_plan_archived(
        &mut self,
        event: evento::EventDetails<MealPlanArchived>,
    ) -> anyhow::Result<()> {
        self.status = MealPlanStatus::Archived.as_str().to_string();
        self.archived_at = Some(event.data.archived_at);
        Ok(())
    }

    /// Handle MealReplaced event to update a specific meal assignment
    ///
    /// This event handler supports the "Replace Individual Meal" feature (Story 3.2)
    /// by swapping out a single recipe while preserving the rest of the plan.
    ///
    /// **Major Fix 2.3**: Updates rotation state to mark new recipe as used and
    /// remove old recipe from used set, maintaining rotation integrity.
    async fn meal_replaced(
        &mut self,
        event: evento::EventDetails<MealReplaced>,
    ) -> anyhow::Result<()> {
        // Find the meal assignment matching the date and meal_type
        if let Some(assignment) = self
            .meal_assignments
            .iter_mut()
            .find(|a| a.date == event.data.date && a.meal_type == event.data.meal_type)
        {
            assignment.recipe_id = event.data.new_recipe_id.clone();
        }

        // Update rotation state to reflect the recipe swap
        let mut rotation_state =
            crate::rotation::RotationState::from_json(&self.rotation_state_json).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse rotation state in meal_replaced for meal_plan_id={}: {}",
                    self.meal_plan_id,
                    e
                )
            })?;

        // Remove old recipe from used set (if present)
        rotation_state
            .used_recipe_ids
            .remove(&event.data.old_recipe_id);

        // Mark new recipe as used
        rotation_state.mark_recipe_used(event.data.new_recipe_id.clone());

        // Save updated rotation state back to JSON
        self.rotation_state_json = rotation_state.to_json().map_err(|e| {
            anyhow::anyhow!("Failed to serialize rotation state in meal_replaced: {}", e)
        })?;

        Ok(())
    }

    /// Handle MealPlanRegenerated event to replace all meal assignments (Story 3.7)
    ///
    /// This event handler supports the "Regenerate Full Meal Plan" feature by replacing
    /// all 21 meal assignments with freshly generated recipes while preserving rotation state.
    ///
    /// **Rotation Integrity**: Rotation state updated with new recipe usage, cycle preserved.
    async fn meal_plan_regenerated(
        &mut self,
        event: evento::EventDetails<MealPlanRegenerated>,
    ) -> anyhow::Result<()> {
        // Replace all meal assignments with new assignments
        self.meal_assignments = event.data.new_assignments;

        // Update rotation state (preserved, not reset)
        self.rotation_state_json = event.data.rotation_state_json;

        Ok(())
    }
}
