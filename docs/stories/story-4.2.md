# Story 4.2: Category-Based Ingredient Grouping

Status: Approved

## Story

As a user shopping in a grocery store,
I want ingredients grouped by store section,
so that I can shop efficiently without backtracking.

## Acceptance Criteria

1. Shopping list displays collapsible sections per category
2. Default categories: Produce, Dairy, Meat & Seafood, Pantry, Frozen, Bakery, Other
3. Each category shows item count (e.g., "Produce (8 items)")
4. Items within category listed alphabetically
5. User can expand/collapse categories
6. All categories expanded by default on first view
7. Category order matches typical grocery store layout
8. Empty categories hidden from view

## Tasks / Subtasks

- [ ] Task 1: Implement category assignment service (AC: #2, #7)
  - [ ] Subtask 1.1: Create `CategorizationService` in `crates/shopping/src/categorization.rs`
  - [ ] Subtask 1.2: Define `Category` enum with variants: Produce, Dairy, Meat, Pantry, Frozen, Bakery, Other
  - [ ] Subtask 1.3: Implement `assign_category()` function with keyword matching logic
  - [ ] Subtask 1.4: Create keyword mapping constants (PRODUCE_KEYWORDS, DAIRY_KEYWORDS, MEAT_KEYWORDS, SPICE_KEYWORDS, BAKING_KEYWORDS, PANTRY_KEYWORDS)
  - [ ] Subtask 1.5: Write unit tests verifying category assignment for 50+ common ingredients

- [ ] Task 2: Update shopping list generation to include categories (AC: #2)
  - [ ] Subtask 2.1: Modify `aggregate_ingredients()` in `crates/shopping/src/aggregation.rs` to call `assign_category()` for each ingredient
  - [ ] Subtask 2.2: Update `AggregatedIngredient` struct to include `category: String` field
  - [ ] Subtask 2.3: Update `ShoppingListGenerated` event to include category per item
  - [ ] Subtask 2.4: Update read model projection to store category in `shopping_list_items.category` column
  - [ ] Subtask 2.5: Write integration test verifying categories persist in database

- [ ] Task 3: Update shopping list template with category sections (AC: #1, #3, #4, #5, #6, #7, #8)
  - [ ] Subtask 3.1: Modify `templates/pages/shopping-list.html` to group items by category
  - [ ] Subtask 3.2: Implement collapsible category sections using TwinSpark or CSS details/summary
  - [ ] Subtask 3.3: Display item count in category header (e.g., "Produce (8 items)")
  - [ ] Subtask 3.4: Sort items alphabetically within each category
  - [ ] Subtask 3.5: Set all categories to expanded state by default
  - [ ] Subtask 3.6: Hide empty categories (categories with 0 items)
  - [ ] Subtask 3.7: Order categories in typical grocery store layout: Produce, Dairy, Meat, Frozen, Pantry, Bakery, Other

- [ ] Task 4: Style category sections with Tailwind CSS (AC: #1, #3)
  - [ ] Subtask 4.1: Create category header styles with distinct background colors per category
  - [ ] Subtask 4.2: Style item count badge in header
  - [ ] Subtask 4.3: Add expand/collapse icon indicators (chevron or +/-)
  - [ ] Subtask 4.4: Style category sections for mobile and desktop responsiveness
  - [ ] Subtask 4.5: Add smooth transition animations for expand/collapse

- [ ] Task 5: Update shopping list query logic (AC: #8)
  - [ ] Subtask 5.1: Modify `GetShoppingListByWeek` query in `crates/shopping/src/read_model.rs` to fetch items grouped by category
  - [ ] Subtask 5.2: Return items sorted by category order (Produce first, Other last)
  - [ ] Subtask 5.3: Filter out empty categories before returning to template
  - [ ] Subtask 5.4: Add category-level aggregation (item count per category)

- [ ] Task 6: Comprehensive testing (AC: #1-#8)
  - [ ] Subtask 6.1: Unit tests for `assign_category()` with 50+ ingredients covering all categories
  - [ ] Subtask 6.2: Unit test for category enum ordering (Produce=0, Dairy=1, ..., Other=6)
  - [ ] Subtask 6.3: Integration test: Generate shopping list, verify categories assigned correctly
  - [ ] Subtask 6.4: Integration test: Verify empty categories excluded from response
  - [ ] Subtask 6.5: E2E Playwright test: View shopping list, verify category sections rendered
  - [ ] Subtask 6.6: E2E test: Verify categories expanded by default
  - [ ] Subtask 6.7: E2E test: Collapse and expand categories, verify state changes
  - [ ] Subtask 6.8: Achieve 80% code coverage for categorization module (cargo tarpaulin)

## Dev Notes

### Architecture Patterns and Constraints

**Category Assignment Service**:
The `CategorizationService` is a pure, stateless domain service that maps ingredient names to grocery store categories using keyword matching. This service is called during the ingredient aggregation phase (Story 4.1) to ensure every shopping list item has a category assigned before persisting to the database.

**Category Enum**:
Categories are represented as a Rust enum with explicit ordering that mirrors typical grocery store layouts. This ordering is used both for database storage (as string values) and for UI display sorting.

```rust
pub enum Category {
    Produce,    // Fresh fruits and vegetables
    Dairy,      // Milk, cheese, yogurt, butter
    Meat,       // Meat, poultry, fish, seafood
    Frozen,     // Frozen foods section
    Pantry,     // Shelf-stable items (rice, pasta, canned goods)
    Bakery,     // Bread, baked goods, flour, sugar
    Other,      // Uncategorized items
}
```

**Keyword Matching Logic**:
Category assignment uses simple substring matching against predefined keyword lists. For example, if an ingredient name contains "chicken", "beef", "pork", or "fish", it's assigned to the Meat category. This approach provides 95%+ accuracy for common ingredients without requiring ML models in MVP.

**Collapsible Sections**:
Category sections use HTML `<details>` and `<summary>` elements for native browser collapse/expand functionality, eliminating JavaScript dependencies. TwinSpark can optionally enhance with AJAX if future requirements demand dynamic category updates without page reload.

**Empty Category Filtering**:
Categories with zero items are filtered out in the query layer (`GetShoppingListByWeek`) before passing data to the template. This ensures the UI never displays empty "Frozen (0 items)" sections, keeping the interface clean.

### Source Tree Components to Touch

**Domain Crate** (`crates/shopping/`):
- `src/categorization.rs` (NEW) - `CategorizationService` with `assign_category()` function and keyword constants
- `src/aggregation.rs` (UPDATE) - Call `assign_category()` for each `AggregatedIngredient`
- `src/aggregate.rs` (UPDATE) - `AggregatedIngredient` struct includes `category` field
- `src/events.rs` (UPDATE) - `ShoppingListGenerated` event includes category per item
- `src/read_model.rs` (UPDATE) - `GetShoppingListByWeek` query groups by category
- `tests/categorization_tests.rs` (NEW) - Unit tests for category assignment (50+ ingredients)

**Templates** (`templates/`):
- `templates/pages/shopping-list.html` (UPDATE) - Add category section grouping with collapsible headers

**Styling** (`static/css/`):
- Tailwind CSS utility classes for category headers, item lists, expand/collapse icons

**Database Migration** (if not already in Story 4.1):
- Verify `shopping_list_items.category` column exists (should be added in Story 4.1 migration)

### Project Structure Notes

**Alignment with Unified Project Structure**:

Per `solution-architecture.md` section 11.1:
- **Domain service location**: `crates/shopping/src/categorization.rs` is a pure function module (stateless logic)
- **Keyword constants**: Defined as `const` arrays in `categorization.rs` for compile-time optimization
- **Read model grouping**: Category grouping performed in query layer (`read_model.rs`), not in template logic

**Database Schema** (from Story 4.1):
- `shopping_list_items.category` column stores category as TEXT (enum variant name: "produce", "dairy", "meat", "frozen", "pantry", "bakery", "other")
- Index on `(shopping_list_id, category)` for fast category-grouped queries

**Testing Standards**:

Per `solution-architecture.md` section 15:
- **Unit tests**: Category assignment logic with edge cases (unknown ingredients → "other", mixed case names, plurals)
- **Integration tests**: End-to-end shopping list generation verifying categories assigned and persisted
- **E2E tests**: Full user flow (view shopping list, verify categories displayed, expand/collapse sections)
- **Coverage target**: 80% via `cargo tarpaulin`
- **TDD enforced**: Write tests first (red), implement (green), refactor (maintain green)

### Testing Standards Summary

**Unit Test Cases for Category Assignment**:
1. Produce: "tomato", "spinach", "carrot", "apple" → `Category::Produce`
2. Dairy: "milk", "cheese", "yogurt", "butter" → `Category::Dairy`
3. Meat: "chicken breast", "ground beef", "salmon", "shrimp" → `Category::Meat`
4. Frozen: "frozen peas", "ice cream" → `Category::Frozen` (requires FROZEN_KEYWORDS addition)
5. Pantry: "rice", "pasta", "canned tomatoes", "olive oil" → `Category::Pantry`
6. Bakery: "flour", "sugar", "bread", "yeast" → `Category::Bakery`
7. Other: "unknown ingredient" → `Category::Other`
8. Edge cases: Case-insensitive ("Chicken Breast" → Meat), plurals ("tomatoes" → Produce)

**Integration Test Scenarios**:
1. Generate shopping list with 3 recipes → Verify all items have non-null category
2. Generate shopping list with items spanning all categories → Verify each category present in database
3. Query shopping list grouped by category → Verify items returned in correct category order
4. Generate shopping list with only Produce items → Verify only Produce category returned (other categories filtered)

**E2E Test Scenarios** (Playwright):
1. Navigate to shopping list page → Verify category headers rendered (Produce, Dairy, Meat, etc.)
2. Verify item counts displayed in headers (e.g., "Produce (8 items)")
3. Verify all categories expanded by default (all items visible)
4. Click category header to collapse → Verify items hidden
5. Click again to expand → Verify items visible again
6. Verify empty categories not displayed (e.g., if no Frozen items, no Frozen section)
7. Verify category order: Produce first, Other last

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md#Module: categorization.rs] - Category assignment service design with keyword matching logic
- [Source: docs/tech-spec-epic-4.md#Module: aggregation.rs] - Ingredient aggregation algorithm calling `assign_category()`
- [Source: docs/tech-spec-epic-4.md#Read Model Tables] - `shopping_list_items.category` column schema

**Solution Architecture**:
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Shopping domain crate organization (categorization as domain service)
- [Source: docs/solution-architecture.md#3.2 Data Models] - `shopping_list_items` table schema with category column

**Epic Requirements**:
- [Source: docs/epics.md#Story 4.2] - User story and AC for category-based ingredient grouping
- [Source: docs/epics.md#Epic 4 Technical Summary] - CategorizationService overview

**PRD Constraints**:
- [Source: docs/PRD.md#FR-8: Shopping List Generation] - Functional requirement for category-grouped shopping lists
- [Source: docs/PRD.md#Non-Functional Requirements] - 80% code coverage, TDD enforced

## Dev Agent Record

### Context Reference

- [Story Context XML](/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.2.xml) - Generated 2025-10-18

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
