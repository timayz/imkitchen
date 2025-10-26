# Story 7.7: Algorithm Integration Tests and Benchmarks

Status: Draft

## Story

As a **development team**,
I want **comprehensive integration tests and performance benchmarks for the meal planning algorithm**,
so that **we ensure correctness, reliability, and performance targets are met**.

## Acceptance Criteria

1. Integration test: full multi-week generation with realistic data (50 recipes)
2. Test: dietary restrictions filter correctly (all enums + custom)
3. Test: time/skill constraints respected (weeknight vs weekend, skill levels)
4. Test: main courses never repeat across weeks (uniqueness verified)
5. Test: accompaniments assigned correctly (pairing logic)
6. Benchmark: 5-week generation <5 seconds using `criterion` crate
7. Coverage >80% for algorithm module (measured via `cargo-tarpaulin`)

## Tasks / Subtasks

- [ ] Create integration test suite (AC: 1)
  - [ ] Create `tests/integration/test_multi_week.rs`
  - [ ] Set up test fixture: 50 recipes (15 appetizers, 20 mains, 15 desserts, 10 accompaniments)
  - [ ] Vary complexity, cuisines, dietary tags, times
  - [ ] Create realistic UserPreferences
  - [ ] Call `generate_multi_week_meal_plans()` end-to-end
  - [ ] Assert successful generation (no errors)
  - [ ] Assert 5 weeks generated
  - [ ] Assert 21 assignments per week

- [ ] Test dietary restriction filtering (AC: 2)
  - [ ] Test Vegan user: all assigned recipes have Vegan tag
  - [ ] Test GlutenFree + DairyFree: recipes have both tags
  - [ ] Test Custom("peanuts"): no recipes with peanut ingredients
  - [ ] Iterate through all assignments, validate dietary tags
  - [ ] Assert `filter_by_dietary_restrictions` integration

- [ ] Test time and skill constraints (AC: 3)
  - [ ] Test Beginner user: all assigned main courses are Simple complexity
  - [ ] Test Intermediate user: no Complex main courses
  - [ ] Test weeknight assignments: prep+cook ≤ 30 minutes
  - [ ] Test weekend assignments: prep+cook ≤ 90 minutes
  - [ ] Iterate through assignments by day of week, validate constraints

- [ ] Test main course uniqueness (AC: 4)
  - [ ] Collect all main course recipe IDs from 5 weeks (35 total)
  - [ ] Assert no duplicates: `unique_ids.len() == 35`
  - [ ] Test with exactly 35 main courses (boundary: full rotation)
  - [ ] Test with 40 main courses (5 unused at end)

- [ ] Test accompaniment pairing (AC: 5)
  - [ ] Find main courses with `accepts_accompaniment = true`
  - [ ] Assert corresponding `MealAssignment` has `accompaniment_recipe_id = Some(...)`
  - [ ] Verify accompaniment recipe exists and is correct category
  - [ ] Find main courses with `accepts_accompaniment = false`
  - [ ] Assert `accompaniment_recipe_id = None`

- [ ] Create performance benchmark suite (AC: 6)
  - [ ] Create `benches/algorithm_benchmarks.rs`
  - [ ] Add `criterion` dev dependency to `Cargo.toml`
  - [ ] Benchmark: `filter_by_dietary_restrictions` with 100 recipes
  - [ ] Benchmark: `select_main_course_with_preferences` with 100 candidates
  - [ ] Benchmark: `generate_single_week` with 50 recipes
  - [ ] Benchmark: `generate_multi_week_meal_plans` 5 weeks with 50 recipes
  - [ ] Assert P95 < 5 seconds for multi-week generation
  - [ ] Run with `cargo bench`

- [ ] Measure code coverage (AC: 7)
  - [ ] Install `cargo-tarpaulin`: `cargo install cargo-tarpaulin`
  - [ ] Run: `cargo tarpaulin --package meal_planning --out Html --output-dir coverage/`
  - [ ] Open `coverage/index.html` and review
  - [ ] Identify uncovered lines, add tests if critical
  - [ ] Assert overall coverage >80% for `crates/meal_planning/src/algorithm.rs`

- [ ] Edge case integration tests
  - [ ] Test insufficient recipes error (6 appetizers, 7 mains, 7 desserts)
  - [ ] Test dietary filtering reduces to insufficient
  - [ ] Test single week generation (7 of each type)
  - [ ] Test appetizer/dessert exhaustion and reset mid-generation
  - [ ] Test no compatible main course for specific day (error propagation)

- [ ] CI/CD integration
  - [ ] Add test execution to CI pipeline
  - [ ] Add benchmark execution (informational, not blocking)
  - [ ] Add coverage report generation
  - [ ] Fail build if coverage <80%

## Dev Notes

### Architecture Patterns

