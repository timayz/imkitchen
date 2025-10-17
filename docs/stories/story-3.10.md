# Story 3.10: Handle Insufficient Recipes for Generation

Status: Approved

## Story

As a **user with too few favorite recipes**,
I want **clear guidance on what's needed**,
so that **I can successfully generate a meal plan**.

## Acceptance Criteria

1. "Generate Meal Plan" button visible but triggers validation
2. Error message: "You need at least 7 favorite recipes to generate a weekly meal plan. You currently have {count}."
3. Helpful guidance: "Add {7 - count} more recipes to get started!"
4. Direct link to "Add Recipe" page or "Discover Recipes" page
5. Error displayed with friendly styling (not alarming red)
6. Validation prevents wasted algorithm execution
7. Count updates in real-time as user adds/removes favorites

## Tasks / Subtasks

### Task 1: Implement Pre-flight Validation (AC: 1, 2, 6)
- [ ] Add validation check in meal plan generation route handler
  - [ ] Query favorite recipe count: `SELECT COUNT(*) FROM recipes WHERE user_id=? AND is_favorite=true`
  - [ ] Compare count against minimum threshold (7 recipes)
  - [ ] If insufficient, return validation error instead of executing algorithm
- [ ] Create `InsufficientRecipesError` in meal planning domain error types
  - [ ] Include current_count and required_count fields
  - [ ] Map to HTTP 422 Unprocessable Entity status
- [ ] Write unit tests:
  - [ ] Test: count < 7 returns validation error
  - [ ] Test: count >= 7 proceeds to generation
  - [ ] Test: error includes correct counts in message

### Task 2: Create Helpful Error Template (AC: 2, 3, 4, 5)
- [ ] Create or update error display template component
  - [ ] Render error message with dynamic count: "You need at least 7 favorite recipes... You currently have {count}."
  - [ ] Calculate and display recipes needed: {7 - count} more recipes
  - [ ] Use friendly styling: soft orange/yellow background, informational icon (not red alert)
- [ ] Add action buttons to error message
  - [ ] "Add Recipe" button linking to `/recipes/new`
  - [ ] "Discover Recipes" button linking to `/discover`
  - [ ] Buttons styled as primary CTAs for clear user action
- [ ] Write integration tests:
  - [ ] Test: error page renders with correct message
  - [ ] Test: action links navigate to correct routes
  - [ ] Test: friendly styling applied (no alarming colors)

### Task 3: Real-time Count Update (AC: 7)
- [ ] Add favorite count display to dashboard
  - [ ] Query and display: "You have {count} favorite recipes"
  - [ ] Update count when user favorites/unfavorites recipe
  - [ ] Show progress toward minimum: "{count}/7 recipes (need {7-count} more to generate plan)"
- [ ] Add conditional button state on dashboard
  - [ ] If count < 7: Button shows "Add More Recipes" (disabled generation)
  - [ ] If count >= 7: Button shows "Generate Meal Plan" (enabled)
  - [ ] Tooltip explains requirement when hovering disabled button
- [ ] Write E2E tests with Playwright:
  - [ ] Test: user with 5 recipes sees helpful guidance
  - [ ] Test: user adds 2 more recipes, count updates, button enables
  - [ ] Test: user clicks "Generate Meal Plan" with sufficient recipes, generation succeeds

## Dev Notes

### Validation Logic
- Minimum threshold: **7 favorite recipes** (configurable via domain constant)
- Validation occurs before algorithm execution to avoid wasted computation
- Pre-flight check in route handler: `crates/meal_planning/src/read_model.rs` query
- Domain error type: `MealPlanningError::InsufficientRecipes { current: usize, required: usize }`

### UI/UX Considerations
- **Friendly tone**: Avoid alarming language or aggressive red styling
- **Actionable guidance**: Clear next steps with direct navigation links
- **Progress indicator**: Show users how close they are to goal
- **Consistent messaging**: Same count threshold across all validation points

### Architecture Patterns
- **Validation-first**: Pre-flight checks prevent invalid state transitions
- **CQRS Read Model**: Favorite count query from `recipes` read model table
- **Server-rendered errors**: Askama template for error page (no client-side validation)
- **Progressive enhancement**: TwinSpark for real-time count updates (optional)

### Project Structure Notes

**Files to Create/Modify:**
- `crates/meal_planning/src/error.rs` - Add `InsufficientRecipes` error variant
- `crates/meal_planning/src/read_model.rs` - Add `count_favorite_recipes(user_id)` query
- `src/routes/meal_plan.rs` - Add pre-flight validation in generation handler
- `templates/components/error-message.html` - Reusable error component with friendly styling
- `templates/pages/dashboard.html` - Add favorite count display and conditional button state
- `tests/meal_plan_tests.rs` - Integration tests for validation flow
- `e2e/tests/meal-planning.spec.ts` - E2E test for insufficient recipes scenario

**Expected Module Locations:**
```
crates/meal_planning/
├── src/
│   ├── error.rs              # Add InsufficientRecipes variant
│   ├── read_model.rs         # Add count_favorite_recipes query
│   └── commands.rs           # No changes needed
src/routes/
├── meal_plan.rs              # Add validation before generation
templates/
├── components/
│   └── error-message.html    # Friendly error component
├── pages/
│   └── dashboard.html        # Count display and button state
```

### References

- **[Source: docs/epics.md#story-310]** - Acceptance criteria and validation requirements
- **[Source: docs/solution-architecture.md#32-data-models-and-relationships]** - `recipes` table schema with `is_favorite` column
- **[Source: docs/solution-architecture.md#412-form-actions-and-mutations]** - Error handling pattern with HTTP 422 status
- **[Source: docs/solution-architecture.md#71-component-structure]** - Askama template component patterns

### Testing Standards

**Target Coverage:** 80% code coverage per NFR requirements

**Test Types:**
1. **Unit Tests** (Rust):
   - Validation logic in meal planning domain
   - Error type serialization and formatting
   - Favorite count query accuracy

2. **Integration Tests** (Rust):
   - Route handler validation behavior
   - Error page rendering with correct data
   - HTTP status codes and response structure

3. **E2E Tests** (Playwright):
   - Full user flow: insufficient recipes → add recipes → enable generation
   - Error message display and navigation links
   - Real-time count updates and button state changes

**TDD Approach:**
- Write tests first for validation logic
- Implement validation in route handler
- Create error templates and verify rendering
- Ensure all tests pass before completion

## Dev Agent Record

### Context Reference

**Story Context XML:** [story-context-3.10.xml](../story-context-3.10.xml)

Generated on: 2025-10-17
Workflow: BMAD Story Context Workflow v1.0

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
