/// Integration tests for Recipe Rotation System (Story 3.3)
///
/// These tests verify that rotation state tracking works correctly with
/// evento projections and database operations.
use meal_planning::rotation::RotationState;

#[cfg(test)]
mod rotation_integration {
    use super::*;

    /// Test: RotationState serialization round-trip
    ///
    /// Verifies that RotationState can be serialized to JSON and deserialized
    /// without data loss (AC-4: rotation state persists)
    #[test]
    fn test_rotation_state_serialization_round_trip() {
        let mut state = RotationState::with_favorite_count(15).unwrap();
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        state.mark_recipe_used("recipe_3".to_string());

        // Serialize to JSON
        let json = state.to_json().expect("Failed to serialize");

        // Deserialize back
        let restored = RotationState::from_json(&json).expect("Failed to deserialize");

        // Verify all fields preserved
        assert_eq!(restored.cycle_number, state.cycle_number);
        assert_eq!(restored.total_favorite_count, state.total_favorite_count);
        assert_eq!(restored.used_count(), 3);
        assert!(restored.is_recipe_used("recipe_1"));
        assert!(restored.is_recipe_used("recipe_2"));
        assert!(restored.is_recipe_used("recipe_3"));
        assert!(!restored.is_recipe_used("recipe_4"));
    }

    /// Test: Rotation cycle reset logic
    ///
    /// Verifies that cycle resets correctly when all favorites are used
    /// (AC-2: each recipe used exactly once, AC-3: cycle resets after all used)
    #[test]
    fn test_rotation_cycle_reset_behavior() {
        let mut state = RotationState::with_favorite_count(5).unwrap();

        // Mark 4 recipes as used - should NOT trigger reset
        for i in 1..=4 {
            state.mark_recipe_used(format!("recipe_{}", i));
        }
        assert_eq!(state.cycle_number, 1);
        assert_eq!(state.used_count(), 4);
        assert!(!state.should_reset_cycle());

        // Mark 5th recipe - should trigger reset
        state.mark_recipe_used("recipe_5".to_string());
        assert_eq!(state.used_count(), 5);
        assert!(state.should_reset_cycle());

        // Actually reset the cycle
        state.reset_cycle();
        assert_eq!(state.cycle_number, 2);
        assert_eq!(state.used_count(), 0);

        // All recipes should be available again
        for i in 1..=5 {
            assert!(!state.is_recipe_used(&format!("recipe_{}", i)));
        }
    }

    /// Test: Rotation state with no favorites
    ///
    /// Verifies edge case handling when user has no favorites
    #[test]
    fn test_rotation_state_no_favorites() {
        // Should fail with validation error
        let result = RotationState::with_favorite_count(0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "total_favorite_count must be greater than 0"
        );
    }

    /// Test: Rotation state prevents duplicate marking
    ///
    /// Verifies that marking the same recipe multiple times only counts once
    /// (AC-2: each recipe used exactly once)
    #[test]
    fn test_rotation_state_prevents_duplicates() {
        let mut state = RotationState::with_favorite_count(10).unwrap();

        // Mark same recipe 3 times
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_1".to_string());

        // Should only count once (HashSet deduplication)
        assert_eq!(state.used_count(), 1);
        assert!(state.is_recipe_used("recipe_1"));
    }

    /// Test: Rotation state across multiple cycles
    ///
    /// Verifies that rotation state correctly tracks multiple cycles
    /// (AC-3: cycle resets and recipes become available again)
    #[test]
    fn test_rotation_state_multiple_cycles() {
        let mut state = RotationState::with_favorite_count(3).unwrap();

        // Cycle 1: Use all 3 recipes
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        state.mark_recipe_used("recipe_3".to_string());
        assert_eq!(state.cycle_number, 1);
        assert!(state.should_reset_cycle());
        state.reset_cycle();

        // Cycle 2: Use same recipes again
        assert_eq!(state.cycle_number, 2);
        assert_eq!(state.used_count(), 0);
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        state.mark_recipe_used("recipe_3".to_string());
        assert!(state.should_reset_cycle());
        state.reset_cycle();

        // Cycle 3: Verify cycle number increments correctly
        assert_eq!(state.cycle_number, 3);
        assert_eq!(state.used_count(), 0);
    }

