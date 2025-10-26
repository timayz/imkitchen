use bincode::{Decode, Encode};
use chrono;
use recipe::Cuisine;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// RotationState tracks which recipes have been used in the current rotation cycle
///
/// The rotation system ensures each favorite recipe is used exactly once before
/// any recipe repeats. When all favorites have been used once, the cycle resets.
///
/// **Multi-Week Rotation Business Rules (Story 6.5):**
/// - **Main Courses**: MUST be unique across ALL generated weeks (strict uniqueness - never repeat)
/// - **Appetizers**: CAN repeat after all available appetizers used once (soft rotation with reset)
/// - **Desserts**: CAN repeat after all available desserts used once (soft rotation with reset)
/// - **Accompaniments**: NOT tracked in rotation state (can repeat freely)
/// - **Cuisine Variety**: Tracked via `cuisine_usage_count` to promote diversity
/// - **Complexity Spacing**: `last_complex_meal_date` avoids consecutive high-complexity meals
///
/// # Storage
/// This state is persisted in two locations:
/// - As JSON in the `meal_plans.rotation_state` column (database)
/// - As bincode-encoded events in the evento event store
///
/// # Example Usage
/// ```
/// use meal_planning::RotationState;
/// use recipe::Cuisine;
///
/// // Initialize new rotation state
/// let mut state = RotationState::new();
///
/// // Track main course usage (strict uniqueness)
/// state.mark_used_main_course("main-course-1");
/// assert!(state.is_main_course_used("main-course-1"));
///
/// // Track appetizer usage with soft reset
/// state.mark_used_appetizer("app-1");
/// state.mark_used_appetizer("app-2");
/// state.mark_used_appetizer("app-3");
/// state.reset_appetizers_if_all_used(3); // Clears list when all exhausted
///
/// // Track cuisine variety
/// state.increment_cuisine_usage(&Cuisine::Italian);
/// assert_eq!(state.get_cuisine_usage(&Cuisine::Italian), 1);
///
/// // Track complex meal spacing
/// state.update_last_complex_meal_date("2025-10-27");
/// ```
///
/// # Serialization
/// Supports both `serde` (JSON for database) and `bincode` (evento events):
/// ```
/// # use meal_planning::RotationState;
/// let state = RotationState::new();
///
/// // JSON serialization (for database storage)
/// let json = state.to_json().unwrap();
/// let deserialized = RotationState::from_json(&json).unwrap();
///
/// // Bincode serialization (for evento event storage)
/// use bincode::{Encode, Decode};
/// let encoded = bincode::encode_to_vec(&state, bincode::config::standard()).unwrap();
/// let (decoded, _): (RotationState, _) =
///     bincode::decode_from_slice(&encoded, bincode::config::standard()).unwrap();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct RotationState {
    // Pre-Epic 6 fields (backwards compatible)
    pub cycle_number: u32,
    pub cycle_started_at: String, // RFC3339 formatted timestamp
    pub used_recipe_ids: HashSet<String>,
    pub total_favorite_count: usize,

    // Epic 6: Multi-week rotation tracking (Story 6.3 AC-4)
    #[serde(default)]
    pub used_main_course_ids: Vec<String>, // Main courses MUST be unique (never repeat)
    #[serde(default)]
    pub used_appetizer_ids: Vec<String>, // Can repeat after all used once
    #[serde(default)]
    pub used_dessert_ids: Vec<String>, // Can repeat after all used once
    #[serde(default)]
    pub cuisine_usage_count: HashMap<Cuisine, u32>, // Tracks cuisine variety
    #[serde(default)]
    pub last_complex_meal_date: Option<String>, // ISO 8601 date (avoid consecutive complex)
}

impl RotationState {
    /// Create a new rotation state starting at cycle 1
    pub fn new() -> Self {
        RotationState {
            cycle_number: 1,
            cycle_started_at: chrono::Utc::now().to_rfc3339(),
            used_recipe_ids: HashSet::new(),
            total_favorite_count: 0,
            // Epic 6 fields (Story 6.3 AC-4)
            used_main_course_ids: Vec::new(),
            used_appetizer_ids: Vec::new(),
            used_dessert_ids: Vec::new(),
            cuisine_usage_count: HashMap::new(),
            last_complex_meal_date: None,
        }
    }

