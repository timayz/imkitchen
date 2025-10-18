# Story 3.12: Recipe Complexity Calculation

Status: Approved

## Story

As a system,
I want to accurately calculate recipe complexity,
so that meal assignments match user capacity and availability.

## Acceptance Criteria

1. Complexity calculated automatically on recipe creation/update
2. Scoring factors: ingredient count (weight 30%), instruction step count (weight 40%), advance prep requirement (weight 30%)
3. Simple: <8 ingredients, <6 steps, no advance prep (score <30)
4. Moderate: 8-15 ingredients OR 6-10 steps (score 30-60)
5. Complex: >15 ingredients OR >10 steps OR advance prep required (score >60)
6. Complexity badge stored in recipe read model for fast filtering
7. Recalculated automatically when recipe edited
8. Complexity visible on recipe cards throughout app

## Tasks / Subtasks

- [ ] Task 1: Implement RecipeComplexityCalculator domain service (AC: #1, #2, #3, #4, #5)
  - [ ] Subtask 1.1: Create `RecipeComplexityCalculator` in `crates/recipe/src/complexity.rs`
  - [ ] Subtask 1.2: Implement scoring formula: `(ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)`
  - [ ] Subtask 1.3: Define advance_prep_multiplier: 0 if none, 50 if <4 hours, 100 if >=4 hours
  - [ ] Subtask 1.4: Map score to complexity enum (Simple <30, Moderate 30-60, Complex >60)
  - [ ] Subtask 1.5: Write unit tests for complexity calculation with edge cases

- [ ] Task 2: Integrate complexity calculation into recipe creation flow (AC: #1, #6)
  - [ ] Subtask 2.1: Call `RecipeComplexityCalculator::calculate()` in `RecipeCreated` event handler
  - [ ] Subtask 2.2: Store complexity in Recipe aggregate state
  - [ ] Subtask 2.3: Emit `RecipeComplexityCalculated` event with complexity value
  - [ ] Subtask 2.4: Update read model projection to store complexity in `recipes` table
  - [ ] Subtask 2.5: Write integration test for recipe creation with complexity calculation

- [ ] Task 3: Add complexity recalculation on recipe updates (AC: #7)
  - [ ] Subtask 3.1: Call `RecipeComplexityCalculator::calculate()` in `RecipeUpdated` event handler
  - [ ] Subtask 3.2: Compare new complexity with existing; emit `RecipeComplexityUpdated` if changed
  - [ ] Subtask 3.3: Update read model projection to update complexity in `recipes` table
  - [ ] Subtask 3.4: Write integration test for recipe update with complexity recalculation

- [ ] Task 4: Add complexity column to recipes read model (AC: #6)
  - [ ] Subtask 4.1: Create database migration `migrations/07_add_recipe_complexity.sql`
  - [ ] Subtask 4.2: Add `complexity` column to `recipes` table (ENUM: 'simple', 'moderate', 'complex')
  - [ ] Subtask 4.3: Create index on complexity column for fast filtering
  - [ ] Subtask 4.4: Backfill complexity for existing recipes (calculate from current data)

- [ ] Task 5: Display complexity on recipe cards and detail pages (AC: #8)
  - [ ] Subtask 5.1: Update `RecipeCard` template to display complexity badge
  - [ ] Subtask 5.2: Style complexity badges (Simple: green, Moderate: yellow, Complex: red)
  - [ ] Subtask 5.3: Update recipe detail template to show complexity with explanation
  - [ ] Subtask 5.4: Write E2E test with Playwright verifying complexity visibility

- [ ] Task 6: Add complexity filtering to recipe library (AC: #8)
  - [ ] Subtask 6.1: Add complexity filter checkboxes to recipe library page
  - [ ] Subtask 6.2: Update recipe query to filter by complexity
  - [ ] Subtask 6.3: Write integration test for complexity filtering

- [ ] Task 7: Integration with meal planning algorithm (AC: #3, #4, #5)
  - [ ] Subtask 7.1: Update `MealPlanningAlgorithm` to use complexity scores
  - [ ] Subtask 7.2: Match simple recipes to busy weeknights (<45min availability)
  - [ ] Subtask 7.3: Match complex recipes to days with more availability (weekends, >60min)
  - [ ] Subtask 7.4: Write unit tests for algorithm complexity matching logic

- [ ] Task 8: Comprehensive testing (AC: #1-#8)
  - [ ] Subtask 8.1: Unit tests for complexity calculation formula
  - [ ] Subtask 8.2: Integration tests for recipe creation/update with complexity
  - [ ] Subtask 8.3: E2E tests for complexity visibility and filtering
  - [ ] Subtask 8.4: Achieve 80% code coverage for complexity logic (cargo tarpaulin)

## Dev Notes

### Architecture Patterns and Constraints

**Domain Service Pattern**: `RecipeComplexityCalculator` is a stateless domain service that encapsulates complexity calculation business logic. It's invoked by the Recipe aggregate during command handling, ensuring complexity is always derived from current recipe data.

**Event Sourcing**: Complexity calculation results are captured in domain events (`RecipeComplexityCalculated`, `RecipeComplexityUpdated`). This provides audit trail and enables replaying complexity changes over time.

**CQRS Projection**: Complexity stored in `recipes` read model table for fast filtering and display. The projection handler subscribes to complexity events and updates the read model.

**Complexity Formula**:
```rust
complexity_score = (ingredient_count * 0.3) + (step_count * 0.4) + (advance_prep_multiplier * 0.3)

where:
- advance_prep_multiplier = 0 (no prep), 50 (<4 hours), 100 (>=4 hours)
- Simple: score < 30
- Moderate: score 30-60
- Complex: score > 60
```

**Example Calculations**:
- Simple recipe: 5 ingredients, 4 steps, no prep = (5 * 0.3) + (4 * 0.4) + (0 * 0.3) = 1.5 + 1.6 + 0 = 3.1 → Simple
- Moderate recipe: 10 ingredients, 8 steps, no prep = (10 * 0.3) + (8 * 0.4) + (0 * 0.3) = 3 + 3.2 + 0 = 6.2 → Moderate (score * 10 = 62, but actually should be 6.2... need to verify formula)
- Complex recipe: 12 ingredients, 6 steps, 4-hour marinade = (12 * 0.3) + (6 * 0.4) + (50 * 0.3) = 3.6 + 2.4 + 15 = 21 → Moderate (but advance prep should push to Complex)

**Note**: The formula from epics may need adjustment. With current weights, advance prep has too much influence (50 * 0.3 = 15). Consider normalizing or adjusting thresholds.

### Source Tree Components to Touch

**Domain Crate** (`crates/recipe/`):
- `src/complexity.rs` (NEW) - `RecipeComplexityCalculator` domain service
- `src/aggregate.rs` - Call complexity calculator in `RecipeCreated` and `RecipeUpdated` handlers
- `src/events.rs` - Add `RecipeComplexityCalculated` and `RecipeComplexityUpdated` events
- `src/read_model.rs` - Add projection handler for complexity events
- `tests/complexity_tests.rs` (NEW) - Unit tests for complexity calculation

**Database Migrations** (`migrations/`):
- `007_add_recipe_complexity.sql` (NEW) - Add complexity column and index

**Templates** (`templates/`):
- `components/recipe-card.html` - Add complexity badge display
- `pages/recipe-detail.html` - Add complexity section with explanation
- `pages/recipe-list.html` - Add complexity filter checkboxes

**Styling** (`static/css/`):
- Add complexity badge styles (green/yellow/red)

**Integration** (`crates/meal_planning/`):
- `src/algorithm.rs` - Use complexity scores for constraint satisfaction

### Project Structure Notes

**Alignment with Unified Project Structure**:

Per `solution-architecture.md` sections 2.1 and 14:
- Domain services in domain crate: `crates/recipe/src/complexity.rs`
- Event handlers invoke domain services: `RecipeAggregate` calls `RecipeComplexityCalculator`
- Read model projections: `project_recipe_complexity` handler updates `recipes` table
- Domain service is pure function (stateless, no dependencies)

**Database Schema Location**:
- Read model migration: `migrations/007_add_recipe_complexity.sql`
- Complexity column: `complexity TEXT CHECK(complexity IN ('simple', 'moderate', 'complex'))`
- Index for filtering: `CREATE INDEX idx_recipes_complexity ON recipes(complexity)`

**Testing Standards**:

Per `solution-architecture.md` section 15:
- **Unit tests**: Complexity calculation formula in `crates/recipe/tests/complexity_tests.rs`
- **Integration tests**: Recipe creation/update with complexity in `tests/recipe_tests.rs`
- **E2E tests**: Complexity visibility and filtering in `e2e/tests/recipe-management.spec.ts`
- **Coverage target**: 80% via `cargo tarpaulin`
- **TDD enforced**: Write tests first (red), implement (green), refactor (maintain green)

### Testing Standards Summary

**Test Cases for Complexity Calculation**:
1. Simple recipe: 5 ingredients, 4 steps, no prep → Simple
2. Moderate recipe (ingredients): 12 ingredients, 5 steps, no prep → Moderate
3. Moderate recipe (steps): 6 ingredients, 8 steps, no prep → Moderate
4. Complex recipe (ingredients): 18 ingredients, 5 steps, no prep → Complex
5. Complex recipe (steps): 8 ingredients, 12 steps, no prep → Complex
6. Complex recipe (advance prep): 6 ingredients, 5 steps, 4-hour marinade → Complex
7. Edge case: 0 ingredients, 0 steps → Invalid (should fail validation)
8. Edge case: Exactly 8 ingredients, 6 steps → Moderate (boundary)
9. Edge case: Negative values → Invalid (should fail validation)

**Integration Test Scenarios**:
1. Create recipe → Verify complexity stored in read model
2. Update recipe (add ingredients) → Verify complexity recalculated
3. Update recipe (no change to complexity factors) → Verify complexity unchanged
4. Filter recipes by complexity → Verify correct recipes returned

**E2E Test Scenarios**:
1. Create recipe with varying complexity → Verify badge displayed
2. Filter recipe library by complexity → Verify filtered results
3. View recipe detail → Verify complexity explanation shown

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-3.md#Services and Modules] - MealPlanningAlgorithm uses complexity scores
- [Source: docs/tech-spec-epic-3.md#Domain Services] - RecipeComplexityCalculator design
- [Source: docs/tech-spec-epic-2.md#Recipe Aggregate] - Recipe domain structure

**Solution Architecture**:
- [Source: docs/solution-architecture.md#2.1 Architecture Pattern] - Event-sourced DDD with domain services
- [Source: docs/solution-architecture.md#3.2 Data Models] - recipes table schema
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Recipe domain crate organization

**Epic Requirements**:
- [Source: docs/epics.md#Story 3.12] - Acceptance criteria and complexity formula
- [Source: docs/epics.md#Epic 3 Technical Summary] - Domain services for meal planning

**PRD Constraints**:
- [Source: docs/PRD.md#Non-Functional Requirements] - 80% code coverage, TDD enforced

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-3.12.xml) - Generated 2025-10-17

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
