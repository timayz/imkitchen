# Story 3.6: Cuisine Variety Scheduling

Status: drafted

## Story

As a user,
I want the algorithm to distribute cuisines evenly across weeks,
So that I don't eat the same cuisine type too frequently.

## Acceptance Criteria

1. Algorithm tracks cuisine frequency across all generated weeks
2. Cuisine variety weight (default 0.7) influences cuisine distribution
3. Higher weight (closer to 1.0) = maximum variety, lower weight = repeat cuisines more
4. Algorithm prioritizes less-recently-used cuisines when selecting recipes
5. Tests verify cuisine distribution respects variety weight configuration

## Tasks / Subtasks

- [ ] Load user cuisine variety weight from profile (AC: #2)
  - [ ] Query user_profiles table to fetch cuisine_variety_weight (default 0.7)
  - [ ] Pass weight to generation algorithm function
  - [ ] Validate weight is between 0.0 and 1.0

- [ ] Implement cuisine frequency tracking (AC: #1)
  - [ ] Create `CuisineTracker` struct to track cuisine usage counts
  - [ ] Initialize tracker with all known cuisine types at count 0
  - [ ] After selecting each recipe, increment cuisine count
  - [ ] Maintain tracker across all weeks during generation

- [ ] Implement cuisine-aware recipe selection (AC: #3, #4)
  - [ ] Modify recipe selection to consider cuisine frequency
  - [ ] Calculate selection weight for each recipe: `base_weight * (1 - (cuisine_frequency * variety_weight))`
  - [ ] Higher variety_weight = more penalty for repeated cuisines
  - [ ] Lower variety_weight = less penalty, allowing more repetition
  - [ ] Use weighted random selection based on calculated weights

- [ ] Define cuisine-aware selection algorithm (AC: #3, #4)
  - [ ] For each meal slot, calculate weights for all candidate recipes
  - [ ] Weight formula: `weight = 1.0 / (1.0 + (cuisine_count * variety_weight))`
  - [ ] Higher cuisine_count with high variety_weight = lower selection probability
  - [ ] Perform weighted random selection from candidate pool
  - [ ] Update cuisine frequency tracker after selection

- [ ] Update generation algorithm to use cuisine tracking (AC: #1, #2, #4)
  - [ ] Initialize CuisineTracker at generation start
  - [ ] For each week and day, use cuisine-aware selection
  - [ ] Track cuisine frequencies across all weeks
  - [ ] Ensure main course uniqueness still enforced alongside cuisine variety

- [ ] Write unit tests for cuisine tracking (AC: #1, #5)
  - [ ] Test tracker initialization with cuisine types
  - [ ] Test frequency incrementation after recipe selection
  - [ ] Test weight calculation with variety_weight = 0.0 (all equal weight)
  - [ ] Test weight calculation with variety_weight = 1.0 (maximum variety)
  - [ ] Test weight calculation with variety_weight = 0.7 (default)

- [ ] Write unit tests for weighted selection (AC: #3, #4, #5)
  - [ ] Create recipe pool with 3 cuisines: Italian (5 recipes), Chinese (5 recipes), Mexican (5 recipes)
  - [ ] Generate 21 meals with variety_weight = 1.0
  - [ ] Verify each cuisine used ~7 times (even distribution)
  - [ ] Generate 21 meals with variety_weight = 0.0
  - [ ] Verify uneven distribution (random selection, no variety enforcement)

- [ ] Write integration tests for cuisine variety (AC: #2, #3, #4, #5)
  - [ ] Create test user with cuisine_variety_weight = 0.9
  - [ ] Create 15 favorited recipes: 5 Italian, 5 Chinese, 5 Mexican
  - [ ] Execute generate_meal_plan for 4 weeks (28 meals)
  - [ ] Use evento::load to verify generated meal plan
  - [ ] Calculate cuisine distribution across weeks
  - [ ] Verify distribution is relatively even (no cuisine > 40% of meals)

- [ ] Write integration test for low variety weight (AC: #3, #5)
  - [ ] Create test user with cuisine_variety_weight = 0.2
  - [ ] Execute generation with same recipe pool
  - [ ] Verify distribution is more uneven (some cuisines > 40%)
  - [ ] Confirm algorithm respects lower variety preference

## Dev Notes

### Architecture Patterns

- **Stateful Tracking**: CuisineTracker maintains state across generation of all weeks
- **Weighted Selection**: Use weighted random selection instead of uniform random
- **Configuration-Driven**: User preference (variety_weight) controls algorithm behavior
- **Graceful Degradation**: If all recipes are same cuisine, algorithm still works

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/generator.rs` - Implement CuisineTracker and weighted selection
- `crates/imkitchen-mealplan/src/command.rs` - Load user cuisine_variety_weight
- `src/queries/users.rs` - Query function for user profile with cuisine_variety_weight
- `tests/mealplan_test.rs` - Integration tests for cuisine variety

### Technical Constraints

**Cuisine Variety Weight** [Source: PRD.md FR026, architecture.md user_profiles]:
- Default value: 0.7 (balanced variety)
- Range: 0.0 (no variety enforcement) to 1.0 (maximum variety)
- Stored in user_profiles.cuisine_variety_weight column
- User can adjust via profile settings

**Weighted Selection Algorithm** [Source: epics.md Story 3.6 ACs]:
```rust
struct CuisineTracker {
    counts: HashMap<String, u32>,
}

impl CuisineTracker {
    fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    fn increment(&mut self, cuisine: &str) {
        *self.counts.entry(cuisine.to_string()).or_insert(0) += 1;
    }

    fn get_count(&self, cuisine: &str) -> u32 {
        *self.counts.get(cuisine).unwrap_or(&0)
    }

    fn calculate_weight(&self, cuisine: &str, variety_weight: f32) -> f32 {
        let count = self.get_count(cuisine) as f32;
        1.0 / (1.0 + (count * variety_weight))
    }
}

fn select_recipe_with_cuisine_variety(
    recipes: &[Recipe],
    tracker: &CuisineTracker,
    variety_weight: f32,
) -> Option<&Recipe> {
    // Calculate weights for all recipes
    let weights: Vec<f32> = recipes
        .iter()
        .map(|r| tracker.calculate_weight(&r.cuisine_type, variety_weight))
        .collect();

    // Weighted random selection
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;

    let mut rng = thread_rng();
    let dist = WeightedIndex::new(&weights).ok()?;
    let index = dist.sample(&mut rng);

    recipes.get(index)
}
```

**Integration with Existing Algorithm** [Source: Story 3.1, Story 3.5]:
- Must work alongside dietary restriction filtering (Story 3.5)
- Must still enforce main course uniqueness (Story 3.1)
- Order of operations:
  1. Filter by dietary restrictions
  2. Filter by main course uniqueness (if applicable)
  3. Apply cuisine-aware weighted selection
  4. Update cuisine tracker after selection

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines]
- **Unit Tests**: Test tracking and weight calculation logic
  - Pure functions, no database needed
  - Test edge cases: all same cuisine, empty tracker, weight boundaries
- **Integration Tests**: Full generation with cuisine variety
  - Setup user with specific variety_weight
  - Create balanced recipe pool across cuisines
  - Generate meal plan
  - Measure cuisine distribution
  - Verify distribution aligns with variety_weight
  - Use evento::load for verification
- **Statistical Tests**: Verify variety behavior
  - Generate multiple meal plans with same weight
  - Calculate average cuisine distribution
  - Verify variance aligns with weight setting

### References

- [Source: epics.md#Epic 3 Story 3.6]
- [Source: PRD.md FR026 - Cuisine variety preferences]
- [Source: architecture.md Data Architecture - user_profiles.cuisine_variety_weight default 0.7]
- [Source: CLAUDE.md Command Guidelines - Load configuration data]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