    /// Create a new rotation state with a specific favorite count
    ///
    /// Returns an error if total_favorite_count is 0, as rotation requires at least 1 recipe.
    pub fn with_favorite_count(total_favorite_count: usize) -> Result<Self, String> {
        if total_favorite_count == 0 {
            return Err("total_favorite_count must be greater than 0".to_string());
        }

        Ok(RotationState {
            cycle_number: 1,
            cycle_started_at: chrono::Utc::now().to_rfc3339(),
            used_recipe_ids: HashSet::new(),
            total_favorite_count,
            // Epic 6 fields (Story 6.3 AC-4)
            used_main_course_ids: Vec::new(),
            used_appetizer_ids: Vec::new(),
            used_dessert_ids: Vec::new(),
            cuisine_usage_count: HashMap::new(),
            last_complex_meal_date: None,
        })
    }

    /// Check if cycle should reset based on current state
    pub fn should_reset_cycle(&self) -> bool {
        self.used_recipe_ids.len() >= self.total_favorite_count && self.total_favorite_count > 0
    }

    /// Mark a recipe as used in the current cycle
    pub fn mark_recipe_used(&mut self, recipe_id: String) {
        self.used_recipe_ids.insert(recipe_id);
    }

    /// Check if a recipe has been used in the current cycle
    pub fn is_recipe_used(&self, recipe_id: &str) -> bool {
        self.used_recipe_ids.contains(recipe_id)
    }

    /// Remove a recipe from the used set (Story 3.6: return recipe to pool after replacement)
    ///
    /// Returns Ok(()) if the recipe was successfully unmarked, or Err if it wasn't in the used set.
    /// This is used when replacing a meal slot - the old recipe returns to the rotation pool.
    pub fn unmark_recipe_used(&mut self, recipe_id: &str) -> Result<(), String> {
        if self.used_recipe_ids.remove(recipe_id) {
            Ok(())
        } else {
            Err(format!(
                "Recipe {} was not marked as used in cycle {}",
                recipe_id, self.cycle_number
            ))
        }
    }

    /// Reset the cycle when all favorites have been used once
    ///
    /// This allows recipes to be reused in subsequent meal plans.
    /// Uses saturating_add to prevent overflow if cycle_number reaches u32::MAX.
    pub fn reset_cycle(&mut self) {
        self.cycle_number = self.cycle_number.saturating_add(1);
        self.cycle_started_at = chrono::Utc::now().to_rfc3339();
        self.used_recipe_ids.clear();
    }

    /// Get count of recipes used in current cycle
    pub fn used_count(&self) -> usize {
        self.used_recipe_ids.len()
    }

    /// Serialize to JSON string for database storage
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    // ============================================================================
    // Epic 6: Multi-Week Rotation Helpers (Story 6.3 AC-4)
    // ============================================================================

    /// Mark a main course as used (Story 6.3 AC-4)
    ///
    /// Main courses MUST be unique across all weeks - they never repeat.
    /// This method adds the recipe_id to used_main_course_ids vector.
    ///
    /// # Arguments
    /// * `recipe_id` - ID of main course recipe to mark as used
    ///
    /// # Uniqueness
    /// This method does NOT check for duplicates - caller must verify uniqueness
    /// using `is_main_course_used()` before calling this method.
    pub fn mark_used_main_course(&mut self, recipe_id: &str) {
        if !self.used_main_course_ids.contains(&recipe_id.to_string()) {
            self.used_main_course_ids.push(recipe_id.to_string());
        }
    }

    /// Mark an appetizer as used (Story 6.3 AC-4)
    ///
    /// Appetizers CAN repeat after all appetizers have been used once.
    /// Tracks usage for rotation fairness.
    pub fn mark_used_appetizer(&mut self, recipe_id: &str) {
        self.used_appetizer_ids.push(recipe_id.to_string());
    }

