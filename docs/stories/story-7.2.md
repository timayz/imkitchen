# Story 7.2: Main Course Selection with Preferences

Status: Done

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

- [x] Implement main course selection function (AC: 1)
  - [x] Create function in `crates/meal_planning/src/algorithm.rs`
  - [x] Signature: `pub fn select_main_course_with_preferences(available_main_courses: &[Recipe], preferences: &UserPreferences, rotation_state: &RotationState, date: Date, day_of_week: DayOfWeek) -> Option<Recipe>`
  - [x] Return `Option<Recipe>` (None if no compatible recipes)

- [x] Implement time constraint filtering (AC: 2)
  - [x] Determine if date is weeknight (Mon-Fri) or weekend (Sat-Sun)
  - [x] Weeknight: filter recipes where `prep_time_minutes + cook_time_minutes <= preferences.max_prep_time_weeknight`
  - [x] Weekend: filter recipes where `prep_time_minutes + cook_time_minutes <= preferences.max_prep_time_weekend`
  - [x] Default weeknight: 30 minutes, weekend: 90 minutes

- [x] Implement skill level filtering (AC: 3)
  - [x] Beginner: only Simple complexity recipes
  - [x] Intermediate: Simple + Moderate complexity recipes
  - [x] Advanced: all complexity levels (Simple, Moderate, Complex)
  - [x] Filter based on `recipe.complexity` field

- [x] Implement consecutive complex avoidance (AC: 4)
  - [x] Check `preferences.avoid_consecutive_complex` flag
  - [x] If true and `rotation_state.last_complex_meal_date` is yesterday, filter out Complex recipes
  - [x] Allow Complex recipes if last complex was 2+ days ago or None

- [x] Implement cuisine variety scoring (AC: 5)
  - [x] Calculate score: `variety_weight * (1.0 / (cuisine_usage_count[recipe.cuisine] + 1.0))`
  - [x] Use `rotation_state.get_cuisine_usage(cuisine)` for usage count
  - [x] `variety_weight` = `preferences.cuisine_variety_weight` (0.0-1.0, default 0.7)
  - [x] Higher score = more diverse (less-used cuisine)

- [x] Select highest-scored recipe (AC: 6)
  - [x] After filtering, score all remaining recipes
  - [x] Return recipe with highest cuisine variety score
  - [x] If multiple recipes tie, select first one (deterministic)

- [x] Handle no compatible recipes (AC: 7)
  - [x] If all recipes filtered out, return `None`
  - [x] Do not panic or error, allow caller to handle gracefully

- [x] Write comprehensive unit tests (AC: 8)
  - [x] Test weeknight time filtering (30min limit)
  - [x] Test weekend time filtering (90min limit)
  - [x] Test skill level filtering (Beginner, Intermediate, Advanced)
  - [x] Test consecutive complex avoidance (yesterday vs 2 days ago)
  - [x] Test cuisine variety scoring formula
  - [x] Test highest-scored selection
  - [x] Test no compatible recipes returns None
  - [x] Test preference combination scenarios

- [x] Performance benchmark (AC: 9)
  - [x] Create benchmark in `benches/algorithm_benchmarks.rs`
  - [x] Measure selection time with 100 candidate recipes
  - [x] Assert <10ms execution time (P95)

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

/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-7.2.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

**Implementation Plan:**
1. Added UserPreferences and SkillLevel types to algorithm.rs:181-215
2. Implemented select_main_course_with_preferences function:535-647 with multi-stage filtering:
   - Time constraint filtering (weeknight vs weekend)
   - Skill level filtering (Beginner/Intermediate/Advanced)
   - Consecutive complex avoidance (checks last_complex_meal_date)
   - Cuisine variety scoring using formula: variety_weight * (1.0 / (usage_count + 1.0))
3. Added cuisine field to RecipeForPlanning struct:21
4. Wrote 12 comprehensive unit tests covering all ACs:1122-1508
5. Created performance benchmark in benches/algorithm_benchmarks.rs

**Test Results:**
- All 84 tests in meal_planning crate pass
- 12 new tests specifically for Story 7.2, covering:
  - Weeknight/weekend time filtering
  - Skill level filtering (Beginner, Intermediate, Advanced)
  - Consecutive complex avoidance (yesterday vs 2+ days ago)
  - Cuisine variety scoring formula validation
  - Highest-scored selection and tie-breaking
  - No compatible recipes handling
  - Multi-constraint combination scenarios

**Performance Results:**
- Benchmark: ~715 nanoseconds (~0.0007ms) for 100 recipes
- Target was <10ms (P95), achieved **~14,000x faster** than requirement
- O(n) complexity as designed, uses efficient Vec filtering

### Completion Notes List

**Implementation completed successfully - all ACs satisfied:**

