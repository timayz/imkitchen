# Story 4.3: Multi-Week Shopping List Access

Status: Approved

## Story

As a user,
I want to view shopping lists for current and future weeks,
so that I can plan bulk shopping or shop ahead.

## Acceptance Criteria

1. Shopping list page displays week selector dropdown
2. Options: "This Week", "Next Week", "Week of {date}" for upcoming weeks
3. Selecting week generates shopping list for that week's meals
4. Current week highlighted as default
5. Future weeks accessible up to 4 weeks ahead
6. Each week's shopping list independent (no cross-week aggregation)
7. Past weeks not accessible (out of scope for MVP)
8. Week selection persists in URL query param for bookmarking

## Tasks / Subtasks

- [ ] Task 1: Update shopping list query to support week selection (AC: #3, #5, #6)
  - [ ] Subtask 1.1: Modify `GetShoppingListByWeek` query in `crates/shopping/src/read_model.rs` to accept week_start_date parameter
  - [ ] Subtask 1.2: Add validation to ensure week_start_date is a valid Monday (ISO week start)
  - [ ] Subtask 1.3: Add upper bound check: future weeks limited to +4 weeks from current date
  - [ ] Subtask 1.4: Return empty shopping list with clear message if no meal plan exists for selected week
  - [ ] Subtask 1.5: Write unit tests for week validation logic (valid Monday, future limit, past week rejection)

- [ ] Task 2: Implement week selector UI component (AC: #1, #2, #4)
  - [ ] Subtask 2.1: Create week selector dropdown in `templates/pages/shopping-list.html`
  - [ ] Subtask 2.2: Populate dropdown with options: "This Week", "Next Week", "Week of Oct 21", "Week of Oct 28", "Week of Nov 4"
  - [ ] Subtask 2.3: Generate week options dynamically (current week + 4 future weeks)
  - [ ] Subtask 2.4: Highlight current week as default selection with distinct styling
  - [ ] Subtask 2.5: Add icons/labels to distinguish current vs future weeks

- [ ] Task 3: Implement week selection with URL query params (AC: #8)
  - [ ] Subtask 3.1: Update shopping list route handler to parse `?week=2025-10-21` query parameter
  - [ ] Subtask 3.2: Default to current week if query param missing or invalid
  - [ ] Subtask 3.3: Update dropdown selection state based on URL query param
  - [ ] Subtask 3.4: Implement TwinSpark or JavaScript to update URL when dropdown selection changes
  - [ ] Subtask 3.5: Ensure browser back/forward navigation works correctly with week selection

- [ ] Task 4: Generate shopping lists on-demand for selected weeks (AC: #3, #6)
  - [ ] Subtask 4.1: Check if shopping list already exists for selected week (query `shopping_lists` by week_start_date)
  - [ ] Subtask 4.2: If exists, return cached shopping list from read model
  - [ ] Subtask 4.3: If not exists, trigger `GenerateShoppingList` command with selected week's meal plan
  - [ ] Subtask 4.4: Ensure each week's shopping list is independent (no cross-week ingredient aggregation)
  - [ ] Subtask 4.5: Handle edge case: meal plan not yet generated for future week (show helpful message)

- [ ] Task 5: Update shopping list route handler (AC: #1-#8)
  - [ ] Subtask 5.1: Update `GET /shopping` route in `src/routes/shopping.rs` to accept optional `week` query param
  - [ ] Subtask 5.2: Parse and validate week parameter (ISO 8601 date, Monday check)
  - [ ] Subtask 5.3: Query meal plan for selected week, verify it exists
  - [ ] Subtask 5.4: Invoke shopping list query with validated week_start_date
  - [ ] Subtask 5.5: Pass week options and selected week to template for dropdown rendering
  - [ ] Subtask 5.6: Return 404 with helpful message if user requests past week (per AC #7)

- [ ] Task 6: Styling and responsive design (AC: #1, #4)
  - [ ] Subtask 6.1: Style week selector dropdown with Tailwind CSS (consistent with app design)
  - [ ] Subtask 6.2: Add responsive styling for mobile (full-width dropdown, touch-friendly)
  - [ ] Subtask 6.3: Highlight current week with distinct color/badge (e.g., green "Current" label)
  - [ ] Subtask 6.4: Add loading indicator when switching weeks (if using AJAX)

- [ ] Task 7: Comprehensive testing (AC: #1-#8)
  - [ ] Subtask 7.1: Unit test: `GetShoppingListByWeek` with various week_start_dates (current, future, past, invalid)
  - [ ] Subtask 7.2: Unit test: Week validation logic (reject past weeks, accept current + 4 future weeks, reject 5+ weeks ahead)
  - [ ] Subtask 7.3: Integration test: GET /shopping?week=2025-10-21 returns correct week's shopping list
  - [ ] Subtask 7.4: Integration test: GET /shopping with no query param defaults to current week
  - [ ] Subtask 7.5: Integration test: GET /shopping?week=<past_date> returns 404 error
  - [ ] Subtask 7.6: Integration test: GET /shopping?week=<5_weeks_future> returns 400 error (out of range)
  - [ ] Subtask 7.7: E2E Playwright test: Navigate to shopping list, select different week from dropdown, verify list updates
  - [ ] Subtask 7.8: E2E test: Verify URL updates when week selected, browser back/forward works correctly
  - [ ] Subtask 7.9: Achieve 80% code coverage for shopping list query and route handler (cargo tarpaulin)

## Dev Notes

### Architecture Patterns and Constraints

**Week Selection Query Pattern**:
This story extends the existing shopping list query logic to support week-based filtering. The `GetShoppingListByWeek` query already accepts a week_start_date parameter (from Story 4.1), so the core infrastructure exists. This story focuses on:
1. Exposing week selection in the UI (dropdown)
2. Validating week ranges (current + 4 future weeks only)
3. URL-based state management via query parameters

**On-Demand Shopping List Generation**:
Shopping lists are generated lazily when first requested for a given week. The flow:
- User selects "Week of Oct 21" from dropdown
- Route handler checks if shopping list exists for that week (`shopping_lists` table query)
- If exists → return cached list from read model
- If not exists → trigger `GenerateShoppingList` command with meal_plan_id for that week
- `ShoppingListGenerated` event updates read model
- Template renders shopping list with category grouping (from Story 4.2)

**Week Independence**:
Each week's shopping list is an independent aggregate. Ingredients are NOT aggregated across weeks. This design decision simplifies implementation and aligns with user mental model (users shop per week, not multi-week bulk).

**URL Query Parameter State Management**:
Week selection persists in URL via `?week=YYYY-MM-DD` query param. Benefits:
- Bookmarkable URLs (users can share specific week's shopping list)
- Browser back/forward navigation works naturally
- Server-side rendering compatible (no client-side state)

**Week Validation Rules** (from PRD FR-9):
- Current week: Always accessible
- Future weeks: +1 to +4 weeks from current date (4 weeks total)
- Past weeks: Rejected with 404 error (out of MVP scope per epics.md)
- Invalid dates: Return 400 Bad Request with validation error

### Source Tree Components to Touch

**Domain Crate** (`crates/shopping/`):
- `src/read_model.rs` (UPDATE) - Enhance `GetShoppingListByWeek` with week range validation
- `src/aggregate.rs` (NO CHANGE) - Existing `GenerateShoppingList` command supports any week
- `src/error.rs` (UPDATE) - Add `InvalidWeekError`, `PastWeekNotAccessibleError`, `FutureWeekOutOfRangeError` variants

**HTTP Routes** (`src/routes/`):
- `src/routes/shopping.rs` (UPDATE) - Enhance `GET /shopping` handler to parse `?week=` query param, validate week, invoke query with selected week

**Templates** (`templates/`):
- `templates/pages/shopping-list.html` (UPDATE) - Add week selector dropdown above shopping list content, highlight current week, display selected week's list

**Database** (READ-ONLY):
- `shopping_lists` table already has `week_start_date` column (from Story 4.1 migration)
- No schema changes needed

### Project Structure Notes

**Alignment with Unified Project Structure**:

Per `solution-architecture.md` section 2.3:
- **Week selection route**: `GET /shopping?week=YYYY-MM-DD` (query param pattern for filtering)
- **Route handler location**: `src/routes/shopping.rs` (already exists from Story 4.1)
- **Query layer**: Week validation performed in `crates/shopping/src/read_model.rs` (not in template)

**Week Calculation Logic**:
- Use `chrono` crate for date manipulation (already dependency)
- Calculate week start (Monday) using `chrono::Datelike::iso_week()` and `NaiveDate::from_isoywd()`
- Current week: `Utc::now().date_naive().iso_week()`
- Future week validation: `(selected_week - current_week).num_weeks() <= 4`

**Template Data Structure**:
```rust
struct ShoppingListPageData {
    current_week: NaiveDate,
    selected_week: NaiveDate,
    week_options: Vec<WeekOption>, // Current + 4 future weeks
    shopping_list: Option<ShoppingListData>, // None if no meal plan for selected week
}

struct WeekOption {
    label: String, // "This Week", "Next Week", "Week of Oct 21"
    iso_date: String, // "2025-10-21" (ISO 8601)
    is_current: bool,
    is_selected: bool,
}
```

### Testing Standards Summary

**Unit Test Cases for Week Validation**:
1. Current week (Monday = today) → Valid, returns shopping list
2. Future week (+1 week) → Valid, returns shopping list if meal plan exists
3. Future week (+4 weeks) → Valid, at boundary
4. Future week (+5 weeks) → Invalid, returns `FutureWeekOutOfRangeError`
5. Past week (-1 week) → Invalid, returns `PastWeekNotAccessibleError`
6. Invalid date format ("2025-13-99") → Invalid, returns `InvalidWeekError`
7. Non-Monday date ("2025-10-22" is Tuesday) → Invalid, returns `InvalidWeekError` (week must start Monday)

**Integration Test Scenarios**:
1. GET /shopping (no query param) → Defaults to current week, returns 200 OK with shopping list
2. GET /shopping?week=2025-10-21 (valid future week) → Returns 200 OK with that week's shopping list
3. GET /shopping?week=2025-10-14 (valid past week) → Returns 404 Not Found with error message
4. GET /shopping?week=2025-11-25 (5 weeks future) → Returns 400 Bad Request with range error
5. GET /shopping?week=invalid → Returns 400 Bad Request with validation error
6. GET /shopping?week=2025-10-21 (no meal plan for that week) → Returns 200 OK with empty state message ("No meal plan for this week yet")

**E2E Test Scenarios** (Playwright):
1. Navigate to /shopping → Verify week dropdown displays with "This Week" selected
2. Select "Next Week" from dropdown → Verify URL updates to `?week=YYYY-MM-DD`, shopping list refreshes
3. Select "Week of Oct 28" → Verify correct week's shopping list displays
4. Click browser back button → Verify previous week's list restores
5. Bookmark URL with `?week=` param → Reopen bookmark, verify correct week loads
6. Verify current week highlighted with green badge/icon in dropdown
7. Verify dropdown responsive on mobile (touch-friendly, full-width)

### References

**Epic Requirements**:
- [Source: docs/epics.md#Story 4.3] - User story and AC for multi-week shopping list access
- [Source: docs/epics.md#Epic 4 Technical Summary] - Shopping domain architecture and evento subscriptions

**PRD Functional Requirements**:
- [Source: docs/PRD.md#FR-9: Multi-Week Shopping List Access] - Requirement for current + future week shopping lists

**Solution Architecture**:
- [Source: docs/solution-architecture.md#2.3 Page Routing and Navigation] - URL query param pattern for state (`?week=`)
- [Source: docs/solution-architecture.md#3.2 Data Models] - `shopping_lists` table schema with week_start_date column
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Shopping crate organization (read_model.rs for queries)

**Technical Specification**:
- [Source: docs/tech-spec-epic-4.md#CQRS Implementation] - `GetShoppingListByWeek` query signature
- [Source: docs/tech-spec-epic-4.md#Read Model Tables] - `shopping_lists` table schema (already includes week_start_date)

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.3.xml` (Generated: 2025-10-18)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
