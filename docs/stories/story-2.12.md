# Story 2.12: Batch Recipe Import

Status: Done

## Story

As a user,
I want to import multiple recipes at once from a JSON file,
so that I can quickly populate my recipe library without entering each recipe manually.

## Acceptance Criteria

1. "Import Recipes" button displayed next to "New recipe" button on recipe list page
2. Clicking "Import Recipes" opens modal with file upload interface
3. Modal displays link to download `example-recipe.json` as format reference
4. File input accepts `.json` files only
5. JSON file must contain array of recipe objects (not single recipe)
6. Each recipe in array validated against same schema as single recipe creation
7. Required fields per recipe: title, recipe_type, ingredients (min 1), instructions (min 1)
8. Free tier limit enforced: import rejected if (current_count + import_count) > 10
9. Successful recipes imported, failed recipes reported with specific error messages
10. Results modal displays: successful count, failed count, per-recipe error details
11. Page refreshes after clicking "Done" to show newly imported recipes
12. Partial success supported: some recipes can succeed while others fail (no rollback)

## Tasks / Subtasks

### Phase 1: Backend Foundation (TDD) - Estimated 3-4 hours

- [ ] Write failing unit tests (`crates/recipe/tests/batch_import_tests.rs`)
  - [ ] `test_batch_import_valid_recipes`: 3 valid recipes → 3 successful, 0 failed (AC: 6, 9)
  - [ ] `test_batch_import_rejects_free_tier_overflow`: Free user with 8 recipes, import 3 → RecipeLimitExceeded error (AC: 8)
  - [ ] `test_batch_import_partial_failure`: 2 valid, 1 invalid → 2 successful, 1 failed with error message (AC: 9, 12)
  - [ ] `test_batch_import_empty_array`: Empty array `[]` → validation error (AC: 5)
  - [ ] `test_batch_import_invalid_recipe_type`: Invalid recipe_type → validation error (AC: 6)
  - [ ] `test_batch_import_missing_required_fields`: Missing title/ingredients → validation error per recipe (AC: 7)

- [ ] Implement BatchImportRecipesCommand (AC: 5, 6, 7, 8, 9)
  - [ ] Create `BatchImportRecipe` struct in `crates/recipe/src/commands.rs`
  - [ ] Create `BatchImportRecipesCommand` struct with `user_id` and `recipes: Vec<BatchImportRecipe>`
  - [ ] Create `BatchImportResult` struct with `successful_recipe_ids`, `failed_imports`, `total_attempted`
  - [ ] Export from `crates/recipe/src/lib.rs`

- [ ] Implement BatchImportCompleted event (AC: 9, 10)
  - [ ] Add `BatchImportCompleted` event in `crates/recipe/src/events.rs`
  - [ ] Include fields: `user_id`, `successful_recipe_ids`, `failed_imports: Vec<(usize, String)>`, `total_attempted`
  - [ ] Derive `evento::Event` trait

- [ ] Implement batch_import_recipes function (AC: 6, 7, 8, 9, 12)
  - [ ] Create `batch_import_recipes()` in `crates/recipe/src/lib.rs`
  - [ ] Check free tier limit: query current recipe count, validate (current + import) ≤ 10
  - [ ] Loop through recipes array, validate each recipe
  - [ ] Call existing `create_recipe()` command per valid recipe
  - [ ] Collect successful recipe IDs and failed imports with error messages
  - [ ] Return `BatchImportResult` with counts and details
  - [ ] DO NOT rollback successful recipes if later ones fail (partial success) (AC: 12)

- [ ] Verify all unit tests pass
  - [ ] Run: `cargo test -p recipe batch_import`
  - [ ] Ensure 100% of test cases pass

### Phase 2: HTTP Route (TDD) - Estimated 2-3 hours