    /// Mark a dessert as used (Story 6.3 AC-4)
    ///
    /// Desserts CAN repeat after all desserts have been used once.
    /// Tracks usage for rotation fairness.
    pub fn mark_used_dessert(&mut self, recipe_id: &str) {
        self.used_dessert_ids.push(recipe_id.to_string());
    }

    /// Check if a main course has been used (Story 6.3 AC-4)
    ///
    /// Returns true if recipe_id is in used_main_course_ids vector.
    /// Use this to enforce main course uniqueness constraint.
    ///
    /// # Arguments
    /// * `recipe_id` - ID of main course recipe to check
    ///
    /// # Returns
    /// * `true` if main course already used (cannot be assigned again)
    /// * `false` if main course not yet used (available for assignment)
    pub fn is_main_course_used(&self, recipe_id: &str) -> bool {
        self.used_main_course_ids.contains(&recipe_id.to_string())
    }

    /// Reset appetizer tracking if all available appetizers have been used (Story 6.5 AC-5)
    ///
    /// Clears used_appetizer_ids list when all available appetizers exhausted,
    /// allowing appetizers to repeat in subsequent assignments.
    ///
    /// # Arguments
    /// * `available_count` - Total number of available appetizer recipes
    ///
    /// # Behavior
    /// - If `used_appetizer_ids.len() >= available_count`, clears the list
    /// - Otherwise, leaves the list unchanged
    /// - Handles edge case: `available_count == 0` safely (clears list)
    pub fn reset_appetizers_if_all_used(&mut self, available_count: usize) {
        if self.used_appetizer_ids.len() >= available_count {
            self.used_appetizer_ids.clear();
        }
    }

    /// Reset dessert tracking if all available desserts have been used (Story 6.5 AC-5)
    ///
    /// Clears used_dessert_ids list when all available desserts exhausted,
    /// allowing desserts to repeat in subsequent assignments.
    ///
    /// # Arguments
    /// * `available_count` - Total number of available dessert recipes
    ///
    /// # Behavior
    /// - If `used_dessert_ids.len() >= available_count`, clears the list
    /// - Otherwise, leaves the list unchanged
    /// - Handles edge case: `available_count == 0` safely (clears list)
    pub fn reset_desserts_if_all_used(&mut self, available_count: usize) {
        if self.used_dessert_ids.len() >= available_count {
            self.used_dessert_ids.clear();
        }
    }

    /// Increment cuisine usage count (Story 6.3 AC-4)
    ///
    /// Tracks how many times each cuisine has been used across weeks.
    /// Used by algorithm to prefer less-frequently-used cuisines for variety.
    ///
    /// # Arguments
    /// * `cuisine` - Cuisine enum variant (e.g., Cuisine::Italian, Cuisine::Indian)
    pub fn increment_cuisine_usage(&mut self, cuisine: &Cuisine) {
        *self.cuisine_usage_count.entry(cuisine.clone()).or_insert(0) += 1;
    }

    /// Get cuisine usage count (Story 6.3 AC-4)
    ///
    /// Returns how many times a specific cuisine has been used.
    /// Returns 0 if cuisine has never been used.
    pub fn get_cuisine_usage(&self, cuisine: &Cuisine) -> u32 {
        *self.cuisine_usage_count.get(cuisine).unwrap_or(&0)
    }

    /// Update last complex meal date (Story 6.3 AC-4)
    ///
    /// Tracks the date of the last complex meal to avoid consecutive complex meals.
    /// Algorithm uses this to space out complex recipes across the week.
    ///
    /// # Arguments
    /// * `date` - ISO 8601 date string (YYYY-MM-DD)
    pub fn update_last_complex_meal_date(&mut self, date: &str) {
        self.last_complex_meal_date = Some(date.to_string());
    }
}

impl Default for RotationState {
    fn default() -> Self {
        Self::new()
    }
}

/// RotationSystem manages recipe rotation logic for meal planning
///
/// Ensures recipes are used exactly once before repeating, and handles
/// cycle resets when all favorites have been used.
pub struct RotationSystem;

impl RotationSystem {
    /// Filter recipes to only those not yet used in the current cycle
    ///
    /// Returns a list of recipe IDs that are available for assignment.
    pub fn filter_available_recipes(
        all_favorite_ids: &[String],
        rotation_state: &RotationState,
    ) -> Vec<String> {
        all_favorite_ids
            .iter()
            .filter(|id| !rotation_state.is_recipe_used(id))
            .cloned()
            .collect()
    }

