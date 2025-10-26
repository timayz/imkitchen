use bincode::{Decode, Encode};
use evento::AggregatorName;
use serde::{Deserialize, Serialize};

/// CourseType enum for course slot classification (AC-4)
/// Renamed from MealType to reflect new course-based meal planning model
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CourseType {
    Appetizer,
    MainCourse,
    Dessert,
}

impl CourseType {
    pub fn as_str(&self) -> &str {
        match self {
            CourseType::Appetizer => "appetizer",
            CourseType::MainCourse => "main_course",
            CourseType::Dessert => "dessert",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "appetizer" => Ok(CourseType::Appetizer),
            "main_course" => Ok(CourseType::MainCourse),
            "dessert" => Ok(CourseType::Dessert),
            // AC-9: Backward compatibility for old data
            "breakfast" => Ok(CourseType::Appetizer),
            "lunch" => Ok(CourseType::MainCourse),
            "dinner" => Ok(CourseType::Dessert),
            _ => Err(format!("Invalid course type: {}", s)),
        }
    }
}

// Keep old MealType as deprecated alias for backward compatibility
#[deprecated(since = "0.5.0", note = "Use CourseType instead")]
pub type MealType = CourseType;

/// Week status enum for multi-week meal planning (Story 6.3 AC-3)
///
/// Tracks the lifecycle state of a weekly meal plan within a multi-week batch.
/// Used to determine which weeks can be regenerated (only Future weeks are unlocked).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "snake_case")]
pub enum WeekStatus {
    /// Week hasn't started yet (start_date > today)
    Future,
    /// Today falls within week (start_date <= today <= end_date)
    Current,
    /// Week completed (end_date < today)
    Past,
    /// User manually archived (optional future feature)
    Archived,
}

/// Meal assignment representing a single recipe assigned to a course slot
///
/// AC-4: Renamed meal_type to course_type to reflect course-based model
/// Note: course_type stored as String for bincode compatibility (like complexity in RecipeTagged)
/// Story 6.3 AC-8: Added accompaniment_recipe_id field for main course pairing
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MealAssignment {
    pub date: String,                         // ISO 8601 date (YYYY-MM-DD)
    pub course_type: String, // AC-4: "appetizer", "main_course", or "dessert" (renamed from meal_type)
    pub recipe_id: String,   // Recipe assigned to this slot
    pub prep_required: bool, // True if recipe has advance_prep_hours > 0
    pub assignment_reasoning: Option<String>, // Human-readable explanation of assignment (Story 3.8)
    #[serde(default)]
    pub accompaniment_recipe_id: Option<String>, // Story 6.3 AC-8: Accompaniment for main courses
}

/// MealPlanGenerated event emitted when a new meal plan is generated
///
/// This event captures the complete meal plan generation result including all
/// 21 meal assignments (7 days × 3 meals) and rotation state tracking.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanGenerated {
    pub user_id: String,                       // Owner of the meal plan
    pub start_date: String,                    // ISO 8601 date (Monday of the week)
    pub meal_assignments: Vec<MealAssignment>, // 21 assignments (7 days × 3 meals)
    pub rotation_state_json: String,           // JSON serialized RotationState
    pub generated_at: String,                  // RFC3339 formatted timestamp
}

/// RecipeUsedInRotation event emitted when a recipe is used in meal plan generation
///
/// This event tracks recipe usage for the rotation system, ensuring recipes are
/// used exactly once before repeating. Emitted for each recipe assigned during generation.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RecipeUsedInRotation {
    pub recipe_id: String, // Recipe that was used
    pub cycle_number: u32, // Rotation cycle number
    pub used_at: String,   // RFC3339 formatted timestamp
}

/// RotationCycleReset event emitted when all favorites have been used once
///
/// This event triggers the start of a new rotation cycle, allowing all favorite
/// recipes to be used again in subsequent meal plans.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct RotationCycleReset {
    pub user_id: String,       // User whose rotation is resetting
    pub old_cycle_number: u32, // Previous cycle number
    pub new_cycle_number: u32, // New cycle number (old + 1)
    pub favorite_count: usize, // Total number of favorite recipes
    pub reset_at: String,      // RFC3339 formatted timestamp
}