- [ ] Write integration test (`tests/batch_import_integration_tests.rs`)
  - [ ] `test_post_import_recipes_success`: Upload valid JSON with 2 recipes → 200 OK, "2 recipes imported" (AC: 9, 10)
  - [ ] `test_post_import_recipes_invalid_json`: Upload invalid JSON syntax → 422, error message (AC: 5)
  - [ ] `test_post_import_recipes_empty_array`: Upload `[]` → 422, "No recipes found" (AC: 5)
  - [ ] `test_post_import_recipes_exceeds_limit`: Free user, upload 3 recipes when at 9/10 → 403, limit error (AC: 8)
  - [ ] `test_post_import_recipes_partial_failure`: Upload 2 valid + 1 invalid → 200 OK, "2 successful, 1 failed" (AC: 9, 12)
  - [ ] `test_post_import_recipes_requires_auth`: Upload without auth → 401 Unauthorized

- [ ] Implement POST /recipes/import route handler (AC: 4, 5, 6, 7, 8, 9, 10)
  - [ ] Add `post_import_recipes()` function in `src/routes/recipes.rs`
  - [ ] Accept `Multipart` form data with file upload
  - [ ] Extract file contents from `recipes_file` field
  - [ ] Parse JSON to `Vec<BatchImportRecipe>` using `serde_json::from_str()`
  - [ ] Validate root is array (not single object) (AC: 5)
  - [ ] Validate array is non-empty (AC: 5)
  - [ ] Create `BatchImportRecipesCommand` with authenticated user_id
  - [ ] Call `batch_import_recipes()` from recipe crate
  - [ ] Handle `RecipeLimitExceeded` error with 403 and user-friendly message (AC: 8)
  - [ ] Render `BatchImportResultTemplate` with success/failure counts and per-recipe errors (AC: 10)

- [ ] Create BatchImportResultTemplate (AC: 10)
  - [ ] Add `BatchImportResultTemplate` struct in `src/routes/recipes.rs`
  - [ ] Include fields: `successful_count`, `failed_count`, `total_attempted`, `failures: Vec<(usize, String)>`, `user`

- [ ] Register route in router (AC: 1, 2, 3, 4)
  - [ ] Add `.route("/recipes/import", post(post_import_recipes))` in `src/server.rs` or `src/routes/mod.rs`
  - [ ] Protect with auth middleware (existing pattern)

- [ ] Verify integration tests pass
  - [ ] Run: `cargo test batch_import_integration`
  - [ ] Ensure all scenarios covered

### Phase 3: UI Templates - Estimated 2-3 hours

- [ ] Modify recipe list page template (AC: 1, 2)
  - [ ] Edit `templates/pages/recipe-list.html`
  - [ ] Add "Import Recipes" button next to existing "New recipe" button
  - [ ] Button markup:
    ```html
    <button
        ts-req="/recipes/import-modal"
        ts-target="#import-modal"
        ts-swap="innerHTML"
        class="btn-secondary">
        <svg><!-- Upload icon --></svg>
        Import Recipes
    </button>
    ```
  - [ ] Add empty modal container: `<div id="import-modal"></div>`

- [ ] Create batch import modal template (AC: 2, 3, 4)
  - [ ] Create `templates/components/batch-import-modal.html`
  - [ ] Add modal header with title "Import Recipes" and close button
  - [ ] Add description text explaining JSON array format
  - [ ] Add download link to `/example-recipe.json` (AC: 3)
  - [ ] Create form with `enctype="multipart/form-data"`
  - [ ] Add file input: `<input type="file" name="recipes_file" accept=".json,application/json" required>` (AC: 4)
  - [ ] Add TwinSpark attributes: `ts-req="/recipes/import"`, `ts-target="#import-modal"`, `ts-swap="innerHTML"`
  - [ ] Add "Cancel" and "Upload and Import" buttons
  - [ ] Style with Tailwind CSS (modal-backdrop, modal-content, form-field classes)

- [ ] Create batch import results template (AC: 10, 11)
  - [ ] Create `templates/components/batch-import-results.html`
  - [ ] Display success message if `successful_count > 0`: "X recipes imported successfully!"
  - [ ] Display error message if `failed_count > 0`: "X recipes failed to import:"
  - [ ] List per-recipe errors: "Recipe #N: error_message"
  - [ ] Add "Done" button that calls `window.location.reload()` (AC: 11)
  - [ ] Style with success-message (green), error-message (red) classes

