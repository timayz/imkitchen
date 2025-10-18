# Story 4.5: Shopping List Item Checkoff

Status: ContextReadyDraft

## Story

As a user shopping at the store,
I want to check off items as I collect them,
so that I track progress and avoid missing items.

## Acceptance Criteria

1. Each shopping list item has checkbox
2. Tapping/clicking checkbox marks item as collected (strike-through styling)
3. Checked state persists across page refreshes
4. Progress indicator at top: "{checked} of {total} items collected"
5. Filter options: "Show All", "Show Remaining", "Show Collected"
6. Checked items move to bottom of category section
7. Checking all items in category collapses that section automatically
8. Reset button to uncheck all items (for next shopping trip)

## Tasks / Subtasks

- [ ] Task 1: Implement checkbox toggle command and event (AC: #1, #2, #3)
  - [ ] Subtask 1.1: Define `ShoppingListItemChecked` event in `crates/shopping/src/events.rs` with fields: `item_id`, `is_collected`, `checked_at`
  - [ ] Subtask 1.2: Create `MarkItemCollectedCommand` in `crates/shopping/src/commands.rs` with validation (item exists, belongs to user's shopping list)
  - [ ] Subtask 1.3: Implement `mark_item_collected` command handler that loads shopping list aggregate, validates item, and appends `ShoppingListItemChecked` event
  - [ ] Subtask 1.4: Add evento aggregate handler for `shopping_list_item_checked` event in `crates/shopping/src/aggregate.rs`
  - [ ] Subtask 1.5: Write unit tests for command handler (valid toggle, invalid item_id, permission checks)

- [ ] Task 2: Create read model projection for checkbox state (AC: #3)
  - [ ] Subtask 2.1: Implement `project_shopping_list_item_checked` handler in `crates/shopping/src/read_model.rs`
  - [ ] Subtask 2.2: Update `shopping_list_items.is_collected` column via SQL UPDATE query
  - [ ] Subtask 2.3: Store timestamp in `collected_at` column (add migration if needed)
  - [ ] Subtask 2.4: Ensure projection is idempotent (handle duplicate events gracefully)
  - [ ] Subtask 2.5: Write integration tests for projection logic

- [ ] Task 3: Implement HTTP route for checkbox toggle (AC: #2)
  - [ ] Subtask 3.1: Create `POST /shopping/items/:id/check` route in `src/routes/shopping.rs`
  - [ ] Subtask 3.2: Extract `item_id` from path parameter and `is_collected` boolean from form/JSON body
  - [ ] Subtask 3.3: Invoke `mark_item_collected` command from shopping domain crate
  - [ ] Subtask 3.4: Return TwinSpark-compatible HTML fragment with updated item (strike-through styling if checked)
  - [ ] Subtask 3.5: Handle errors (item not found, permission denied) with appropriate HTTP status codes
  - [ ] Subtask 3.6: Write integration tests for route (success, error cases)

- [ ] Task 4: Create checkbox UI component with TwinSpark (AC: #1, #2, #6)
  - [ ] Subtask 4.1: Update `templates/components/shopping-item.html` with checkbox input
  - [ ] Subtask 4.2: Add TwinSpark attributes: `ts-req="/shopping/items/{{item.id}}/check"`, `ts-req-method="POST"`, `ts-target="closest .shopping-item"`, `ts-swap="outerHTML"`
  - [ ] Subtask 4.3: Include hidden input for `is_collected` toggle value (opposite of current state)
  - [ ] Subtask 4.4: Apply strike-through CSS class (`.line-through`) when `is_collected` is true
  - [ ] Subtask 4.5: Implement automatic reordering: checked items move to bottom of category via DOM manipulation or server re-render
  - [ ] Subtask 4.6: Test checkbox interaction (click, visual feedback, server update)

- [ ] Task 5: Implement LocalStorage backup for offline persistence (AC: #3)
  - [ ] Subtask 5.1: Create JavaScript module `static/js/shopping-offline.js` for LocalStorage management
  - [ ] Subtask 5.2: On checkbox change, store state in LocalStorage: key=`shopping_item_${item_id}`, value=`is_collected` boolean
  - [ ] Subtask 5.3: On page load, read LocalStorage and apply checked states to checkboxes (before server update completes)
  - [ ] Subtask 5.4: Sync LocalStorage with server on successful POST response (clear outdated entries)
  - [ ] Subtask 5.5: Handle offline scenario: queue checkbox changes in LocalStorage, sync when online
  - [ ] Subtask 5.6: Test offline checkbox persistence (disconnect network, check items, refresh page, reconnect)

- [ ] Task 6: Build progress indicator (AC: #4)
  - [ ] Subtask 6.1: Query `shopping_list_items` for total count and checked count: `SELECT COUNT(*) as total, SUM(CASE WHEN is_collected THEN 1 ELSE 0 END) as checked`
  - [ ] Subtask 6.2: Pass progress data to `templates/pages/shopping-list.html` template context
  - [ ] Subtask 6.3: Render progress indicator at top of page: `<div class="progress">{checked} of {total} items collected</div>`
  - [ ] Subtask 6.4: Add progress bar visual (optional): `<progress value="{checked}" max="{total}"></progress>`
  - [ ] Subtask 6.5: Update progress indicator dynamically via TwinSpark when item checked (include in partial response)
  - [ ] Subtask 6.6: Test progress calculation accuracy

- [ ] Task 7: Implement filter options (AC: #5)
  - [ ] Subtask 7.1: Add filter buttons to `templates/pages/shopping-list.html`: "Show All", "Show Remaining", "Show Collected"
  - [ ] Subtask 7.2: Create route `GET /shopping?filter=all|remaining|collected` with query param handling
  - [ ] Subtask 7.3: Modify shopping list query in `crates/shopping/src/read_model.rs` to filter by `is_collected` based on query param
  - [ ] Subtask 7.4: Apply active state styling to selected filter button
  - [ ] Subtask 7.5: Use TwinSpark to update shopping list content without full page reload on filter change
  - [ ] Subtask 7.6: Test all three filter states (show correct items, hide others)

- [ ] Task 8: Implement category auto-collapse (AC: #7)
  - [ ] Subtask 8.1: Add JavaScript logic to detect when all items in category are checked
  - [ ] Subtask 8.2: Trigger collapse animation on category section when last item checked
  - [ ] Subtask 8.3: Store collapsed state in LocalStorage per category: key=`category_${category_name}_collapsed`
  - [ ] Subtask 8.4: On page load, restore collapsed state from LocalStorage
  - [ ] Subtask 8.5: Allow manual expand/collapse override (user can expand if desired)
  - [ ] Subtask 8.6: Test auto-collapse behavior (check all items, verify collapse, uncheck one, verify expand)

- [ ] Task 9: Add reset button (AC: #8)
  - [ ] Subtask 9.1: Create "Reset Checklist" button in `templates/pages/shopping-list.html`
  - [ ] Subtask 9.2: Implement route `POST /shopping/:week/reset` in `src/routes/shopping.rs`
  - [ ] Subtask 9.3: Create `ResetShoppingListCommand` that unchecks all items for given shopping list
  - [ ] Subtask 9.4: Emit `ShoppingListReset` event and update all items' `is_collected = false` in read model
  - [ ] Subtask 9.5: Clear LocalStorage for all items in this shopping list
  - [ ] Subtask 9.6: Show confirmation dialog before reset: "Are you sure? This will uncheck all items."
  - [ ] Subtask 9.7: Display success message: "Shopping list reset for next trip"
  - [ ] Subtask 9.8: Test reset functionality (check items, reset, verify all unchecked)

- [ ] Task 10: End-to-end testing (All ACs)
  - [ ] Subtask 10.1: Write Playwright E2E test: navigate to shopping list, check items, verify strike-through and progress
  - [ ] Subtask 10.2: Test persistence: check items, refresh page, verify state persists
  - [ ] Subtask 10.3: Test filter: check some items, filter to "Remaining", verify only unchecked shown
  - [ ] Subtask 10.4: Test offline: disconnect network, check items, reconnect, verify sync
  - [ ] Subtask 10.5: Test auto-collapse: check all items in category, verify collapse
  - [ ] Subtask 10.6: Test reset: check items, click reset, verify all unchecked
  - [ ] Subtask 10.7: Verify 80%+ code coverage for shopping domain checkbox logic

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- ShoppingList aggregate manages shopping list state via evento
- `ShoppingListItemChecked` event captures checkbox state changes
- `ShoppingListReset` event for bulk uncheck operation
- Read model projections update `shopping_list_items.is_collected` column
- All checkbox state changes maintain full audit trail

**CQRS Pattern:**
- Commands: `MarkItemCollectedCommand`, `ResetShoppingListCommand`
- Queries: Read from `shopping_list_items` table with filtering by `is_collected`
- Projections: `project_shopping_list_item_checked`, `project_shopping_list_reset`

**Progressive Enhancement with TwinSpark:**
- Checkbox toggles work via TwinSpark AJAX without full page reload
- Server returns HTML fragments for updated items
- Falls back to traditional form POST if JavaScript disabled
- LocalStorage provides offline-first experience

**Offline-First PWA:**
- LocalStorage caches checkbox state for offline use
- Service worker ensures shopping list page cached
- Background sync queues checkbox changes when offline
- Automatic sync when connectivity restored

### Source Tree Components

**Shopping Domain Crate (`crates/shopping/`):**
- `src/events.rs` - Define `ShoppingListItemChecked`, `ShoppingListReset` events
- `src/commands.rs` - Define `MarkItemCollectedCommand`, `ResetShoppingListCommand`
- `src/aggregate.rs` - Add event handlers for checkbox events
- `src/read_model.rs` - Implement projections for checkbox state
- `tests/` - Unit tests for command/event logic

**HTTP Routes (`src/routes/shopping.rs`):**
- `POST /shopping/items/:id/check` - Toggle checkbox for single item
- `POST /shopping/:week/reset` - Reset all checkboxes for shopping list
- `GET /shopping?filter=all|remaining|collected` - Filter shopping list by checkbox state

**Askama Templates:**
- `templates/pages/shopping-list.html` - Main shopping list page with progress indicator, filter buttons, reset button
- `templates/components/shopping-item.html` - Individual shopping item with checkbox, TwinSpark attributes, strike-through styling
- `templates/partials/shopping-category.html` - Category section with auto-collapse logic

**JavaScript/Static Assets:**
- `static/js/shopping-offline.js` - LocalStorage management for offline checkbox persistence
- `static/js/category-collapse.js` - Auto-collapse logic for fully checked categories

**Database Schema:**
- `shopping_list_items` table already has `is_collected` BOOLEAN column
- Add `collected_at` TIMESTAMP column via migration (optional for audit trail)
- Indexes: `idx_shopping_list_items_collected` on `(shopping_list_id, is_collected)` for filter queries

### Testing Standards

**Unit Tests (`crates/shopping/tests/`):**
- Test `MarkItemCollectedCommand` validation (item exists, user owns shopping list)
- Test `ResetShoppingListCommand` bulk uncheck logic
- Test evento aggregate event handlers update state correctly
- Coverage target: 90%+ for shopping domain checkbox logic

**Integration Tests (`tests/shopping_tests.rs`):**
- Test `POST /shopping/items/:id/check` route (success, errors)
- Test `POST /shopping/:week/reset` route with confirmation
- Test filter query parameter handling
- Test read model projections update database correctly

**E2E Tests (`e2e/tests/shopping.spec.ts`):**
- Test complete shopping flow: view list, check items, verify progress
- Test checkbox persistence across page refresh
- Test filter functionality (Show All, Remaining, Collected)
- Test offline checkbox with network disconnect/reconnect
- Test auto-collapse when all category items checked
- Test reset button unchecks all items

**TDD Approach:**
- Write failing test for checkbox command handler
- Implement command to pass test
- Write failing test for read model projection
- Implement projection to pass test
- Write failing E2E test for checkbox UI interaction
- Implement TwinSpark integration to pass test

### Project Structure Notes

**Alignment with Unified Project Structure:**
- Shopping domain follows established crate pattern from Stories 4.1-4.4
- TwinSpark pattern consistent with Story 4.4 (real-time updates)
- LocalStorage integration follows PWA offline-first pattern from Epic 5
- Server-side rendering with Askama maintains architecture consistency

**Detected Conflicts or Variances:**
- None - checkbox functionality integrates cleanly with existing shopping list implementation
- LocalStorage is additive (doesn't conflict with server-side state)
- Category auto-collapse is client-side enhancement (progressive)

### References

**Source Documentation:**
- [Epic 4 Story 4.5 Requirements - docs/epics.md:974-995]
- [Shopping List Data Model - docs/solution-architecture.md:354-364]
- [Event Sourcing Pattern - docs/solution-architecture.md:54-73]
- [TwinSpark Progressive Enhancement - docs/solution-architecture.md:533-560]
- [PWA Offline Strategy - docs/solution-architecture.md:951-1003]
- [Shopping Domain Crate Structure - docs/solution-architecture.md:1856-1868]
- [Testing Strategy - docs/solution-architecture.md:1951-2066]

**Technical Specifications:**
- [Epic 4 Tech Spec - docs/tech-spec-epic-4.md] (shopping list generation, aggregation logic)
- [Solution Architecture - docs/solution-architecture.md] (evento, CQRS, server-side rendering)

**Previous Story Learnings:**
- [Story 4.1 - docs/stories/story-4.1.md] (ShoppingList aggregate, database schema)
- [Story 4.2 - docs/stories/story-4.2.md] (Category grouping, collapsible sections)
- [Story 4.4 - docs/stories/story-4.4.md] (TwinSpark real-time updates, checkbox state preservation)

## Dev Agent Record

### Context Reference

**Context File:** `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.5.xml`

**Generated:** 2025-10-18

**Contents:**
- Complete architecture patterns (Event Sourcing, CQRS, TwinSpark, Offline-First PWA)
- Existing codebase analysis (shopping domain, events, commands, aggregates, routes)
- Technology stack and dependencies (evento, sqlx, axum, askama, TwinSpark)
- Database schema (shopping_lists, shopping_list_items with is_collected column)
- Testing standards (unit, integration, E2E with Playwright)
- Implementation tasks breakdown (10 tasks, 60+ subtasks)
- Technical specifications (API endpoints, event schemas, UI components)
- Reference documentation (PRD, architecture, tech specs, previous stories)
- Performance requirements and constraints
- Open questions and implementation notes

### Agent Model Used

Claude 3.5 Sonnet (claude-sonnet-4-5-20250929)

### Debug Log References

N/A - Story in Draft status

### Completion Notes List

N/A - Story not yet started

### File List

**Expected Files to Create/Modify:**

**Domain Layer:**
- `crates/shopping/src/events.rs` - Add `ShoppingListItemChecked`, `ShoppingListReset` events
- `crates/shopping/src/commands.rs` - Add `MarkItemCollectedCommand`, `ResetShoppingListCommand`
- `crates/shopping/src/aggregate.rs` - Add event handlers
- `crates/shopping/src/read_model.rs` - Add projections

**HTTP Layer:**
- `src/routes/shopping.rs` - Add checkbox toggle and reset routes

**Templates:**
- `templates/pages/shopping-list.html` - Add progress indicator, filter buttons, reset button
- `templates/components/shopping-item.html` - Add checkbox with TwinSpark
- `templates/partials/shopping-category.html` - Add auto-collapse logic

**JavaScript:**
- `static/js/shopping-offline.js` - LocalStorage management (new file)
- `static/js/category-collapse.js` - Auto-collapse logic (new file)

**Migrations:**
- `migrations/XXX_add_collected_at_to_shopping_list_items.sql` - Add `collected_at` timestamp column (optional)

**Tests:**
- `crates/shopping/tests/checkbox_tests.rs` - Unit tests
- `tests/shopping_checkbox_tests.rs` - Integration tests
- `e2e/tests/shopping-checkbox.spec.ts` - E2E tests