    /// Check if cycle should reset (all favorites have been used)
    ///
    /// Returns true if used_count equals total favorite count.
    pub fn should_reset_cycle(total_favorite_count: usize, rotation_state: &RotationState) -> bool {
        rotation_state.used_count() >= total_favorite_count
    }

    /// Update rotation state after meal plan generation
    ///
    /// Marks all assigned recipes as used, and resets the cycle if needed.
    pub fn update_after_generation(
        assigned_recipe_ids: &[String],
        total_favorite_count: usize,
        mut rotation_state: RotationState,
    ) -> RotationState {
        // Mark all assigned recipes as used
        for recipe_id in assigned_recipe_ids {
            rotation_state.mark_recipe_used(recipe_id.clone());
        }

        // Reset cycle if all favorites have been used
        if Self::should_reset_cycle(total_favorite_count, &rotation_state) {
            rotation_state.reset_cycle();
        }

        rotation_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_state_new() {
        let state = RotationState::new();
        assert_eq!(state.cycle_number, 1);
        assert_eq!(state.used_count(), 0);
        assert_eq!(state.total_favorite_count, 0);
        assert!(!state.cycle_started_at.is_empty());
    }

    #[test]
    fn test_rotation_state_with_favorite_count() {
        let state = RotationState::with_favorite_count(15).unwrap();
        assert_eq!(state.cycle_number, 1);
        assert_eq!(state.used_count(), 0);
        assert_eq!(state.total_favorite_count, 15);
        assert!(!state.cycle_started_at.is_empty());
    }

    #[test]
    fn test_rotation_state_with_favorite_count_validation() {
        // Should fail with 0 favorites
        let result = RotationState::with_favorite_count(0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "total_favorite_count must be greater than 0"
        );

        // Should succeed with valid count
        let state = RotationState::with_favorite_count(5).unwrap();
        assert_eq!(state.total_favorite_count, 5);
    }

    #[test]
    fn test_should_reset_cycle_method() {
        let mut state = RotationState::with_favorite_count(3).unwrap();
        assert!(!state.should_reset_cycle());

        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        assert!(!state.should_reset_cycle());

        state.mark_recipe_used("recipe_3".to_string());
        assert!(state.should_reset_cycle());
    }

    #[test]
    fn test_mark_recipe_used() {
        let mut state = RotationState::new();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());

        assert_eq!(state.used_count(), 2);
        assert!(state.is_recipe_used("recipe_1"));
        assert!(state.is_recipe_used("recipe_2"));
        assert!(!state.is_recipe_used("recipe_3"));
    }

    #[test]
    fn test_reset_cycle() {
        let mut state = RotationState::new();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());

        assert_eq!(state.cycle_number, 1);
        assert_eq!(state.used_count(), 2);

        state.reset_cycle();