- AC-1: Function `select_main_course_with_preferences` implemented at algorithm.rs:558
- AC-2: Time filtering by weeknight (30min default) vs weekend (90min default) - lines 565-571
- AC-3: Skill level filtering (Beginner→Simple, Intermediate→Simple+Moderate, Advanced→All) - lines 587-597
- AC-4: Consecutive complex avoidance checks last_complex_meal_date via rotation_state - lines 599-619
- AC-5: Cuisine variety scoring using formula with rotation_state.get_cuisine_usage() - lines 627-638
- AC-6: Returns highest-scored recipe, deterministic tie-breaking - lines 640-646
- AC-7: Returns None gracefully when all recipes filtered - lines 622-625
- AC-8: 12 comprehensive unit tests cover all preference combinations - lines 1122-1508
- AC-9: Performance benchmark shows ~0.0007ms (far below 10ms target) - benches/algorithm_benchmarks.rs

**Architecture Notes:**
- Pure function design: no side effects, deterministic
- Filter-then-score pattern: hard constraints first, then optimization
- Leverages existing RecipeComplexityCalculator and RotationState APIs
- Added Cuisine field to RecipeForPlanning (breaking change for existing tests, all updated)

### File List

- `crates/meal_planning/src/algorithm.rs` - Main implementation (types + function + tests)
- `crates/meal_planning/benches/algorithm_benchmarks.rs` - Performance benchmark (new file)
- `crates/meal_planning/Cargo.toml` - Added criterion dependency and bench config
- `crates/meal_planning/src/constraints.rs` - Updated test helper to include cuisine
- `crates/meal_planning/src/dietary_filter.rs` - Updated test helper to include cuisine
- `crates/meal_planning/src/lib.rs` - Updated test helper to include cuisine

### Change Log

**2025-10-26 - Story 7.2 Implementation:**
- Added UserPreferences struct with time limits, skill level, complexity avoidance, and cuisine variety weight
- Added SkillLevel enum (Beginner, Intermediate, Advanced)
- Implemented select_main_course_with_preferences function with multi-factor filtering and scoring
- Added cuisine: Cuisine field to RecipeForPlanning struct
- Created 12 unit tests covering all acceptance criteria
- Created criterion-based performance benchmark exceeding requirements by 14,000x
- All tests passing (84/84 in meal_planning crate)

**2025-10-26 - Senior Developer Review notes appended**

---

## Senior Developer Review (AI)

**Reviewer:** Jonathan
**Date:** 2025-10-26
**Outcome:** ✅ **Approve**

### Summary

Story 7.2 implementation is **production-ready** and exceeds all acceptance criteria. The `select_main_course_with_preferences` function demonstrates excellent software engineering practices:

- **Clean architecture**: Pure function design with no side effects
- **Performance excellence**: 715ns execution time (~14,000x faster than 10ms target)
- **Comprehensive testing**: 12 unit tests with 100% AC coverage
- **Code quality**: Well-documented, idiomatic Rust with clear separation of concerns
- **Security**: No injection risks, proper error handling via Option types

The implementation follows the filter-then-score pattern precisely as specified in the tech spec, leveraging existing `RotationState` and `RecipeComplexityCalculator` APIs without introducing unnecessary coupling.

### Key Findings

**Strengths (Notable):**

1. **Exceptional performance** - Benchmark shows 715ns for 100 recipes, demonstrating O(n) complexity is well-optimized
2. **Defensive programming** - Uses `unwrap_or(0)` for optional times, handles None cases gracefully throughout
3. **Type safety** - Leverages Rust's type system (Option, enums) to eliminate null pointer risks
4. **Test quality** - Tests are deterministic, cover edge cases (yesterday vs 2+ days, tie-breaking), and validate formulas
5. **Documentation** - Inline comments map directly to AC numbers, making traceability trivial

**Medium Priority Observations:**

1. **[Med] Float comparison in scoring** (algorithm.rs:642)
   - Uses `partial_cmp` with `unwrap_or(Ordering::Equal)` for f32 sorting
   - While safe, could document NaN handling expectations
   - Current approach is reasonable given cuisine scores are always valid (0.0-1.0 range)

2. **[Low] Cuisine field addition is breaking change**
   - RecipeForPlanning now requires cuisine field
   - All test helpers updated correctly
   - Consider migration strategy for existing data (if applicable)

### Acceptance Criteria Coverage

| AC | Requirement | Status | Evidence |
|----|-------------|--------|----------|
| AC-1 | Function implemented | ✅ | algorithm.rs:558 |
| AC-2 | Time filtering (weeknight/weekend) | ✅ | Lines 565-586, tests 1122-1172 |
| AC-3 | Skill level filtering | ✅ | Lines 587-597, tests 1175-1255 |
| AC-4 | Consecutive complex avoidance | ✅ | Lines 599-619, tests 1258-1319 |
| AC-5 | Cuisine variety scoring | ✅ | Lines 627-638, tests 1322-1384 |
| AC-6 | Highest-scored selection | ✅ | Lines 640-646, tests 1387-1443 |
| AC-7 | Graceful None handling | ✅ | Lines 622-625, tests 1446-1469 |
| AC-8 | Comprehensive tests | ✅ | 12 tests, lines 1122-1508 |
| AC-9 | Performance <10ms | ✅ | Benchmark: 0.0007ms (benches/algorithm_benchmarks.rs) |

