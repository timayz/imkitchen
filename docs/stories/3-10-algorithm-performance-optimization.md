# Story 3.10: Algorithm Performance Optimization

Status: drafted

## Story

As a developer,
I want the meal planning algorithm to complete in <5 seconds,
So that users experience instant gratification and trust the system.

## Acceptance Criteria

1. Algorithm completes 5-week generation in <5 seconds (P95 latency)
2. Database queries optimized with indexes on favorited recipes, dietary restrictions
3. Recipe selection logic uses efficient filtering and randomization
4. Performance tests verify P95, P99 latencies with 100+ favorited recipes
5. Algorithm scales linearly with recipe count (no exponential growth)

## Tasks / Subtasks

- [ ] Profile current algorithm performance (AC: #1, #4)
  - [ ] Use `std::time::Instant` to measure generation time
  - [ ] Test with 10, 25, 50, 100, 200 favorited recipes
  - [ ] Record P50, P95, P99 latencies for each count
  - [ ] Identify bottlenecks using logs or profiling tools
  - [ ] Establish baseline before optimization

- [ ] Optimize database queries (AC: #2)
  - [ ] Add index on recipe_favorites(user_id) for fast favorited recipe lookup
  - [ ] Add index on recipes(owner_id, recipe_type) for filtering
  - [ ] Add index on user_profiles(user_id) for profile data
  - [ ] Use single query to load all favorited recipes (avoid N+1)
  - [ ] Measure query time before/after indexing

- [ ] Optimize in-memory data structures (AC: #3)
  - [ ] Use Vec instead of HashMap for recipe storage (better cache locality)
  - [ ] Pre-allocate Vec with estimated capacity
  - [ ] Use slices for passing data between functions (avoid clones)
  - [ ] Profile memory allocations with `cargo flamegraph`

- [ ] Optimize filtering logic (AC: #3)
  - [ ] Combine dietary restriction and recipe type filters in single pass
  - [ ] Use iterator chains instead of intermediate Vec allocations
  - [ ] Pre-compute cuisine frequency map once, reuse across weeks
  - [ ] Avoid redundant string comparisons

- [ ] Optimize random selection (AC: #3)
  - [ ] Reuse single RNG instance across generation (don't recreate)
  - [ ] Use `slice.choose()` for uniform selection (fast path)
  - [ ] Cache weighted selection distribution when possible
  - [ ] Avoid expensive weight recalculation in hot loop

- [ ] Add performance benchmarks (AC: #4, #5)
  - [ ] Create benchmark suite using Criterion.rs or built-in bencher
  - [ ] Benchmark generation with 10, 50, 100, 200 recipes
  - [ ] Measure P50, P95, P99 latencies across 100 iterations
  - [ ] Track latency growth: should be linear (O(n)), not quadratic (O(n²))
  - [ ] Fail benchmark if P95 > 5 seconds for 100 recipes

- [ ] Write performance test suite (AC: #1, #4)
  - [ ] Test generation 100 times with 100 favorited recipes
  - [ ] Calculate P95 and P99 latencies
  - [ ] Assert P95 < 5 seconds
  - [ ] Assert P99 < 8 seconds
  - [ ] Log latency distribution for analysis

- [ ] Test scalability (AC: #5)
  - [ ] Measure latency with 50 recipes
  - [ ] Measure latency with 100 recipes
  - [ ] Measure latency with 200 recipes
  - [ ] Calculate scaling factor: latency(200) / latency(100)
  - [ ] Assert scaling factor < 2.5 (approximately linear)

- [ ] Document performance characteristics (AC: #1, #5)
  - [ ] Record P95/P99 latencies for various recipe counts
  - [ ] Document Big O complexity of algorithm
  - [ ] Note any performance caveats or limits
  - [ ] Add performance section to CLAUDE.md or architecture.md

## Dev Notes

### Architecture Patterns

- **In-Memory Processing**: Load all data once, process in memory (ADR-002)
- **Cache Locality**: Use Vec for sequential access, avoid HashMap indirection
- **Single Query**: Fetch all favorited recipes in one database round-trip
- **Lazy Evaluation**: Use iterators to avoid intermediate allocations

### Project Structure Notes

Files to modify:
- `crates/imkitchen-mealplan/src/generator.rs` - Optimize algorithm implementation
- `migrations/queries/YYYYMMDDHHMMSS_indexes.sql` - Add performance indexes
- `tests/mealplan_test.rs` - Add performance test suite
- `benches/generation_bench.rs` - Optional Criterion benchmarks

### Technical Constraints

**Performance Target** [Source: PRD.md NFR002, epics.md Story 3.10]:
- P95 latency: < 5 seconds for 5-week generation with 100 favorited recipes
- P99 latency: < 8 seconds (allowing some variance)
- Linear scaling: O(n) where n = favorited recipe count
- No exponential growth (avoid O(n²) or worse)

**Database Indexes** [Source: architecture.md Performance Considerations]:
```sql
-- Recipe favorites lookup
CREATE INDEX idx_recipe_favorites_user ON recipe_favorites(user_id);

-- Recipe filtering
CREATE INDEX idx_recipes_owner_type ON recipes(owner_id, recipe_type);
CREATE INDEX idx_recipes_shared ON recipes(is_shared) WHERE is_shared = 1;

-- User profile lookup
CREATE INDEX idx_user_profiles_user ON user_profiles(user_id);

-- Meal plan queries
CREATE INDEX idx_meal_plans_user_week ON meal_plans(user_id, week_number);
CREATE INDEX idx_snapshots_meal_plan ON meal_plan_recipe_snapshots(meal_plan_id);
```

**Optimized Algorithm Structure** [Source: ADR-002, architecture.md]:
```rust
pub struct MealPlanGenerator {
    rng: ThreadRng,  // Reuse RNG
    cuisine_tracker: CuisineTracker,
    used_mains: HashSet<String>,
}

impl MealPlanGenerator {
    pub fn generate(
        &mut self,
        recipes: Vec<Recipe>,  // Pre-loaded in single query
        user_restrictions: &[String],
        variety_weight: f32,
        weeks: Vec<NaiveDate>,
    ) -> Vec<WeekData> {
        // Filter once, partition by type
        let (appetizers, mains, desserts, accompaniments) =
            Self::partition_recipes(recipes, user_restrictions);

        // Generate each week
        weeks.iter().map(|week_start| {
            self.generate_week(
                &appetizers,
                &mains,
                &desserts,
                &accompaniments,
                *week_start,
            )
        }).collect()
    }

    fn partition_recipes(
        recipes: Vec<Recipe>,
        restrictions: &[String],
    ) -> (Vec<Recipe>, Vec<Recipe>, Vec<Recipe>, Vec<Recipe>) {
        // Single pass partitioning with filtering
        recipes.into_iter()
            .filter(|r| matches_dietary_restrictions(r, restrictions))
            .fold(
                (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
                |(mut app, mut main, mut dess, mut acc), recipe| {
                    match recipe.recipe_type.as_str() {
                        "Appetizer" => app.push(recipe),
                        "MainCourse" => main.push(recipe),
                        "Dessert" => dess.push(recipe),
                        "Accompaniment" => acc.push(recipe),
                        _ => {}
                    }
                    (app, main, dess, acc)
                },
            )
    }
}
```

**Performance Measurement** [Source: epics.md Story 3.10 AC#4]:
```rust
#[tokio::test]
async fn test_generation_performance_p95() -> anyhow::Result<()> {
    // Setup: 100 favorited recipes
    let (executor, pool) = setup_test_db().await?;
    let user_id = create_test_user_with_recipes(&pool, 100).await?;

    let command = Command::new(executor, pool);
    let input = GenerateMealPlanInput { user_id: user_id.clone() };
    let metadata = EventMetadata { /* ... */ };

    // Measure 100 iterations
    let mut latencies = Vec::new();
    for _ in 0..100 {
        let start = std::time::Instant::now();
        command.generate_meal_plan(input.clone(), metadata.clone()).await?;
        latencies.push(start.elapsed().as_millis());
    }

    latencies.sort();
    let p95_index = (latencies.len() as f32 * 0.95) as usize;
    let p95_latency = latencies[p95_index];

    println!("P95 latency: {}ms", p95_latency);
    assert!(p95_latency < 5000, "P95 latency exceeds 5 seconds: {}ms", p95_latency);

    Ok(())
}
```

### Testing Strategy

[Source: CLAUDE.md Testing Guidelines, architecture.md Performance Considerations]
- **Performance Tests**: Measure latency distribution
  - Run 100 iterations for statistical validity
  - Calculate P50, P95, P99 percentiles
  - Assert P95 < 5000ms
  - Use tokio::test for async context
- **Scalability Tests**: Verify linear growth
  - Test with doubling recipe counts (50 → 100 → 200)
  - Calculate scaling factor
  - Assert approximately linear (factor < 2.5)
- **Profiling**: Use cargo flamegraph
  - Identify hot paths in algorithm
  - Optimize bottlenecks iteratively
  - Re-profile after changes

### References

- [Source: epics.md#Epic 3 Story 3.10]
- [Source: PRD.md NFR002 - P95 latency < 5 seconds]
- [Source: architecture.md ADR-002 - Pure Rust in-memory algorithm]
- [Source: architecture.md Performance Considerations - Indexing strategy]
- [Source: CLAUDE.md Testing Guidelines - Performance testing approach]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

### Completion Notes List

### File List