        assert_eq!(state.cycle_number, 2);
        assert_eq!(state.used_count(), 0);
        assert!(!state.is_recipe_used("recipe_1"));
    }

    #[test]
    fn test_reset_cycle_overflow_protection() {
        let mut state = RotationState::new();
        state.cycle_number = u32::MAX;

        // Should saturate at u32::MAX instead of overflowing
        state.reset_cycle();
        assert_eq!(state.cycle_number, u32::MAX);
        assert_eq!(state.used_count(), 0);
    }

    #[test]
    fn test_json_serialization() {
        let mut state = RotationState::new();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());

        let json = state.to_json().unwrap();
        let deserialized = RotationState::from_json(&json).unwrap();

        assert_eq!(deserialized.cycle_number, state.cycle_number);
        assert_eq!(deserialized.used_count(), state.used_count());
        assert!(deserialized.is_recipe_used("recipe_1"));
        assert!(deserialized.is_recipe_used("recipe_2"));
    }

    #[test]
    fn test_filter_available_recipes() {
        let all_favorites = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
        ];

        let mut rotation_state = RotationState::new();
        rotation_state.mark_recipe_used("recipe_1".to_string());

        let available = RotationSystem::filter_available_recipes(&all_favorites, &rotation_state);

        assert_eq!(available.len(), 2);
        assert!(available.contains(&"recipe_2".to_string()));
        assert!(available.contains(&"recipe_3".to_string()));
        assert!(!available.contains(&"recipe_1".to_string()));
    }

    #[test]
    fn test_should_reset_cycle() {
        let mut rotation_state = RotationState::new();
        rotation_state.mark_recipe_used("recipe_1".to_string());
        rotation_state.mark_recipe_used("recipe_2".to_string());

        // Not yet time to reset (2 used out of 3 total)
        assert!(!RotationSystem::should_reset_cycle(3, &rotation_state));

        rotation_state.mark_recipe_used("recipe_3".to_string());

        // All 3 favorites used, should reset
        assert!(RotationSystem::should_reset_cycle(3, &rotation_state));
    }

    #[test]
    fn test_update_after_generation() {
        let assigned_ids = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
        ];
        let total_favorites = 3;
        let initial_state = RotationState::new();

        let updated_state =
            RotationSystem::update_after_generation(&assigned_ids, total_favorites, initial_state);

        // All 3 recipes used, cycle should have reset
        assert_eq!(updated_state.cycle_number, 2);
        assert_eq!(updated_state.used_count(), 0);
    }

    // ============================================================================
    // Epic 6: Multi-Week Helper Method Tests (Story 6.3)
    // ============================================================================

    #[test]
    fn test_mark_used_main_course_epic6() {
        let mut state = RotationState::new();

        // Mark main courses as used
        state.mark_used_main_course("main-1");
        state.mark_used_main_course("main-2");

        assert_eq!(state.used_main_course_ids.len(), 2);
        assert!(state.is_main_course_used("main-1"));
        assert!(state.is_main_course_used("main-2"));
        assert!(!state.is_main_course_used("main-3"));

        // Verify uniqueness - duplicate should not be added
        state.mark_used_main_course("main-1");
        assert_eq!(state.used_main_course_ids.len(), 2); // Still 2
    }

    #[test]
    fn test_mark_used_appetizer_epic6() {
        let mut state = RotationState::new();

        // Appetizers CAN repeat
        state.mark_used_appetizer("app-1");
        state.mark_used_appetizer("app-2");
        state.mark_used_appetizer("app-1"); // Duplicate allowed

        assert_eq!(state.used_appetizer_ids.len(), 3); // Both occurrences tracked
    }

    #[test]
    fn test_mark_used_dessert_epic6() {
        let mut state = RotationState::new();

        // Desserts CAN repeat
        state.mark_used_dessert("dessert-1");
        state.mark_used_dessert("dessert-2");
        state.mark_used_dessert("dessert-1"); // Duplicate allowed

        assert_eq!(state.used_dessert_ids.len(), 3); // Both occurrences tracked
    }

    #[test]
    fn test_increment_cuisine_usage_epic6() {
        use recipe::Cuisine;

        let mut state = RotationState::new();

        // Track cuisine usage
        state.increment_cuisine_usage(&Cuisine::Italian);
        state.increment_cuisine_usage(&Cuisine::Italian);
        state.increment_cuisine_usage(&Cuisine::Indian);

        assert_eq!(state.get_cuisine_usage(&Cuisine::Italian), 2);
        assert_eq!(state.get_cuisine_usage(&Cuisine::Indian), 1);
        assert_eq!(state.get_cuisine_usage(&Cuisine::Mexican), 0); // Never used
    }

    #[test]
    fn test_update_last_complex_meal_date_epic6() {
        let mut state = RotationState::new();

        assert_eq!(state.last_complex_meal_date, None);

        state.update_last_complex_meal_date("2025-10-27");

        assert_eq!(state.last_complex_meal_date, Some("2025-10-27".to_string()));

        // Can be updated
        state.update_last_complex_meal_date("2025-10-28");
        assert_eq!(state.last_complex_meal_date, Some("2025-10-28".to_string()));
    }

    #[test]
    fn test_epic6_fields_initialized_in_constructor() {
        let state = RotationState::new();

        // Verify all Epic 6 fields start empty/None
        assert_eq!(state.used_main_course_ids.len(), 0);
        assert_eq!(state.used_appetizer_ids.len(), 0);
        assert_eq!(state.used_dessert_ids.len(), 0);
        assert_eq!(state.cuisine_usage_count.len(), 0);
        assert_eq!(state.last_complex_meal_date, None);
    }

    #[test]
    fn test_update_after_generation_partial() {
        let assigned_ids = vec!["recipe_1".to_string(), "recipe_2".to_string()];
        let total_favorites = 5;
        let initial_state = RotationState::new();

        let updated_state =
            RotationSystem::update_after_generation(&assigned_ids, total_favorites, initial_state);

        // Only 2 out of 5 used, cycle should NOT reset
        assert_eq!(updated_state.cycle_number, 1);
        assert_eq!(updated_state.used_count(), 2);
        assert!(updated_state.is_recipe_used("recipe_1"));
        assert!(updated_state.is_recipe_used("recipe_2"));
    }

    /// Story 3.6 - Test unmark_recipe_used() removes recipe from used set
    #[test]
    fn test_unmark_recipe_used_success() {
        let mut state = RotationState::new();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());

        assert_eq!(state.used_count(), 2);
        assert!(state.is_recipe_used("recipe_1"));

        // Unmark recipe_1
        let result = state.unmark_recipe_used("recipe_1");
        assert!(result.is_ok());

        // Recipe should be removed from used set
        assert_eq!(state.used_count(), 1);
        assert!(!state.is_recipe_used("recipe_1"));
        assert!(state.is_recipe_used("recipe_2"));
    }

    /// Story 3.6 - Test unmark_recipe_used() returns error if recipe not in used set
    #[test]
    fn test_unmark_recipe_used_not_found() {
        let mut state = RotationState::new();
        state.mark_recipe_used("recipe_1".to_string());

        // Try to unmark a recipe that was never marked
        let result = state.unmark_recipe_used("recipe_2");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Recipe recipe_2 was not marked as used"));

        // Original state unchanged
        assert_eq!(state.used_count(), 1);
        assert!(state.is_recipe_used("recipe_1"));
    }

    /// Story 3.6 - Test recipe becomes available after unmarking
    #[test]
    fn test_recipe_available_after_unmark() {
        let all_favorites = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
        ];

        let mut rotation_state = RotationState::new();
        rotation_state.mark_recipe_used("recipe_1".to_string());
        rotation_state.mark_recipe_used("recipe_2".to_string());

        // Only recipe_3 available
        let available = RotationSystem::filter_available_recipes(&all_favorites, &rotation_state);
        assert_eq!(available.len(), 1);
        assert!(available.contains(&"recipe_3".to_string()));

        // Unmark recipe_1
        rotation_state.unmark_recipe_used("recipe_1").unwrap();

        // Now recipe_1 and recipe_3 are available
        let available = RotationSystem::filter_available_recipes(&all_favorites, &rotation_state);
        assert_eq!(available.len(), 2);
        assert!(available.contains(&"recipe_1".to_string()));
        assert!(available.contains(&"recipe_3".to_string()));
    }

    /// Story 3.6 - Test replacement maintains rotation cycle integrity
    #[test]
    fn test_replacement_maintains_rotation_integrity() {
        let mut state = RotationState::with_favorite_count(5).unwrap();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        state.mark_recipe_used("recipe_3".to_string());

        assert_eq!(state.used_count(), 3);
        assert_eq!(state.cycle_number, 1);

        // Replace recipe_2 with recipe_4 (unmark old, mark new)
        state.unmark_recipe_used("recipe_2").unwrap();
        state.mark_recipe_used("recipe_4".to_string());

        // Still 3 recipes used, no cycle reset
        assert_eq!(state.used_count(), 3);
        assert_eq!(state.cycle_number, 1);
        assert!(!state.is_recipe_used("recipe_2"));
        assert!(state.is_recipe_used("recipe_4"));
    }

    // ============================================================================
    // Story 6.5: Reset Methods Tests (AC-5)
    // ============================================================================

    #[test]
    fn test_reset_appetizers_if_all_used_clears_when_exhausted() {
        let mut state = RotationState::new();

        // Mark 3 appetizers as used
        state.mark_used_appetizer("app-1");
        state.mark_used_appetizer("app-2");
        state.mark_used_appetizer("app-3");

        assert_eq!(state.used_appetizer_ids.len(), 3);

        // Call reset with count=3 (all used)
        state.reset_appetizers_if_all_used(3);

        // List should be cleared
        assert_eq!(state.used_appetizer_ids.len(), 0);
    }

    #[test]
    fn test_reset_appetizers_if_all_used_does_not_clear_when_partial() {
        let mut state = RotationState::new();

        // Mark 2 appetizers as used
        state.mark_used_appetizer("app-1");
        state.mark_used_appetizer("app-2");

        assert_eq!(state.used_appetizer_ids.len(), 2);

        // Call reset with count=3 (not all used yet)
        state.reset_appetizers_if_all_used(3);

        // List should NOT be cleared
        assert_eq!(state.used_appetizer_ids.len(), 2);
        assert_eq!(state.used_appetizer_ids[0], "app-1");
        assert_eq!(state.used_appetizer_ids[1], "app-2");
    }

    #[test]
    fn test_reset_appetizers_edge_case_available_count_zero() {
        let mut state = RotationState::new();

        // Edge case: no appetizers available
        state.reset_appetizers_if_all_used(0);

        // Should not panic, list should be cleared (0 >= 0)
        assert_eq!(state.used_appetizer_ids.len(), 0);
    }

    #[test]
    fn test_reset_desserts_if_all_used_clears_when_exhausted() {
        let mut state = RotationState::new();

        // Mark 5 desserts as used
        state.mark_used_dessert("dessert-1");
        state.mark_used_dessert("dessert-2");
        state.mark_used_dessert("dessert-3");
        state.mark_used_dessert("dessert-4");
        state.mark_used_dessert("dessert-5");

        assert_eq!(state.used_dessert_ids.len(), 5);

        // Call reset with count=5 (all used)
        state.reset_desserts_if_all_used(5);

        // List should be cleared
        assert_eq!(state.used_dessert_ids.len(), 0);
    }

    #[test]
    fn test_reset_desserts_if_all_used_does_not_clear_when_partial() {
        let mut state = RotationState::new();

        // Mark 3 desserts as used
        state.mark_used_dessert("dessert-1");
        state.mark_used_dessert("dessert-2");
        state.mark_used_dessert("dessert-3");

        assert_eq!(state.used_dessert_ids.len(), 3);

        // Call reset with count=5 (not all used yet)
        state.reset_desserts_if_all_used(5);

        // List should NOT be cleared
        assert_eq!(state.used_dessert_ids.len(), 3);
        assert_eq!(state.used_dessert_ids[0], "dessert-1");
        assert_eq!(state.used_dessert_ids[1], "dessert-2");
        assert_eq!(state.used_dessert_ids[2], "dessert-3");
    }

    #[test]
    fn test_reset_desserts_edge_case_available_count_zero() {
        let mut state = RotationState::new();

        // Edge case: no desserts available
        state.reset_desserts_if_all_used(0);

        // Should not panic, list should be cleared (0 >= 0)
        assert_eq!(state.used_dessert_ids.len(), 0);
    }

    // ============================================================================
    // Story 6.5: Edge Case Tests (AC-8, AC-9)
    // ============================================================================

    #[test]
    fn test_mark_used_main_course_with_empty_initial_state() {
        let mut state = RotationState::new();

        // Verify empty initially
        assert_eq!(state.used_main_course_ids.len(), 0);

        // Mark first main course
        state.mark_used_main_course("main-1");

        // Should work correctly
        assert_eq!(state.used_main_course_ids.len(), 1);
        assert!(state.is_main_course_used("main-1"));
    }

    #[test]
    fn test_is_main_course_used_with_empty_list_returns_false() {
        let state = RotationState::new();

        // Empty list should return false for any ID
        assert!(!state.is_main_course_used("any-id"));
        assert!(!state.is_main_course_used(""));
    }

    #[test]
    fn test_reset_appetizers_when_list_is_empty() {
        let mut state = RotationState::new();

        // List is empty
        assert_eq!(state.used_appetizer_ids.len(), 0);

        // Call reset with count > 0
        state.reset_appetizers_if_all_used(5);

        // Should not panic, list remains empty
        assert_eq!(state.used_appetizer_ids.len(), 0);
    }

    #[test]
    fn test_increment_cuisine_usage_when_hashmap_is_empty() {
        use recipe::Cuisine;

        let mut state = RotationState::new();

        // HashMap is empty initially
        assert_eq!(state.cuisine_usage_count.len(), 0);

        // Increment a cuisine
        state.increment_cuisine_usage(&Cuisine::Italian);

        // Should insert correctly with count 1
        assert_eq!(state.get_cuisine_usage(&Cuisine::Italian), 1);
        assert_eq!(state.cuisine_usage_count.len(), 1);
    }

    #[test]
    fn test_all_recipes_exhausted_scenario() {
        let mut state = RotationState::new();

        // Mark all main courses as used
        state.mark_used_main_course("main-1");
        state.mark_used_main_course("main-2");
        state.mark_used_main_course("main-3");

        // Verify all return true for is_used
        assert!(state.is_main_course_used("main-1"));
        assert!(state.is_main_course_used("main-2"));
        assert!(state.is_main_course_used("main-3"));

        // Verify unused recipe still returns false
        assert!(!state.is_main_course_used("main-4"));
    }

    #[test]
    fn test_serde_serialization_round_trip() {
        use recipe::Cuisine;

        // Create state with all fields populated
        let mut state = RotationState::new();
        state.mark_used_main_course("main-1");
        state.mark_used_main_course("main-2");
        state.mark_used_appetizer("app-1");
        state.mark_used_dessert("dessert-1");
        state.increment_cuisine_usage(&Cuisine::Italian);
        state.increment_cuisine_usage(&Cuisine::Indian);
        state.update_last_complex_meal_date("2025-10-27");

        // Serialize to JSON
        let json = state.to_json().expect("Serialization should succeed");

        // Deserialize from JSON
        let deserialized = RotationState::from_json(&json).expect("Deserialization should succeed");

        // Verify equality of all Epic 6 fields
        assert_eq!(
            deserialized.used_main_course_ids,
            state.used_main_course_ids
        );
        assert_eq!(deserialized.used_appetizer_ids, state.used_appetizer_ids);
        assert_eq!(deserialized.used_dessert_ids, state.used_dessert_ids);
        assert_eq!(deserialized.cuisine_usage_count.len(), 2);
        assert_eq!(deserialized.get_cuisine_usage(&Cuisine::Italian), 1);
        assert_eq!(deserialized.get_cuisine_usage(&Cuisine::Indian), 1);
        assert_eq!(
            deserialized.last_complex_meal_date,
            Some("2025-10-27".to_string())
        );
    }

    #[test]
    fn test_bincode_encode_decode_round_trip() {
        use recipe::Cuisine;

        // Create state with all fields populated
        let mut state = RotationState::new();
        state.mark_used_main_course("main-1");
        state.mark_used_appetizer("app-1");
        state.mark_used_dessert("dessert-1");
        state.increment_cuisine_usage(&Cuisine::Mexican);
        state.update_last_complex_meal_date("2025-10-28");

        // Encode with bincode
        let encoded = bincode::encode_to_vec(&state, bincode::config::standard())
            .expect("Bincode encode should succeed");

        // Decode with bincode
        let (decoded, _): (RotationState, usize) =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .expect("Bincode decode should succeed");

        // Verify equality for evento event storage compatibility
        assert_eq!(decoded.used_main_course_ids, state.used_main_course_ids);
        assert_eq!(decoded.used_appetizer_ids, state.used_appetizer_ids);
        assert_eq!(decoded.used_dessert_ids, state.used_dessert_ids);
        assert_eq!(decoded.get_cuisine_usage(&Cuisine::Mexican), 1);
        assert_eq!(
            decoded.last_complex_meal_date,
            Some("2025-10-28".to_string())
        );
    }
}