- [ ] Add GET /recipes/import-modal route (AC: 2)
  - [ ] Create `get_import_modal()` function in `src/routes/recipes.rs`
  - [ ] Render `BatchImportModalTemplate` with authenticated user
  - [ ] Register route: `.route("/recipes/import-modal", get(get_import_modal))`

- [ ] Test modal flow manually
  - [ ] Click "Import Recipes" → modal opens
  - [ ] Modal displays format reference link
  - [ ] File input only accepts .json files
  - [ ] Upload triggers AJAX request via TwinSpark
  - [ ] Results modal displays correctly

### Phase 4: Manual & E2E Testing - Estimated 1-2 hours

- [ ] Create test JSON files
  - [ ] `test-batch-valid.json`: Array with 2-3 valid recipes
  - [ ] `test-batch-invalid.json`: Mix of 1 valid + 1 invalid (missing title)
  - [ ] `test-batch-empty.json`: Empty array `[]`
  - [ ] `test-batch-limit.json`: 11 recipes (for free tier limit test)

- [ ] Manual testing scenarios
  - [ ] Valid batch import: Upload `test-batch-valid.json` → 2-3 recipes imported successfully (AC: 9)
  - [ ] Invalid JSON syntax: Upload malformed JSON → 422 error message (AC: 5)
  - [ ] Empty array: Upload `test-batch-empty.json` → "No recipes found" error (AC: 5)
  - [ ] Free tier limit: Create 9 recipes manually, upload 2 more → limit exceeded error (AC: 8)
  - [ ] Partial failure: Upload `test-batch-invalid.json` → 1 successful, 1 failed with error detail (AC: 9, 12)
  - [ ] Verify imported recipes appear in recipe list after refresh (AC: 11)

- [ ] E2E test with Playwright (optional, covered by integration tests)
  - [ ] Create `e2e/tests/batch-import.spec.ts`
  - [ ] Test: Navigate to /recipes → Click "Import Recipes" → Upload file → Verify results modal
  - [ ] Test: Verify recipes appear in list after clicking "Done"
  - [ ] Test: File input validation (reject non-.json files)
  - [ ] Run: `npx playwright test batch-import`

- [ ] Accessibility audit
  - [ ] Modal keyboard navigation: Tab cycles within modal, Escape closes
  - [ ] Screen reader labels: File input has descriptive `aria-label`
  - [ ] Focus management: Focus returns to "Import Recipes" button after modal closes
  - [ ] Run: `axe-core` or manual screen reader test (NVDA/VoiceOver)

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- BatchImportCompleted event captures the batch operation result
- Individual RecipeCreated events still emitted per successful recipe (via existing create_recipe)
- No rollback mechanism: successful recipes remain even if later ones fail (partial success pattern)

**CQRS Pattern:**
- Command: BatchImportRecipesCommand writes multiple RecipeCreated events
- Query: Recipe list reads from recipes table (no changes needed)
- Partial success: Some recipes project successfully, others don't (acceptable trade-off)

**Domain-Driven Design:**
- Batch import logic in recipe domain crate (`crates/recipe/src/lib.rs`)
- Reuses existing `create_recipe()` command per recipe (DRY principle)
- Free tier limit enforced at domain level (same as single recipe creation)

**Server-Side Rendering:**
- Modal rendered server-side via Askama templates
- TwinSpark provides AJAX form submission (progressive enhancement)
- No JavaScript required for core functionality (graceful degradation)

**Multipart Form Handling:**
- Axum's `Multipart` extractor for file uploads
- File contents read as text, parsed as JSON
- Server-side validation only (no client-side validation required)

### Technical Decisions

**Why no rollback for partial failures?**
- Simplicity: Avoids complex transaction management across evento events
- User benefit: Partially successful imports still save work
- Clear feedback: User sees exactly which recipes failed and why
- Retry-friendly: Failed recipes can be fixed and re-imported