**Integration Test Structure:**
```rust
// tests/integration/test_multi_week.rs

use meal_planning::*;
use chrono::Local;

#[tokio::test]
async fn test_full_multi_week_generation_realistic_data() {
    // Setup
    let recipes = create_test_recipe_library(); // 50 recipes
    let preferences = UserPreferences {
        dietary_restrictions: vec![DietaryRestriction::Vegetarian],
        max_prep_time_weeknight: 30,
        max_prep_time_weekend: 90,
        skill_level: SkillLevel::Intermediate,
        avoid_consecutive_complex: true,
        cuisine_variety_weight: 0.7,
    };
    let user_id = UserId::new();

    // Execute
    let result = generate_multi_week_meal_plans(user_id, recipes, preferences).await;

    // Assert
    assert!(result.is_ok());
    let meal_plan = result.unwrap();
    assert_eq!(meal_plan.generated_weeks.len(), 5);
    for week in &meal_plan.generated_weeks {
        assert_eq!(week.meal_assignments.len(), 21);
    }
}
```

**Benchmark Structure:**
```rust
// benches/algorithm_benchmarks.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use meal_planning::*;

fn benchmark_multi_week_generation(c: &mut Criterion) {
    let recipes = create_test_recipe_library();
    let preferences = create_test_preferences();
    let user_id = UserId::new();

    c.bench_function("multi_week_5_weeks_50_recipes", |b| {
        b.iter(|| {
            generate_multi_week_meal_plans(
                black_box(user_id.clone()),
                black_box(recipes.clone()),
                black_box(preferences.clone())
            )
        });
    });
}

criterion_group!(benches, benchmark_multi_week_generation);
criterion_main!(benches);
```

**Coverage Interpretation:**
- **>80% overall:** Good (meets AC-7)
- **Critical paths 100%:** dietary filtering, main selection, week generation
- **Lower coverage acceptable:** error handling branches, edge cases
- **Exclude from coverage:** Debug implementations, test utilities

### Project Structure Notes

**Test Files:**
```
crates/meal_planning/
├── tests/
│   ├── integration/
│   │   ├── test_multi_week.rs
│   │   ├── test_dietary_filtering.rs
│   │   └── test_constraints.rs
│   └── fixtures/
│       ├── test_recipes.rs
│       └── test_preferences.rs
├── benches/
│   └── algorithm_benchmarks.rs
└── Cargo.toml (add criterion dev-dependency)
```

**Cargo.toml Dependencies:**
```toml
[dev-dependencies]
criterion = "0.5"
tokio = { version = "1.0", features = ["macros", "rt-multi-thread"] }
```

**Running Tests and Benchmarks:**
```bash
# Integration tests
cargo test --package meal_planning --test '*'

# Unit + integration
cargo test --package meal_planning

# Benchmarks
cargo bench --package meal_planning

# Coverage
cargo tarpaulin --package meal_planning --out Html --output-dir coverage/
```

### Testing Standards

**Test Data Realism:**
- Realistic recipe names, cuisines, times
- Varied complexity distribution (50% Simple, 30% Moderate, 20% Complex)
- Multiple cuisines (Italian, Mexican, Chinese, Indian, etc.)
- Dietary tag combinations (some Vegan+GlutenFree, etc.)

**Assertion Strategies:**
- **Positive:** Assert expected behavior (21 assignments, 5 weeks)
- **Negative:** Assert errors when expected (insufficient recipes)
- **Property:** Assert invariants (main uniqueness, dietary compliance)
- **Performance:** Assert time bounds (P95 <5s)

**Test Isolation:**
- Each test creates own data (no shared state)
- Use `#[tokio::test]` for async functions
- Clean up any side effects (though algorithm is pure)

**CI/CD Configuration:**
```yaml
# .github/workflows/test.yml
- name: Run meal planning tests
  run: cargo test --package meal_planning

- name: Run meal planning benchmarks
  run: cargo bench --package meal_planning

- name: Generate coverage
  run: |
    cargo install cargo-tarpaulin
    cargo tarpaulin --package meal_planning --out Html --output-dir coverage/

- name: Check coverage threshold
  run: |
    # Parse coverage percentage, fail if <80%
```

### References

- [Tech Spec: Section 3.7 - Integration Tests](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.7](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Tech Spec: Test Strategy Summary](../tech-spec-epic-7.md#test-strategy-summary)
- [Tech Spec: Performance Targets](../tech-spec-epic-7.md#performance)
- [Tech Spec: Coverage >80%](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Criterion Crate Documentation](https://docs.rs/criterion/latest/criterion/)
- [Tarpaulin Coverage Tool](https://github.com/xd009642/tarpaulin)
- [Story 7.1-7.6: All algorithm components](./story-7.1.md) - Integration test targets

## Dev Agent Record

### Context Reference

<!-- Story context XML will be added by story-context workflow -->

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
