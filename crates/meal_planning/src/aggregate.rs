use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{
    AllFutureWeeksRegenerated, MealAssignment, MealPlanArchived, MealPlanGenerated,
    MealPlanRegenerated, MultiWeekMealPlanGenerated, RecipeUsedInRotation, RotationCycleReset,
    SingleWeekRegenerated,
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

    // ============================================================================
    // Epic 6: Multi-Week Event Handlers (Story 6.3 AC-5, AC-6, AC-7)
    // ============================================================================

    /// Handle MultiWeekMealPlanGenerated event (Story 6.3 AC-5)
    ///
    /// This event handler processes multi-week meal plan generation by storing
    /// all generated weeks and rotation state. The aggregate tracks the first week's
    /// data in its root fields for backwards compatibility.
    ///
    /// **Multi-Week Storage Strategy:**
    /// - Aggregate root fields (start_date, meal_assignments) store FIRST week only
    /// - Full multi-week data stored in separate read model (Story 6.4)
    /// - rotation_state_json tracks usage across ALL weeks
    ///
    /// **Event Flow:**
    /// 1. Algorithm generates 1-5 weeks simultaneously
    /// 2. MultiWeekMealPlanGenerated event emitted with all weeks
    /// 3. This handler updates aggregate with first week data
    /// 4. Projection handler (Story 6.4) stores all weeks in read model
    async fn multi_week_meal_plan_generated(
        &mut self,
        event: evento::EventDetails<MultiWeekMealPlanGenerated>,
    ) -> anyhow::Result<()> {
        // Validate rotation state JSON is parseable
        let rotation_state_json = event.data.rotation_state.to_json()?;

        // Validate at least one week generated
        if event.data.weeks.is_empty() {
            return Err(anyhow::anyhow!(
                "MultiWeekMealPlanGenerated event must contain at least 1 week"
            ));
        }

        // Store first week in aggregate root (backwards compatibility)
        let first_week = &event.data.weeks[0];
        self.meal_plan_id = event.aggregator_id.clone();
        self.user_id = event.data.user_id;
        self.start_date = first_week.start_date.clone();
        self.meal_assignments = first_week.meal_assignments.clone();
        self.rotation_state_json = rotation_state_json;
        self.created_at = event.data.generated_at.clone();
        self.status = "active".to_string();
        self.archived_at = None;

        Ok(())
    }

    /// Handle SingleWeekRegenerated event (Story 6.3 AC-6)
    ///
    /// This event handler processes single-week regeneration by updating meal assignments
    /// for one specific week while preserving all other weeks and rotation state.
    ///
    /// **Single Week Update Strategy:**
    /// - If regenerated week is FIRST week: update aggregate root fields
    /// - If regenerated week is NOT first: only projection handler updates (Story 6.4)
    /// - rotation_state_json updated with new recipe usage
    ///
    /// **Locking Safety:**
    /// - Application layer prevents locked (current) week regeneration before event emission
    /// - This handler assumes validation already passed
    async fn single_week_regenerated(
        &mut self,
        event: evento::EventDetails<SingleWeekRegenerated>,
    ) -> anyhow::Result<()> {
        // Validate rotation state JSON is parseable
        let rotation_state_json = event.data.updated_rotation_state.to_json()?;

        // Update rotation state
        self.rotation_state_json = rotation_state_json;

        // If regenerated week matches aggregate's start_date, update root meal assignments
        // (This handles the case where the first week in the multi-week plan is regenerated)
        if self.start_date == event.data.week_start_date {
            self.meal_assignments = event.data.meal_assignments;
        }

        // Note: Projection handler (Story 6.4) updates the specific week in read model

        Ok(())
    }

    /// Handle AllFutureWeeksRegenerated event (Story 6.3 AC-7)
    ///
    /// This event handler processes regeneration of all future weeks while preserving
    /// the current locked week. Updates aggregate with new first future week data.
    ///
    /// **Future Weeks Regeneration Strategy:**
    /// - Current week (locked) preserved and NOT included in event.data.weeks
    /// - All future weeks replaced with new generations
    /// - Aggregate root stores first FUTURE week (not current week)
    /// - Projection handler (Story 6.4) preserves current week in read model
    ///
    /// **Event Flow:**
    /// 1. User clicks "Regenerate All Future Weeks"
    /// 2. Algorithm generates new weeks starting from next Monday (current+1)
    /// 3. AllFutureWeeksRegenerated event emitted with future weeks only
    /// 4. This handler updates aggregate with first future week
    /// 5. Projection handler preserves current week, replaces future weeks
    async fn all_future_weeks_regenerated(
        &mut self,
        event: evento::EventDetails<AllFutureWeeksRegenerated>,
    ) -> anyhow::Result<()> {
        // Validate at least one future week generated
        if event.data.weeks.is_empty() {
            return Err(anyhow::anyhow!(
                "AllFutureWeeksRegenerated event must contain at least 1 future week"
            ));
        }

        // Store first future week in aggregate root
        let first_future_week = &event.data.weeks[0];
        self.start_date = first_future_week.start_date.clone();
        self.meal_assignments = first_future_week.meal_assignments.clone();

        // Note: rotation_state is embedded in weeks data, extract from first week's context
        // For now, we'll maintain the existing rotation_state_json
        // (Story 6.5 will implement full rotation state management)

        Ok(())
    }
}