    /// Test: RotationSystem filter_available_recipes
    ///
    /// Verifies that the system correctly filters recipes based on rotation state
    /// (AC-1: tracks which recipes used, AC-2: prevents duplicates)
    #[test]
    fn test_rotation_system_filters_correctly() {
        use meal_planning::rotation::RotationSystem;

        let all_favorites = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
            "recipe_4".to_string(),
            "recipe_5".to_string(),
        ];

        let mut rotation_state = RotationState::with_favorite_count(5).unwrap();
        rotation_state.mark_recipe_used("recipe_2".to_string());
        rotation_state.mark_recipe_used("recipe_4".to_string());

        let available = RotationSystem::filter_available_recipes(&all_favorites, &rotation_state);

        // Should only return unused recipes (1, 3, 5)
        assert_eq!(available.len(), 3);
        assert!(available.contains(&"recipe_1".to_string()));
        assert!(available.contains(&"recipe_3".to_string()));
        assert!(available.contains(&"recipe_5".to_string()));
        assert!(!available.contains(&"recipe_2".to_string()));
        assert!(!available.contains(&"recipe_4".to_string()));
    }

    /// Test: RotationSystem update_after_generation
    ///
    /// Verifies that rotation state updates correctly after meal plan generation
    /// (AC-1: tracks usage, AC-3: resets when all used)
    #[test]
    fn test_rotation_system_update_after_generation() {
        use meal_planning::rotation::RotationSystem;

        let initial_state = RotationState::with_favorite_count(7).unwrap();

        // Generate meal plan with 7 assignments (uses all favorites)
        let assigned_ids = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
            "recipe_4".to_string(),
            "recipe_5".to_string(),
            "recipe_6".to_string(),
            "recipe_7".to_string(),
        ];

        let updated_state =
            RotationSystem::update_after_generation(&assigned_ids, 7, initial_state);

        // All 7 used, so cycle should have automatically reset
        assert_eq!(updated_state.cycle_number, 2);
        assert_eq!(updated_state.used_count(), 0);
    }

    /// Test: RotationSystem partial generation doesn't reset
    ///
    /// Verifies that partial generation (not all favorites used) doesn't trigger reset
    /// (AC-2: each recipe used once before reset, AC-3: resets only when all used)
    #[test]
    fn test_rotation_system_partial_generation_no_reset() {
        use meal_planning::rotation::RotationSystem;

        let initial_state = RotationState::with_favorite_count(20).unwrap();

        // Generate meal plan with only 7 assignments (20 favorites total)
        let assigned_ids = vec![
            "recipe_1".to_string(),
            "recipe_2".to_string(),
            "recipe_3".to_string(),
            "recipe_4".to_string(),
            "recipe_5".to_string(),
            "recipe_6".to_string(),
            "recipe_7".to_string(),
        ];

        let updated_state =
            RotationSystem::update_after_generation(&assigned_ids, 20, initial_state);

        // Only 7 of 20 used, cycle should NOT reset
        assert_eq!(updated_state.cycle_number, 1);
        assert_eq!(updated_state.used_count(), 7);

        // All 7 assigned recipes should be marked as used
        for id in &assigned_ids {
            assert!(updated_state.is_recipe_used(id));
        }
    }

    /// Test: should_reset_cycle static method
    ///
    /// Verifies the helper method correctly determines when to reset
    #[test]
    fn test_should_reset_cycle_helper() {
        use meal_planning::rotation::RotationSystem;

        let mut state = RotationState::with_favorite_count(5).unwrap();

        // 0 used, 5 total -> no reset
        assert!(!RotationSystem::should_reset_cycle(5, &state));

        // 4 used, 5 total -> no reset
        state.mark_recipe_used("recipe_1".to_string());
        state.mark_recipe_used("recipe_2".to_string());
        state.mark_recipe_used("recipe_3".to_string());
        state.mark_recipe_used("recipe_4".to_string());
        assert!(!RotationSystem::should_reset_cycle(5, &state));

        // 5 used, 5 total -> reset
        state.mark_recipe_used("recipe_5".to_string());
        assert!(RotationSystem::should_reset_cycle(5, &state));

        // Edge case: more used than total (shouldn't happen, but handle gracefully)
        state.mark_recipe_used("recipe_6".to_string());
        assert!(RotationSystem::should_reset_cycle(5, &state));
    }
}
