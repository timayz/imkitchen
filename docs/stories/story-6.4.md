# Story 6.4: Update User Domain Model

Status: Approved

## Story

As a **backend developer**,
I want **to extend User aggregate with meal planning preferences**,
so that **the algorithm can personalize meal plans based on user constraints and preferences**.

## Acceptance Criteria

1. UserPreferences struct created with meal planning fields
2. Fields: max_prep_time_weeknight, max_prep_time_weekend, avoid_consecutive_complex, cuisine_variety_weight, dietary_restrictions
3. DietaryRestriction enum created (Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher, Custom(String))
4. SkillLevel enum created (Beginner, Intermediate, Advanced)
5. UserMealPlanningPreferencesUpdated event created
6. User aggregate integrates preferences
7. Unit tests cover preferences event handling
8. Default values per design decisions (max_prep_time_weeknight: 30, weekend: 90, cuisine_variety_weight: 0.7)

## Tasks / Subtasks

- [ ] Create DietaryRestriction enum (AC: 3)
  - [ ] Create enum in `crates/user/src/lib.rs` or new `types.rs` module
  - [ ] Add variants: Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher, Custom(String)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode
  - [ ] Add unit test for serialization round-trip with Custom variant

- [ ] Create SkillLevel enum (AC: 4)
  - [ ] Create enum in same module as DietaryRestriction
  - [ ] Add variants: Beginner, Intermediate, Advanced
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode
  - [ ] Add unit test for serialization round-trip

- [ ] Create UserPreferences struct (AC: 1, 2)
  - [ ] Add field: user_id (UserId)
  - [ ] Add field: dietary_restrictions (Vec<DietaryRestriction>)
  - [ ] Add field: household_size (u32)
  - [ ] Add field: skill_level (SkillLevel)
  - [ ] Add field: weeknight_availability (TimeRange) - reuse existing type
  - [ ] Add field: max_prep_time_weeknight (u32) - minutes, default 30
  - [ ] Add field: max_prep_time_weekend (u32) - minutes, default 90
  - [ ] Add field: avoid_consecutive_complex (bool) - default true
  - [ ] Add field: cuisine_variety_weight (f32) - 0.0-1.0, default 0.7
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, Encode, Decode
  - [ ] Implement Default trait with specified default values

- [ ] Create UserMealPlanningPreferencesUpdated event (AC: 5)
  - [ ] Create event struct with all preference fields
  - [ ] Add evento::AggregatorName derive
  - [ ] Add bincode::Encode and bincode::Decode derives
  - [ ] Add serde::Serialize and serde::Deserialize derives
  - [ ] Add unit test for event serialization/deserialization

- [ ] Update User aggregate to integrate preferences (AC: 6)
  - [ ] Add preferences: UserPreferences field to User struct
  - [ ] Implement event handler for UserMealPlanningPreferencesUpdated
  - [ ] Update User::new() to initialize with default preferences
  - [ ] Add apply_event method for preference updates
  - [ ] Ensure preferences field properly serialized with evento

- [ ] Create unit tests for preference event handling (AC: 7)
  - [ ] Test: Create user with default preferences
  - [ ] Test: Update preferences via event - verify all fields updated
  - [ ] Test: Multiple preference updates preserve user_id
  - [ ] Test: Custom dietary restriction serialization
  - [ ] Test: Skill level affects preference constraints
  - [ ] Test: Default values match specification (30, 90, 0.7)
  - [ ] Use `unsafe_oneshot` for subscribe in tests (sync event processing)

- [ ] Validate defaults and constraints (AC: 8)
  - [ ] Test: max_prep_time_weeknight defaults to 30 minutes
  - [ ] Test: max_prep_time_weekend defaults to 90 minutes
  - [ ] Test: cuisine_variety_weight defaults to 0.7
  - [ ] Test: avoid_consecutive_complex defaults to true
  - [ ] Test: cuisine_variety_weight validated in range 0.0-1.0
  - [ ] Test: max_prep_time values are positive integers

## Dev Notes

### Domain Model Architecture

**Location:** `crates/user/src/`

**Files to modify/create:**
- `crates/user/src/lib.rs` - User aggregate
- `crates/user/src/types.rs` (new) - DietaryRestriction, SkillLevel enums
- `crates/user/src/events.rs` (or inline in lib.rs) - UserMealPlanningPreferencesUpdated event

**Evento Integration:**
- User aggregate already exists with evento support (from Epic 1)
- Add new event to existing event handler pattern
- Ensure bincode + serde derives for evento compatibility
- All preference fields must be serializable

### Design Decisions from Architecture Doc

**Cuisine Preferences (NOT stored):**
- Cuisine preferences are **inferred** from favorite recipe selection
- No explicit `preferred_cuisines` field needed
- `cuisine_variety_weight` slider controls variety spreading vs. natural repetition
- [Source: docs/architecture-update-meal-planning-enhancements.md#3.2]

**Advance Prep Timing (NOT stored):**
- Advance prep requirements are **recipe characteristics** (stored on Recipe)
- Stored in recipe's `advance_prep_text` field
- Algorithm schedules meals and sends prep reminders based on recipe requirements
- No user preference needed for advance prep willingness
- [Source: docs/architecture-update-meal-planning-enhancements.md#3.2]

**Default Values Rationale:**
- `max_prep_time_weeknight: 30` - Typical weeknight cooking window
- `max_prep_time_weekend: 90` - More flexible weekend schedule
- `cuisine_variety_weight: 0.7` - Balanced variety (70% variety preference)
- `avoid_consecutive_complex: true` - Prevent burnout from complex meals back-to-back
- [Source: docs/architecture-update-meal-planning-enhancements.md#3.4]

### Testing Strategy

**Unit Test Coverage:**
- All new enums: serialization round-trip tests
- UserPreferences: Default trait implementation
- User aggregate: Event handler integration
- Preference updates: Field-level validation
- Use `unsafe_oneshot` instead of `run` for subscribe in tests (sync processing)
- Target >90% coverage per Epic 6 requirements

**Test Organization:**
- Tests inline with modules or in `crates/user/tests/`
- Integration tests use evento test executor
- Mock/fixture patterns from existing User tests

### Project Structure Notes

**Alignment with existing structure:**
- Follows DDD crate structure: `crates/user/` bounded context
- Consistent with Recipe (6.2) and MealPlan (6.3) domain updates
- Matches evento pattern established in Epic 1
- Naming: `snake_case` for modules, `PascalCase` for structs/enums

**No conflicts detected:**
- UserPreferences is new struct, no naming collisions
- DietaryRestriction/SkillLevel are domain-specific enums
- Integrates cleanly with existing User aggregate

### References

- [Source: docs/epics.md#Story 6.4] - Story requirements
- [Source: docs/architecture-update-meal-planning-enhancements.md#3.3] - UserPreferences struct definition
- [Source: docs/architecture-update-meal-planning-enhancements.md#3.4] - Default values and events
- [Source: docs/solution-architecture-compact.md#13] - TDD strategy, unsafe_oneshot for tests
- [Source: docs/stories/story-6.3.md] - Pattern reference for domain model updates

## Change Log

| Date | Author | Changes |
|------|--------|---------|
| 2025-10-25 | Bob (SM Agent) | Initial story creation from epics.md 6.4 |

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-6.4.xml`

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
