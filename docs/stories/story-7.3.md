# Story 7.3: Accompaniment Selection

Status: Approved

## Story

As a **meal planning algorithm**,
I want to **pair main courses with compatible accompaniments**,
so that **complete meals include appropriate sides when main courses accept them**.

## Acceptance Criteria

1. Function `select_accompaniment(main_course, available)` implemented
2. Returns `None` if `main_course.accepts_accompaniment == false`
3. Filters by `preferred_accompaniments` if specified
4. Selects random from filtered list using `thread_rng`
5. Returns `None` if no compatible accompaniments
6. Allows repetition (not tracked in rotation)
7. Unit tests cover pairing scenarios
8. Random selection uses `rand::thread_rng`

## Tasks / Subtasks

- [ ] Implement accompaniment selection function (AC: 1)
  - [ ] Create function in `crates/meal_planning/src/algorithm.rs`
  - [ ] Signature: `pub fn select_accompaniment(main_course: &Recipe, available_accompaniments: &[Recipe]) -> Option<Recipe>`
  - [ ] Return `Option<Recipe>`

- [ ] Check if main course accepts accompaniment (AC: 2)
  - [ ] Read `main_course.accepts_accompaniment` boolean field
  - [ ] If `false`, immediately return `None`
  - [ ] Skip all filtering and selection logic

- [ ] Filter by preferred accompaniment categories (AC: 3)
  - [ ] Check if `main_course.preferred_accompaniments` is non-empty
  - [ ] If specified, filter `available_accompaniments` where `accompaniment.accompaniment_category` is in preferred list
  - [ ] If empty or unspecified, use all available accompaniments

- [ ] Implement random selection (AC: 4, 8)
  - [ ] Use `rand::thread_rng()` for randomness
  - [ ] Use `.choose(&mut rng)` method on filtered slice
  - [ ] Clone selected recipe for return (ownership)

- [ ] Handle no compatible accompaniments (AC: 5)
  - [ ] If filtered list is empty, return `None`
  - [ ] Do not panic or error

- [ ] Allow accompaniment repetition (AC: 6)
  - [ ] Accompaniments NOT tracked in `RotationState`
  - [ ] Can reuse same accompaniment multiple times in week
  - [ ] Document this design decision

- [ ] Write unit tests (AC: 7)
  - [ ] Test main course with `accepts_accompaniment = false` returns None
  - [ ] Test main course with `accepts_accompaniment = true` and preferred categories filters correctly
  - [ ] Test random selection (use seeded RNG for determinism)
  - [ ] Test empty preferred categories uses all available
  - [ ] Test no compatible accompaniments returns None
  - [ ] Test accompaniment repetition allowed (call function twice, may return same recipe)

## Dev Notes

### Architecture Patterns

**Random Selection Strategy:**
- Use `rand` crate's `thread_rng()` for non-deterministic selection
- Tests use `StdRng::seed_from_u64()` for reproducible tests
- Provides variety across meal plan generations

**Accompaniment Categories:**
```rust
pub enum AccompanimentCategory {
    Pasta,
    Rice,
    Fries,
    Salad,
    Bread,
    Vegetable,
    Other,
}

pub struct Recipe {
    accepts_accompaniment: bool,
    preferred_accompaniments: Vec<AccompanimentCategory>,  // For main courses
    accompaniment_category: Option<AccompanimentCategory>,  // If recipe IS an accompaniment
    // ... other fields
}
```

**Example Pairings:**
- Chicken Tikka Masala (main) + Rice (accompaniment)
- Grilled Steak (main) + Fries or Salad (random choice)
- Pasta Carbonara (main) → accepts_accompaniment = false (already complete)

**Repetition Design:**
- Main courses NEVER repeat (tracked in RotationState)
- Appetizers/Desserts repeat after exhaustion
- Accompaniments CAN repeat freely (not tracked)
- Rationale: Sides are simple, less variety needed

### Project Structure Notes

**File Location:**
- `crates/meal_planning/src/algorithm.rs` - Accompaniment selection logic

**Dependency:**
- Add `rand = "0.8"` to `crates/meal_planning/Cargo.toml`

**Data Model Fields:**
- `Recipe.accepts_accompaniment: bool` - Does main course accept a side?
- `Recipe.preferred_accompaniments: Vec<AccompanimentCategory>` - Preferred sides (empty = any)
- `Recipe.accompaniment_category: Option<AccompanimentCategory>` - Category if recipe is a side

### Testing Standards

**Test Pattern for Randomness:**
```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

#[test]
fn test_random_selection_deterministic() {
    let mut rng = StdRng::seed_from_u64(12345);
    // Use seeded RNG for reproducible test results
}
```

**Test Scenarios:**
1. Main doesn't accept accompaniment → None
2. Main prefers Pasta/Rice → only those offered
3. Empty preferences → all accompaniments available
4. No compatible accompaniments → None
5. Multiple compatible → random selection (verify non-panic)
6. Same accompaniment selected twice (repetition allowed)

### References

- [Tech Spec: Section 3.3 - Accompaniment Selection](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.3](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Domain Models: AccompanimentCategory enum](../tech-spec-epic-7.md#data-models-and-contracts)
- [Workflows: Accompaniment pairing in week generation](../tech-spec-epic-7.md#workflows-and-sequencing)
- [Rand Crate Documentation](https://docs.rs/rand/latest/rand/)

## Dev Agent Record

### Context Reference

- [Story Context XML: story-context-7.3.xml](../story-context-7.3.xml) - Generated 2025-10-26

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