**Why reuse create_recipe() instead of batch insert?**
- DRY: Leverages existing validation, business logic, event handling
- Consistency: Same behavior as manual recipe creation
- Maintainability: Single source of truth for recipe creation rules
- Trade-off: Slightly slower than batch insert, but acceptable for MVP

**Why JSON array format?**
- Consistency: Follows existing `example-recipe.json` structure
- Simplicity: Users can copy example and add more recipes
- Extensibility: Future could support CSV, YAML with format detection

### Error Handling Scenarios

| Scenario | Status | User Message |
|----------|--------|--------------|
| Valid import (all succeed) | 200 OK | "3 recipes imported successfully!" |
| Invalid JSON syntax | 422 | "Invalid JSON format. Please check your file syntax." |
| Not an array | 422 | "Expected array of recipes. Root element must be [...]" |
| Empty array | 422 | "No recipes found in file" |
| Missing required field | 200 OK + partial | "Recipe #2: Missing required field 'title'" (in failures list) |
| Invalid recipe_type | 200 OK + partial | "Recipe #3: recipe_type must be 'appetizer', 'main_course', or 'dessert'" |
| Free tier limit exceeded | 403 | "Import would exceed free tier limit. You have 9/10 recipes. Attempting to import 3 more would exceed the limit." |
| Database error | 500 | "Server error during import. Please try again." |

### Testing Strategy

**Unit Tests (crates/recipe/tests/batch_import_tests.rs):**
- Focus: Business logic in `batch_import_recipes()` function
- Coverage: Validation, free tier enforcement, partial success, error collection
- Mocking: In-memory evento executor, mock user tier queries

**Integration Tests (tests/batch_import_integration_tests.rs):**
- Focus: HTTP route handler + domain integration
- Coverage: Multipart form parsing, JSON validation, response rendering
- Setup: In-memory SQLite database, authenticated test user

**E2E Tests (e2e/tests/batch-import.spec.ts) - Optional:**
- Focus: Full user flow from button click to recipe list refresh
- Coverage: Modal interaction, file upload, results display
- Priority: Medium (integration tests provide adequate coverage)

### Deployment Checklist

**Pre-Deployment:**
- [ ] All unit tests pass: `cargo test -p recipe batch_import`
- [ ] All integration tests pass: `cargo test batch_import_integration`
- [ ] Manual testing completed (all scenarios in Phase 4)
- [ ] Accessibility audit passed (keyboard nav, screen reader)
- [ ] Code review completed (if team workflow)

**Deployment:**
- [ ] No database migrations required ✅
- [ ] Feature flag optional: `batch_import_enabled: true` in config
- [ ] Deploy via existing CI/CD pipeline
- [ ] Verify "Import Recipes" button visible in production

**Rollback Plan:**
- Option 1: Hide button via feature flag
- Option 2: Disable `/recipes/import` route (503)
- Option 3: Rollback Docker image: `kubectl rollout undo deployment/imkitchen`

**Monitoring (OpenTelemetry):**
- Metrics: `batch_import_requests_total`, `batch_import_success_count`, `batch_import_failure_count`, `batch_import_duration_seconds`
- Alerts: Failure rate > 50%, import duration > 10 seconds

### Dependencies

**Existing Dependencies (No New Ones):**
- `serde_json` 1.0+ (JSON parsing)
- `validator` 0.20+ (validation)
- `evento` 1.3+ (event sourcing)
- `axum` 0.8+ (HTTP server, multipart forms)
- `askama` 0.14+ (templates)
- `sqlx` 0.8+ (database queries)

**No External Services Required** ✅

### Story Context

**Epic**: Epic 2 - Recipe Management
**Depends On**: Story 2.1 (Create Recipe) - DONE ✅
**Blocks**: None
**Related Stories**: Story 2.2 (Edit Recipe), Story 2.3 (Delete Recipe)

**Priority**: Medium
**Complexity**: Medium (3-4 hours backend, 2-3 hours frontend, 1-2 hours testing)
**Total Estimate**: 8-10 hours

---

_This story follows the imkitchen project conventions: TDD enforced, evento event sourcing, Askama server-rendered templates, TwinSpark progressive enhancement, Tailwind CSS styling._
