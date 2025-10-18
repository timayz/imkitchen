# Story 4.1: Generate Weekly Shopping List

Status: Approved

## Story

As a user with an active meal plan,
I want an automated shopping list for the week,
so that I can efficiently shop for all required ingredients.

## Acceptance Criteria

1. "Shopping List" button visible on dashboard and calendar pages
2. Clicking button generates shopping list for current week
3. System aggregates all ingredients from week's recipes
4. Duplicate ingredients combined with quantities summed (e.g., "onions 2" + "onions 3" = "onions 5")
5. Units normalized for aggregation (convert 1 cup to 240ml, combine with ml measurements)
6. Ingredients grouped by category: Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
7. Shopping list displays item count per category
8. Generation completes within 2 seconds
9. Shopping list persists and accessible for offline use
10. Confirmation: "Shopping list generated for Week of {date}"

## Tasks / Subtasks

- [ ] Task 1: Implement ShoppingList aggregate and domain logic (AC: #1, #2, #3, #8)
  - [ ] Subtask 1.1: Create `ShoppingList` aggregate in `crates/shopping/src/aggregate.rs`
  - [ ] Subtask 1.2: Define `GenerateShoppingList` command accepting meal_plan_id and week_start_date
  - [ ] Subtask 1.3: Implement `ShoppingListGenerated` event with meal_plan_id, week_start_date, item_list
  - [ ] Subtask 1.4: Add business logic to aggregate ingredients from all recipes in meal plan
  - [ ] Subtask 1.5: Write unit tests for shopping list generation with 3 overlapping recipes

- [ ] Task 2: Implement ingredient aggregation service (AC: #4, #5)
  - [ ] Subtask 2.1: Create `IngredientAggregationService` in `crates/shopping/src/aggregation.rs`
  - [ ] Subtask 2.2: Implement ingredient name normalization (case-insensitive, trim whitespace)
  - [ ] Subtask 2.3: Implement quantity summing for same-unit ingredients
  - [ ] Subtask 2.4: Implement unit conversion table (cups↔ml, tbsp↔tsp, lbs↔oz, g↔kg)
  - [ ] Subtask 2.5: Handle incompatible units by keeping separate line items
  - [ ] Subtask 2.6: Write unit tests for aggregation with multiple recipes and unit conversions

- [ ] Task 3: Implement category-based grouping (AC: #6, #7)
  - [ ] Subtask 3.1: Create `CategorizationService` in `crates/shopping/src/categorization.rs`
  - [ ] Subtask 3.2: Define Category enum (Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other)
  - [ ] Subtask 3.3: Create ingredient-to-category mapping with common defaults
  - [ ] Subtask 3.4: Assign category to each shopping list item
  - [ ] Subtask 3.5: Write unit tests verifying 50+ common ingredient mappings

- [ ] Task 4: Create database migrations and read models (AC: #9)
  - [ ] Subtask 4.1: Create migration `migrations/08_create_shopping_lists.sql`
  - [ ] Subtask 4.2: Add `shopping_lists` table (id, user_id, meal_plan_id, week_start_date, generated_at)
  - [ ] Subtask 4.3: Add `shopping_list_items` table (id, shopping_list_id, ingredient_name, quantity, unit, category, is_collected)
  - [ ] Subtask 4.4: Create indexes on user_id, meal_plan_id, and week_start_date
  - [ ] Subtask 4.5: Implement read model projections subscribing to `ShoppingListGenerated` event

- [ ] Task 5: Create HTTP routes and handlers (AC: #1, #2, #10)
  - [ ] Subtask 5.1: Add GET /shopping route to display current week's shopping list
  - [ ] Subtask 5.2: Add POST /shopping/generate handler invoking `GenerateShoppingList` command
  - [ ] Subtask 5.3: Query active meal plan and pass to shopping list generation
  - [ ] Subtask 5.4: Render shopping list template with categorized items
  - [ ] Subtask 5.5: Display confirmation message with week date
  - [ ] Subtask 5.6: Add "Shopping List" navigation button to dashboard and calendar pages

- [ ] Task 6: Create shopping list templates (AC: #6, #7, #10)
  - [ ] Subtask 6.1: Create `templates/pages/shopping-list.html` with Askama
  - [ ] Subtask 6.2: Render categorized ingredient groups with collapsible sections
  - [ ] Subtask 6.3: Display item count per category header
  - [ ] Subtask 6.4: Style with Tailwind CSS (responsive, touch-friendly)
  - [ ] Subtask 6.5: Add week selector for future multi-week access

- [ ] Task 7: Comprehensive testing (AC: #1-#10)
  - [ ] Subtask 7.1: Unit tests for ingredient aggregation service (edge cases, unit conversions)
  - [ ] Subtask 7.2: Unit tests for categorization service (50+ ingredients)
  - [ ] Subtask 7.3: Integration tests for shopping list generation from meal plan
  - [ ] Subtask 7.4: Integration test verifying 2-second generation performance with 14 recipes
  - [ ] Subtask 7.5: E2E Playwright test: Navigate to shopping list, verify categories and items
  - [ ] Subtask 7.6: Achieve 80% code coverage for shopping crate (cargo tarpaulin)

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing & CQRS**:
Shopping list generation is triggered by `MealPlanGenerated` or user action. The `GenerateShoppingList` command creates a `ShoppingList` aggregate, which emits `ShoppingListGenerated` event. Read model projections subscribe to this event to populate `shopping_lists` and `shopping_list_items` tables for fast querying.

**Domain Services**:
- `IngredientAggregationService`: Stateless service that normalizes ingredient names, converts units, and sums quantities. Handles complex aggregation logic like "2 cups flour + 240ml milk" → properly aggregated.
- `CategorizationService`: Maps ingredients to grocery store categories using predefined mappings. Extensible for future AI-based categorization.

**Performance Target**:
Generation must complete within 2 seconds for meal plans with up to 14 recipes (7 days × 2 meals/day). Algorithm complexity: O(n × m) where n = recipes, m = ingredients per recipe. With typical 10 ingredients/recipe: 140 ingredients to process, easily under 2s.

**Persistence**:
Shopping lists persist in database for offline access. Service workers cache shopping list pages for PWA offline functionality (Epic 5).

### Source Tree Components to Touch

**Domain Crate** (`crates/shopping/`):
- `src/aggregate.rs` - `ShoppingList` aggregate with `GenerateShoppingList` command
- `src/commands.rs` - Command handlers for shopping list operations
- `src/events.rs` - `ShoppingListGenerated` event schema
- `src/aggregation.rs` (NEW) - `IngredientAggregationService` domain service
- `src/categorization.rs` (NEW) - `CategorizationService` with category mappings
- `src/read_model.rs` - Read model projections for shopping list queries
- `tests/aggregation_tests.rs` (NEW) - Unit tests for aggregation logic
- `tests/categorization_tests.rs` (NEW) - Unit tests for category assignments

**Database Migrations** (`migrations/`):
- `008_create_shopping_lists.sql` (NEW) - Shopping lists and items tables with indexes

**HTTP Routes** (`src/routes/`):
- `src/routes/shopping.rs` (NEW) - Shopping list routes (GET /shopping, POST /shopping/generate)

**Templates** (`templates/`):
- `templates/pages/shopping-list.html` (NEW) - Shopping list display with categorized groups
- `templates/components/shopping-item.html` (optional) - Reusable item component
- `templates/pages/dashboard.html` (UPDATE) - Add "Shopping List" button
- `templates/pages/meal-calendar.html` (UPDATE) - Add "Shopping List" button

**Styling** (`static/css/`):
- Add shopping list specific styles (category headers, collapsible sections, checkboxes)

### Project Structure Notes

**Alignment with Unified Project Structure**:

Per `solution-architecture.md` sections 11.1 and 11.3:
- **Shopping domain crate**: `crates/shopping/` contains aggregates, commands, events, domain services
- **Domain services**: `IngredientAggregationService` and `CategorizationService` are pure functions (stateless)
- **Read model**: `shopping_lists` and `shopping_list_items` tables updated via evento subscriptions
- **Routes**: `src/routes/shopping.rs` orchestrates domain logic and renders templates (thin handlers)

**Database Schema Location**:
- Migration file: `migrations/008_create_shopping_lists.sql`
- Tables: `shopping_lists` (header), `shopping_list_items` (line items)
- Indexes: user_id, meal_plan_id, week_start_date for fast queries

**Testing Standards**:

Per `solution-architecture.md` section 15:
- **Unit tests**: Aggregation logic, categorization mappings in `crates/shopping/tests/`
- **Integration tests**: Shopping list generation from meal plan in `tests/shopping_tests.rs`
- **E2E tests**: Full user flow (click button, verify list) in `e2e/tests/shopping.spec.ts`
- **Coverage target**: 80% via `cargo tarpaulin`
- **TDD enforced**: Write tests first (red), implement (green), refactor (maintain green)

### Testing Standards Summary

**Unit Test Cases for Aggregation**:
1. Same ingredient, same unit: "chicken 2lbs" + "chicken 1lb" = "chicken 3lbs"
2. Same ingredient, convertible units: "milk 1 cup" + "milk 240ml" = "milk 2 cups"
3. Same ingredient, incompatible units: "onion 1 whole" + "onion 1 cup diced" = separate items
4. Different ingredients: No aggregation
5. Fractional quantities: "flour 1/2 cup" + "flour 1/4 cup" = "flour 3/4 cup"
6. Edge case: Zero quantity, negative quantity (validation errors)

**Unit Test Cases for Categorization**:
1. Produce: tomato, onion, lettuce, carrot → Category::Produce
2. Dairy: milk, cheese, yogurt, butter → Category::Dairy
3. Meat: chicken, beef, pork, fish → Category::Meat
4. Pantry: flour, sugar, pasta, rice → Category::Pantry
5. Spices: salt, pepper, cumin → Category::Pantry (or future Spices subcategory)
6. Unknown ingredient → Category::Other

**Integration Test Scenarios**:
1. Generate shopping list from meal plan with 7 recipes → Verify all ingredients present
2. Meal plan with 14 recipes, overlapping ingredients → Verify quantities aggregated correctly
3. Performance test: 14 recipes, 10 ingredients each (140 total) → Complete <2 seconds
4. Empty meal plan → Error message or empty shopping list

**E2E Test Scenarios**:
1. Navigate to dashboard, click "Shopping List" → Verify redirect to /shopping
2. View shopping list → Verify categories rendered (Produce, Dairy, Meat, etc.)
3. Verify item counts per category match actual items
4. Verify confirmation message displays week date

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md#Acceptance Criteria - Story 1] - Authoritative AC for shopping list generation
- [Source: docs/tech-spec-epic-4.md#Domain Services] - IngredientAggregationService and CategorizationService design
- [Source: docs/tech-spec-epic-4.md#System Architecture Alignment] - Shopping crate structure

**Solution Architecture**:
- [Source: docs/solution-architecture.md#3.2 Data Models] - shopping_lists and shopping_list_items schema
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Shopping domain crate organization
- [Source: docs/solution-architecture.md#11.3 Key Integrations] - Inter-domain communication via events

**Epic Requirements**:
- [Source: docs/epics.md#Story 4.1] - User story and AC for shopping list generation
- [Source: docs/epics.md#Epic 4 Technical Summary] - Aggregates, events, domain services overview

**PRD Constraints**:
- [Source: docs/PRD.md#FR-8: Shopping List Generation] - Functional requirement for automated shopping list
- [Source: docs/PRD.md#Non-Functional Requirements] - 80% code coverage, TDD enforced, <2s response time

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.1.xml) - Generated 2025-10-17

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
