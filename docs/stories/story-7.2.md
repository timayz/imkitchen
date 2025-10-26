# Story 7.2: Main Course Selection with Preferences

Status: Approved

## Story

As a **meal planning algorithm**,
I want to **select main courses based on user preferences and constraints**,
so that **meal assignments match user availability, skill level, and variety preferences**.

## Acceptance Criteria

1. Function `select_main_course_with_preferences` implemented
2. Filters by `max_prep_time` (weeknight vs weekend)
3. Filters by `skill_level` (Beginner→Simple, Intermediate→Simple+Moderate, Advanced→All)
4. Filters by `avoid_consecutive_complex` (checks `rotation_state.last_complex_meal_date`)
5. Scores by `cuisine_variety_weight` (penalizes recent cuisines per formula)
6. Returns highest-scored recipe
7. Handles no compatible recipes gracefully (returns `None`)
8. Unit tests cover preference combinations
9. Performance: <10ms for 100 recipes

## Tasks / Subtasks

- [ ] Implement main course selection function (AC: 1)
  - [ ] Create function in `crates/meal_planning/src/algorithm.rs`
  - [ ] Signature: `pub fn select_main_course_with_preferences(available_main_courses: &[Recipe], preferences: &UserPreferences, rotation_state: &RotationState, date: Date, day_of_week: DayOfWeek) -> Option<Recipe>`
  - [ ] Return `Option<Recipe>` (None if no compatible recipes)

- [ ] Implement time constraint filtering (AC: 2)
  - [ ] Determine if date is weeknight (Mon-Fri) or weekend (Sat-Sun)
  - [ ] Weeknight: filter recipes where `prep_time_minutes + cook_time_minutes <= preferences.max_prep_time_weeknight`
  - [ ] Weekend: filter recipes where `prep_time_minutes + cook_time_minutes <= preferences.max_prep_time_weekend`
  - [ ] Default weeknight: 30 minutes, weekend: 90 minutes

- [ ] Implement skill level filtering (AC: 3)
  - [ ] Beginner: only Simple complexity recipes
  - [ ] Intermediate: Simple + Moderate complexity recipes
  - [ ] Advanced: all complexity levels (Simple, Moderate, Complex)
  - [ ] Filter based on `recipe.complexity` field

- [ ] Implement consecutive complex avoidance (AC: 4)
  - [ ] Check `preferences.avoid_consecutive_complex` flag
  - [ ] If true and `rotation_state.last_complex_meal_date` is yesterday, filter out Complex recipes
  - [ ] Allow Complex recipes if last complex was 2+ days ago or None

- [ ] Implement cuisine variety scoring (AC: 5)
  - [ ] Calculate score: `variety_weight * (1.0 / (cuisine_usage_count[recipe.cuisine] + 1.0))`
  - [ ] Use `rotation_state.get_cuisine_usage(cuisine)` for usage count
  - [ ] `variety_weight` = `preferences.cuisine_variety_weight` (0.0-1.0, default 0.7)
  - [ ] Higher score = more diverse (less-used cuisine)

- [ ] Select highest-scored recipe (AC: 6)
  - [ ] After filtering, score all remaining recipes
  - [ ] Return recipe with highest cuisine variety score
  - [ ] If multiple recipes tie, select first one (deterministic)

- [ ] Handle no compatible recipes (AC: 7)
  - [ ] If all recipes filtered out, return `None`
  - [ ] Do not panic or error, allow caller to handle gracefully

- [ ] Write comprehensive unit tests (AC: 8)
  - [ ] Test weeknight time filtering (30min limit)
  - [ ] Test weekend time filtering (90min limit)
  - [ ] Test skill level filtering (Beginner, Intermediate, Advanced)
  - [ ] Test consecutive complex avoidance (yesterday vs 2 days ago)
  - [ ] Test cuisine variety scoring formula
  - [ ] Test highest-scored selection
  - [ ] Test no compatible recipes returns None
  - [ ] Test preference combination scenarios

- [ ] Performance benchmark (AC: 9)
  - [ ] Create benchmark in `benches/algorithm_benchmarks.rs`
  - [ ] Measure selection time with 100 candidate recipes
  - [ ] Assert <10ms execution time (P95)

## Dev Notes

### Architecture Patterns

**Function Design:**
- Pure function: no side effects, deterministic given same inputs
- Stateless: all state passed via parameters (RotationState)
- Filter-then-score pattern: narrow candidates, then optimize

**Multi-Factor Decision Algorithm:**
1. **Hard Constraints (Filters):** Time, skill, consecutive complex - must satisfy ALL
2. **Soft Preferences (Scoring):** Cuisine variety - optimize for best match
3. **Selection:** Highest score wins

**Cuisine Variety Scoring Formula:**
```
score = variety_weight * (1.0 / (usage_count + 1.0))

Examples with variety_weight=0.7:
- Italian used 0 times: 0.7 * (1/1) = 0.70
- Italian used 1 time:  0.7 * (1/2) = 0.35
- Italian used 2 times: 0.7 * (1/3) = 0.23

Interpretation:
- variety_weight=0.0: no penalty (repeat cuisines freely)
- variety_weight=1.0: maximum penalty (avoid repeats)
- variety_weight=0.7: balanced variety (default)
```

**Weekday Determination:**
- Monday-Friday: weeknight constraints
- Saturday-Sunday: weekend constraints
- Use `chrono::Datelike::weekday()` to get DayOfWeek

### Project Structure Notes

**File Location:**
- `crates/meal_planning/src/algorithm.rs` - Main algorithm functions
- `crates/meal_planning/src/rotation.rs` - RotationState (Epic 6 Story 6.5)

**Data Model Dependencies:**
```rust
pub struct UserPreferences {
    dietary_restrictions: Vec<DietaryRestriction>,
    max_prep_time_weeknight: u32,  // minutes, default 30
    max_prep_time_weekend: u32,     // minutes, default 90
    skill_level: SkillLevel,        // Beginner | Intermediate | Advanced
    avoid_consecutive_complex: bool, // default true
    cuisine_variety_weight: f32,    // 0.0-1.0, default 0.7
}

pub enum SkillLevel {
    Beginner,      // Only Simple recipes
    Intermediate,  // Simple + Moderate
    Advanced,      // All complexity levels
}

pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

pub struct Recipe {
    id: RecipeId,
    prep_time_minutes: u32,
    cook_time_minutes: u32,
    complexity: Complexity,
    cuisine: Cuisine,
    // ... other fields
}
```

**RotationState Methods Used:**
- `rotation_state.get_cuisine_usage(&cuisine) -> u32`
- `rotation_state.get_last_complex_meal_date() -> Option<Date>`

### Testing Standards

**TDD Approach:**
1. Write failing test for each AC
2. Implement minimal code to pass
3. Refactor for performance/clarity

**Test Coverage:**
- Unit tests for each filtering step
- Scoring formula validation
- Edge cases: empty candidates, all filtered, ties
- Integration with RotationState (mock/test state)

**Performance Test:**
- Criterion benchmark with realistic data
- 100 recipes, varied preferences
- Assert P95 < 10ms

### References

- [Tech Spec: Section 3.2 - Main Course Selection](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.2](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Tech Spec: Cuisine Variety Scoring Formula](../tech-spec-epic-7.md#workflows-and-sequencing)
- [Domain Models: UserPreferences struct](../tech-spec-epic-7.md#data-models-and-contracts)
- [Performance: <10ms target](../tech-spec-epic-7.md#performance)
- [Epic 6 Story 6.5: RotationState](./story-6.5.md)

## Dev Agent Record

### Context Reference

<!-- Story context XML will be added by story-context workflow -->

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