/// MealPlanArchived event emitted when a meal plan is archived
///
/// Meal plans are archived when a new plan is generated or when the week ends.
/// Only one meal plan can be active per user at a time.
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanArchived {
    pub archived_at: String, // RFC3339 formatted timestamp
}

/// MealPlanRegenerated event emitted when user regenerates entire meal plan (Story 3.7)
///
/// This event replaces all 21 meal assignments with freshly generated recipes
/// while preserving rotation state (doesn't reset cycle).
///
/// Note: meal_plan_id is provided by event.aggregator_id, not stored in event data
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MealPlanRegenerated {
    pub new_assignments: Vec<MealAssignment>, // Fresh 21 assignments (7 days × 3 meals)
    pub rotation_state_json: String,          // Updated rotation state (cycle preserved)
    pub regeneration_reason: Option<String>,  // Optional reason for regeneration
    pub regenerated_at: String,               // RFC3339 formatted timestamp
}

// ============================================================================
// Epic 6: Multi-Week Meal Planning - Domain Model (Story 6.3)
// ============================================================================

/// WeekMealPlan represents a single week within a multi-week meal plan batch (Story 6.3 AC-1)
///
/// Each week contains 21 meal assignments (7 days × 3 courses: appetizer, main_course, dessert)
/// and tracks its own status, locking state, and shopping list.
///
/// # Week Locking Rules
/// - Current week (start_date <= today <= end_date) is locked (is_locked: true)
/// - Locked weeks cannot be regenerated (safety constraint)
/// - Future weeks (status: Future) can be regenerated individually or in batch
///
/// # Fields
/// - `id`: Unique identifier for this week (UUID)
/// - `user_id`: Owner of the meal plan
/// - `start_date`: Monday of the week (ISO 8601: YYYY-MM-DD)
/// - `end_date`: Sunday of the week (ISO 8601: YYYY-MM-DD)
/// - `status`: Week lifecycle state (Future, Current, Past, Archived)
/// - `is_locked`: Whether week can be regenerated (false for Future, true for Current)
/// - `generation_batch_id`: UUID linking all weeks created together
/// - `meal_assignments`: 21 meal assignments (7 days × 3 courses)
/// - `shopping_list_id`: Associated shopping list identifier
/// - `created_at`: RFC3339 timestamp when week was generated
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WeekMealPlan {
    pub id: String,
    pub user_id: String,
    pub start_date: String, // ISO 8601 date (Monday)
    pub end_date: String,   // ISO 8601 date (Sunday)
    pub status: WeekStatus,
    pub is_locked: bool,
    pub generation_batch_id: String, // UUID linking batch
    pub meal_assignments: Vec<MealAssignment>,
    pub shopping_list_id: String,
    pub created_at: String, // RFC3339 timestamp
}

/// MultiWeekMealPlan represents a batch of generated weeks with rotation tracking (Story 6.3 AC-2)
///
/// Contains all weeks generated in a single batch (up to 5 weeks maximum) and tracks
/// rotation state across weeks to ensure recipe variety and prevent main course repetition.
///
/// # Maximum Weeks
/// - Hard cap: 5 weeks per batch (prevents excessive computation)
/// - Actual weeks generated depends on favorite recipe counts and rotation rules
///
/// # Rotation State
/// - `rotation_state`: Tracks recipe usage across all weeks in batch
/// - Main courses MUST be unique (never repeat across weeks)
/// - Appetizers/desserts CAN repeat after exhausting full list
///
/// # Fields
/// - `user_id`: Owner of the multi-week plan
/// - `generation_batch_id`: UUID for this batch
/// - `generated_weeks`: Vector of WeekMealPlan (1-5 weeks)
/// - `rotation_state`: Cross-week rotation tracking
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MultiWeekMealPlan {
    pub user_id: String,
    pub generation_batch_id: String,
    pub generated_weeks: Vec<WeekMealPlan>,
    pub rotation_state: crate::rotation::RotationState,
}