**Result:** 9/9 ACs fully satisfied ✅

### Test Coverage and Gaps

**Coverage Analysis:**

- **Unit tests**: 12 tests specifically for Story 7.2 (excellent)
- **Edge cases**: Yesterday vs 2+ days ago, empty candidates, ties, variety_weight=0
- **Integration**: Uses real `RotationState` and `RecipeComplexityCalculator` (not mocked)
- **Performance**: Criterion benchmark with 100 recipes

**No gaps identified.** Test suite is comprehensive and follows TDD principles outlined in dev notes.

**Test Quality Observations:**

✅ Deterministic (fixed dates, no flakiness)
✅ Isolated (each test creates own fixtures)
✅ Readable (descriptive names, clear assertions)
✅ Fast (synchronous, no I/O)

### Architectural Alignment

**Tech Spec Compliance:**

✅ **Pure function design** (constraint #1) - No side effects, deterministic
✅ **Stateless** (constraint #2) - State passed via RotationState parameter
✅ **Filter-then-score pattern** (constraint #3) - Lines 580-646 follow exact pattern
✅ **Performance target** (constraint #6) - 715ns << 10ms requirement
✅ **No external dependencies** (constraint #8) - Pure domain logic

**Architecture Patterns:**

- Leverages existing `recipe::Cuisine` enum from recipe crate (good reuse)
- Extends `RecipeForPlanning` struct cleanly with `cuisine` field
- No new dependencies introduced (uses existing chrono, recipe crates)
- Follows Rust idioms: iterators, pattern matching, Option types

**Layering:**

- Domain logic in `meal_planning::algorithm` ✓
- No business logic in tests ✓
- Clear separation: filtering (hard constraints) → scoring (soft preferences) → selection

### Security Notes

**No security concerns identified.**

✅ No user input validation needed (function operates on typed structs)
✅ No injection risks (pure in-memory algorithm)
✅ No secret handling
✅ No network I/O
✅ No unsafe code blocks
✅ Integer overflow protected (`unwrap_or(0)` prevents None panics)

**Date parsing** (line 577): Uses `parse_from_str` with `and_then` - gracefully handles invalid dates by returning None, which is safe (allows complex recipes if date unparseable).

### Best-Practices and References

**Rust Best Practices Applied:**

1. ✅ **Error handling**: Uses `Option<T>` instead of panicking ([Rust Book Ch 9](https://doc.rust-lang.org/book/ch09-00-error-handling.html))
2. ✅ **Iterator patterns**: Chained filters instead of explicit loops (idiomatic Rust)
3. ✅ **Borrowing**: Takes `&[RecipeForPlanning]` slice, returns owned `RecipeForPlanning` (clear ownership)
4. ✅ **Documentation**: Rustdoc comments with examples, parameter descriptions
5. ✅ **Performance**: Zero-copy filtering with references, clone only final result

**Testing Best Practices:**

1. ✅ Uses Criterion for microbenchmarks ([Criterion docs](https://github.com/bheisler/criterion.rs))
2. ✅ Tests use `black_box` to prevent compiler optimizations from skewing results
3. ✅ Test helper functions avoid duplication

**Observed Patterns:**

- **Multi-factor decision algorithms**: Classic CSP (Constraint Satisfaction Problem) approach - hard constraints filter, soft preferences score
- **Defensive defaults**: `unwrap_or(0)` for times, `unwrap_or(Ordering::Equal)` for float comparison
- **Formula validation in tests**: Tests explicitly check scoring formula (e.g., 0.7 * 1/3 = 0.23)

### Action Items

**No blocking or high-priority action items.**

**Optional Enhancements (Low Priority):**

1. **[Low][Enhancement]** Consider adding a doc comment on line 642 explaining NaN handling in `partial_cmp`, though current approach is safe
   - File: `crates/meal_planning/src/algorithm.rs:642`
   - Context: `scored_candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));`
   - Suggestion: Add comment: `// NaN-safe: cuisine scores always in [0.0, 1.0] range`

2. **[Low][Documentation]** Add migration note for `cuisine` field if existing production data needs updating
   - File: Story 7.2 or tech spec
   - Context: RecipeForPlanning now requires cuisine field
   - Suggestion: Document data migration strategy if needed

### Verdict

**✅ APPROVED FOR MERGE**

This implementation is exemplary and sets a high standard for future algorithm work. The combination of:
- Exceeding performance requirements by 4 orders of magnitude
- Comprehensive test coverage with edge cases
- Clean, idiomatic Rust following tech spec architecture
- Zero security concerns
- No technical debt introduced

...makes this a textbook example of well-engineered software. No changes required before merge.
