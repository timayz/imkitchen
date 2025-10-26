# Story 6.5: Create Rotation State Management Module

Status: Approved

## Story

As a **backend developer**,
I want **to implement rotation state tracking logic**,
so that **recipes are properly rotated across weeks without repetition violations**.

## Acceptance Criteria

1. New file `crates/meal_planning/src/rotation.rs` created
2. RotationState::new() constructor implemented
3. Methods: mark_used_main_course, mark_used_appetizer, mark_used_dessert
4. Method: is_main_course_used (checks uniqueness)
5. Methods: reset_appetizers_if_all_used, reset_desserts_if_all_used
6. Method: increment_cuisine_usage
7. Method: update_last_complex_meal_date
8. Unit tests cover all rotation logic
9. Edge cases handled: empty lists, all recipes exhausted

## Tasks / Subtasks

- [ ] Create rotation.rs module file (AC: 1)
  - [ ] Create `crates/meal_planning/src/rotation.rs`
  - [ ] Add `pub mod rotation;` to `crates/meal_planning/src/lib.rs`
  - [ ] Add module-level documentation explaining rotation rules

- [ ] Implement RotationState struct (AC: 2)
  - [ ] Define struct with fields: used_main_course_ids (Vec<RecipeId>), used_appetizer_ids (Vec<RecipeId>), used_dessert_ids (Vec<RecipeId>), cuisine_usage_count (HashMap<Cuisine, u32>), last_complex_meal_date (Option<Date>)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, Encode, Decode
  - [ ] Implement RotationState::new() constructor initializing all fields to empty/zero
  - [ ] Add unit test for constructor

- [ ] Implement main course tracking methods (AC: 3, 4)
  - [ ] Implement mark_used_main_course(&mut self, id: &RecipeId) - pushes to used_main_course_ids
  - [ ] Implement is_main_course_used(&self, id: &RecipeId) -> bool - checks if ID in used_main_course_ids
  - [ ] Add unit test: mark_used_main_course then verify is_main_course_used returns true
  - [ ] Add unit test: is_main_course_used returns false for unused ID

- [ ] Implement appetizer tracking methods (AC: 3, 5)
  - [ ] Implement mark_used_appetizer(&mut self, id: &RecipeId) - pushes to used_appetizer_ids
  - [ ] Implement reset_appetizers_if_all_used(&mut self, available_count: usize) - clears used_appetizer_ids if len() >= available_count
  - [ ] Add unit test: mark 3 appetizers, call reset with count=3, verify list cleared
  - [ ] Add unit test: mark 2 appetizers, call reset with count=3, verify list NOT cleared
  - [ ] Add edge case test: reset with available_count=0 (should not panic)

- [ ] Implement dessert tracking methods (AC: 3, 5)
  - [ ] Implement mark_used_dessert(&mut self, id: &RecipeId) - pushes to used_dessert_ids
  - [ ] Implement reset_desserts_if_all_used(&mut self, available_count: usize) - clears used_dessert_ids if len() >= available_count
  - [ ] Add unit test: mark 5 desserts, call reset with count=5, verify list cleared
  - [ ] Add unit test: mark 3 desserts, call reset with count=5, verify list NOT cleared
  - [ ] Add edge case test: reset with available_count=0 (should not panic)

- [ ] Implement cuisine variety tracking (AC: 6)
  - [ ] Implement increment_cuisine_usage(&mut self, cuisine: &Cuisine) - increments count in HashMap, inserting 0 if not present
  - [ ] Add unit test: increment same cuisine 3 times, verify count is 3
  - [ ] Add unit test: increment different cuisines, verify separate counts
  - [ ] Add helper method get_cuisine_usage(&self, cuisine: &Cuisine) -> u32 for testing/algorithm use

- [ ] Implement complexity spacing tracking (AC: 7)
  - [ ] Implement update_last_complex_meal_date(&mut self, date: Date) - sets last_complex_meal_date to Some(date)
  - [ ] Add unit test: call update_last_complex_meal_date, verify field updated
  - [ ] Add unit test: verify new() constructor leaves last_complex_meal_date as None

- [ ] Write comprehensive edge case tests (AC: 8, 9)
  - [ ] Test: mark_used_main_course with empty initial state
  - [ ] Test: is_main_course_used with empty list returns false
  - [ ] Test: reset_appetizers_if_all_used when list is empty (should not panic)
  - [ ] Test: increment_cuisine_usage when HashMap is empty (inserts correctly)
  - [ ] Test: all recipes exhausted scenario (mark all main courses, verify all return true for is_used)
  - [ ] Test: serialization round-trip with serde (serialize then deserialize, verify equality)
  - [ ] Test: bincode Encode/Decode round-trip (for evento event storage)

- [ ] Add integration documentation (AC: 1)
  - [ ] Add module documentation explaining: main courses NEVER repeat, appetizers/desserts CAN repeat after exhaustion
  - [ ] Add example usage in module doc showing typical flow: new() -> mark_used -> reset_if_all_used
  - [ ] Document that rotation state is stored in evento events and `meal_plan_rotation_state` table

## Dev Notes

**Architecture Context:**

This module implements core business logic for multi-week meal plan generation, ensuring recipes rotate properly without violating uniqueness constraints.

**Key Business Rules:**
- Main courses MUST be unique across ALL generated weeks (strict uniqueness)
- Appetizers CAN repeat after all available appetizers used once (soft rotation)
- Desserts CAN repeat after all available desserts used once (soft rotation)
- Accompaniments NOT tracked (can repeat freely)
- Cuisine variety tracked to promote diversity
- Complex meals tracked to avoid consecutive high-complexity days

**Technical Constraints:**
- RotationState must be serializable via serde (JSON) and bincode (evento events)
- Used in multi-week algorithm (Epic 7) and stored in database (Story 6.6)
- Must handle edge cases gracefully (empty lists, all recipes exhausted)
- Performance: O(n) lookups acceptable for favorite recipe counts (typically <50 per category)

**Testing Standards:**
- Unit tests for all public methods
- Edge case coverage (empty, exhausted, zero counts)
- Serialization round-trip tests (serde + bincode)
- Test coverage target: >90% per Story 6.7

### Project Structure Notes

**Files to Create/Modify:**
```
crates/meal_planning/src/
├── lib.rs                  # Add `pub mod rotation;`
└── rotation.rs             # NEW - rotation state logic
```

**Dependencies:**
- `serde` (Serialize, Deserialize)
- `bincode` (Encode, Decode)
- `std::collections::HashMap`
- Project types: RecipeId, Cuisine, Date from shared_kernel or meal_planning types

**Alignment with Unified Structure:**
- Follows DDD pattern: rotation logic encapsulated in domain module
- Pure business logic (no database, no HTTP dependencies)
- Evento-compatible serialization (bincode for events)

### References

- [Source: docs/architecture-update-meal-planning-enhancements.md#113-rotationstate] - Struct definition
- [Source: docs/architecture-update-meal-planning-enhancements.md#1055-rotation-state-management] - Method specifications
- [Source: docs/architecture-update-meal-planning-enhancements.md#392-rotation-rules] - Business rules
- [Source: docs/epics.md#story-65-create-rotation-state-management-module] - Story definition
- [Source: docs/solution-architecture-compact.md#469-projection-testing] - Testing with unsafe_oneshot for subscriptions

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-6.5.xml` (Generated: 2025-10-26T00:28:26)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