/// WeekMealPlanData used in event payloads (lighter than full WeekMealPlan)
///
/// Contains essential data for event serialization without full struct overhead.
/// Used in MultiWeekMealPlanGenerated and AllFutureWeeksRegenerated events.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct WeekMealPlanData {
    pub id: String,
    pub start_date: String, // ISO 8601 date (Monday)
    pub end_date: String,   // ISO 8601 date (Sunday)
    pub status: WeekStatus,
    pub is_locked: bool,
    pub meal_assignments: Vec<MealAssignment>,
    pub shopping_list_id: String,
}

/// MultiWeekMealPlanGenerated event (Story 6.3 AC-5)
///
/// Emitted when algorithm generates multiple weeks simultaneously in a single batch.
/// This event creates all weeks (1-5 maximum) with rotation state tracking to ensure
/// recipe variety across weeks.
///
/// # Event Flow
/// 1. User requests multi-week generation (e.g., "Generate 3 weeks")
/// 2. Algorithm generates weeks with rotation tracking
/// 3. MultiWeekMealPlanGenerated event emitted with all weeks
/// 4. Aggregate handler creates/updates weeks in state
///
/// # Rotation State
/// - Tracks which recipes used across all weeks
/// - Ensures main courses never repeat
/// - Allows appetizers/desserts to repeat after exhausting full list
///
/// Note: meal_plan_id is provided by event.aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct MultiWeekMealPlanGenerated {
    pub generation_batch_id: String, // UUID for this batch
    pub user_id: String,
    pub weeks: Vec<WeekMealPlanData>, // 1-5 weeks generated
    pub rotation_state: crate::rotation::RotationState,
    pub generated_at: String, // RFC3339 timestamp
}

/// SingleWeekRegenerated event (Story 6.3 AC-6)
///
/// Emitted when user regenerates a single future week (unlocked only).
/// This event replaces meal assignments for one specific week while preserving
/// rotation state and leaving other weeks intact.
///
/// # Event Flow
/// 1. User clicks "Regenerate This Week" on Future week
/// 2. Algorithm generates new 21 assignments for that week
/// 3. SingleWeekRegenerated event emitted
/// 4. Aggregate handler updates only the specified week
///
/// # Locking Constraint
/// - Only Future (unlocked) weeks can be regenerated
/// - Current week (locked) cannot be regenerated (safety)
/// - Application layer enforces this before event emission
///
/// Note: meal_plan_id is provided by event.aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct SingleWeekRegenerated {
    pub week_id: String,                       // ID of week being regenerated
    pub week_start_date: String,               // ISO 8601 date (Monday)
    pub meal_assignments: Vec<MealAssignment>, // New 21 assignments for this week
    pub updated_rotation_state: crate::rotation::RotationState,
    pub regenerated_at: String, // RFC3339 timestamp
}

/// AllFutureWeeksRegenerated event (Story 6.3 AC-7)
///
/// Emitted when user regenerates all future weeks while preserving current week.
/// This event replaces meal assignments for all unlocked weeks (status: Future)
/// while keeping the current week (status: Current, is_locked: true) unchanged.
///
/// # Event Flow
/// 1. User clicks "Regenerate All Future Weeks"
/// 2. Algorithm identifies current week (locked) and future weeks (unlocked)
/// 3. Algorithm generates new weeks starting from next week
/// 4. AllFutureWeeksRegenerated event emitted
/// 5. Aggregate handler preserves current week, replaces future weeks
///
/// # Preserved Current Week
/// - Current week remains locked and unchanged
/// - `preserved_current_week_id` indicates which week was kept
/// - Rotation state continues from current week's recipes
///
/// Note: meal_plan_id is provided by event.aggregator_id
#[derive(Debug, Clone, Serialize, Deserialize, AggregatorName, Encode, Decode)]
pub struct AllFutureWeeksRegenerated {
    pub generation_batch_id: String, // New UUID for regenerated batch
    pub user_id: String,
    pub weeks: Vec<WeekMealPlanData>, // New future weeks (current week excluded)
    pub preserved_current_week_id: Option<String>, // ID of current week kept intact
    pub regenerated_at: String,       // RFC3339 timestamp
}
