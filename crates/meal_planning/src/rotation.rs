use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// RotationState tracks which recipes have been used in the current rotation cycle
///
/// The rotation system ensures each favorite recipe is used exactly once before
/// any recipe repeats. When all favorites have been used once, the cycle resets.
///
/// This state is stored as JSON in the meal_plans.rotation_state column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationState {
    pub cycle_number: u32,
    pub used_recipe_ids: HashSet<String>,
}

impl RotationState {
    /// Create a new rotation state starting at cycle 1
    pub fn new() -> Self {
        RotationState {
            cycle_number: 1,
            used_recipe_ids: HashSet::new(),
        }
    }

    /// Mark a recipe as used in the current cycle
    pub fn mark_recipe_used(&mut self, recipe_id: String) {
        self.used_recipe_ids.insert(recipe_id);
    }

    /// Check if a recipe has been used in the current cycle
    pub fn is_recipe_used(&self, recipe_id: &str) -> bool {
        self.used_recipe_ids.contains(recipe_id)
    }

    /// Reset the cycle when all favorites have been used once
    ///
    /// This allows recipes to be reused in subsequent meal plans.
    pub fn reset_cycle(&mut self) {
        self.cycle_number += 1;
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
}
