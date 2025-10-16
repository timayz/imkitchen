# Story 2.11: Tech Debt & Enhancements

Status: Approved

## Story

As a **development team**,
I want **to address deferred technical improvements from Stories 2.1 and 1.7**,
so that **code quality, test coverage, and documentation meet production standards**.

## Acceptance Criteria

1. **[Story 2.1 - HIGH]** Instruction reordering UI implemented with drag handles or up/down arrows
   - Users can reorder instruction steps in recipe form
   - Step numbers update dynamically after reordering
   - TwinSpark or minimal JavaScript provides smooth UX
   - Changes persist on form submission

2. **[Story 2.1 - HIGH]** Complete test suite written: unit tests for RecipeAggregate, integration tests for HTTP routes, E2E tests for recipe creation flow with Playwright
   - Unit tests: RecipeAggregate event handlers and validation logic (`crates/recipe/tests/recipe_tests.rs`)
   - Integration tests: HTTP routes (GET /recipes/new, POST /recipes, GET /recipes/:id) with database projections (`tests/recipe_integration_tests.rs`)
   - E2E tests: Full recipe creation flow from form submission to detail page display (`e2e/tests/recipe.spec.ts`)
   - Target 80% code coverage via `cargo tarpaulin`

3. **[Story 2.1 - MEDIUM]** Form parsing refactored to use Axum extractors
   - Replace `parse_recipe_form()` function with `Form<CreateRecipeForm>` extractor in `src/routes/recipes.rs:292-340`
   - Implement custom deserializer for array fields (ingredient_name[], ingredient_quantity[], ingredient_unit[], instruction_text[])
   - Remove manual URL decoding logic
   - Validation errors returned via Axum's structured error handling

4. **[Story 2.1 - MEDIUM]** Structured error handling implemented with AppError enum
   - Define `AppError` enum with variants: DatabaseError, ValidationError, EventStoreError, RecipeLimitError
   - Implement `IntoResponse` trait for AppError to render user-friendly error pages
   - Map domain errors (RecipeError) to HTTP status codes (422 Unprocessable Entity, 403 Forbidden, 500 Internal Server Error)
   - Update all recipe routes to use AppError instead of generic Result types

5. **[Story 1.7 - LOW]** Stripe setup guide documented in `docs/stripe-setup.md` or README
   - Document how to obtain Stripe test API keys
   - Document how to create Stripe Price object for $9.99/month subscription
   - Document webhook registration and endpoint secret configuration
   - Link from main README.md to setup guide

6. All tests pass in CI/CD pipeline
   - `cargo test` passes all unit and integration tests
   - `playwright test` passes all E2E tests
   - No test failures in GitHub Actions workflow

7. Code coverage metrics meet or exceed 80% target (NFR requirement)
   - Run `cargo tarpaulin` to generate coverage report
   - Recipe domain crate achieves ≥80% line coverage
   - Recipe routes achieve ≥80% branch coverage
   - Coverage report uploaded to CI artifacts

8. Documentation reviewed and approved by tech lead
   - Stripe setup guide reviewed for accuracy and completeness
   - Code refactoring changes reviewed for maintainability
   - Test coverage gaps identified and documented if any

## Tasks / Subtasks

### Task 1: Implement Instruction Reordering UI (AC-1)
- [ ] Add drag handle icons to instruction rows in `templates/pages/recipe-form.html`
- [ ] Implement JavaScript drag-and-drop logic or up/down arrow buttons
- [ ] Update step numbers dynamically on reorder (client-side renumbering)
- [ ] Ensure form submission includes correct step_number values
- [ ] Test: Verify instruction order persists after save

### Task 2: Write Complete Test Suite (AC-2)
- [ ] Write unit tests for RecipeAggregate in `crates/recipe/tests/recipe_tests.rs`:
  - [ ] `test_recipe_created_event_initializes_aggregate()`
  - [ ] `test_recipe_validation_rejects_empty_title()`
  - [ ] `test_recipe_validation_requires_at_least_one_ingredient()`
  - [ ] `test_recipe_validation_requires_at_least_one_instruction()`
- [ ] Write integration tests in `tests/recipe_integration_tests.rs`:
  - [ ] `test_get_recipe_new_form_returns_200()`
  - [ ] `test_post_recipe_creates_and_redirects_to_detail()`
  - [ ] `test_post_recipe_fails_with_422_on_validation_error()`
  - [ ] `test_get_recipe_detail_returns_200_for_owner()`
  - [ ] `test_recipe_created_event_updates_read_model()`
- [ ] Write E2E tests in `e2e/tests/recipe.spec.ts`:
  - [ ] `test('user can create recipe with all fields', async ({ page }) => { ... })`
  - [ ] `test('recipe creation validates required fields', async ({ page }) => { ... })`
  - [ ] `test('created recipe displays on detail page', async ({ page }) => { ... })`
- [ ] Run `cargo tarpaulin --out Html` and verify 80% coverage
- [ ] Document coverage gaps in PR description if <80%

### Task 3: Refactor Form Parsing to Axum Extractors (AC-3)
- [ ] Define `CreateRecipeForm` struct in `src/routes/recipes.rs` with serde derives
- [ ] Implement custom deserializer for array fields (ingredient_name[], etc.)
- [ ] Replace `parse_recipe_form()` function with `Form<CreateRecipeForm>` in POST /recipes handler
- [ ] Remove manual URL decoding logic
- [ ] Update validation to use `validator` crate on CreateRecipeForm struct
- [ ] Test: Submit recipe form and verify parsing works correctly

### Task 4: Implement Structured Error Handling (AC-4)
- [ ] Create `src/error.rs` with AppError enum (DatabaseError, ValidationError, EventStoreError, RecipeLimitError)
- [ ] Implement `IntoResponse` for AppError with user-friendly HTML error pages
- [ ] Implement `From<RecipeError>` for AppError to map domain errors
- [ ] Update recipe routes to return `Result<Response, AppError>` instead of generic Result
- [ ] Add structured logging for errors with tracing::error!
- [ ] Test: Trigger each error variant and verify correct HTTP status and user message

### Task 5: Document Stripe Setup Guide (AC-5)
- [ ] Create `docs/stripe-setup.md` with sections:
  - [ ] "1. Create Stripe Account and Get Test Keys"
  - [ ] "2. Create Price Object for $9.99/month Subscription"
  - [ ] "3. Configure Webhook Endpoint and Secret"
  - [ ] "4. Set Environment Variables"
- [ ] Add screenshots or code examples where helpful
- [ ] Link from main README.md in "Getting Started" section
- [ ] Review: Tech lead verifies accuracy and completeness

### Task 6: Verify CI/CD Pipeline (AC-6)
- [ ] Ensure `.github/workflows/ci.yml` runs `cargo test`
- [ ] Ensure `.github/workflows/ci.yml` runs `playwright test`
- [ ] Add `cargo tarpaulin` step to CI workflow (upload coverage report as artifact)
- [ ] Run full CI pipeline locally and verify all tests pass
- [ ] Fix any failing tests discovered during CI run

### Task 7: Achieve 80% Code Coverage (AC-7)
- [ ] Run `cargo tarpaulin --workspace --out Html` to generate coverage report
- [ ] Review coverage report and identify untested code paths
- [ ] Add additional tests to reach 80% threshold for recipe crate and routes
- [ ] Document any intentionally untested code (e.g., unreachable error branches)
- [ ] Upload coverage report to CI artifacts for review

### Task 8: Tech Lead Review and Approval (AC-8)
- [ ] Submit PR with all changes and test results
- [ ] Tech lead reviews Stripe documentation
- [ ] Tech lead reviews code refactoring for maintainability
- [ ] Tech lead reviews test coverage report
- [ ] Address any feedback from tech lead
- [ ] Obtain approval and merge PR

## Dev Notes

### Architecture Patterns and Constraints

**Testing Strategy:**
- **Unit Tests**: Focus on domain logic in `crates/recipe/` - event handlers, validation, business rules
- **Integration Tests**: Verify HTTP routes, database projections, and evento event subscriptions work end-to-end
- **E2E Tests**: Use Playwright to test user journeys from browser perspective (form submission, page navigation, data persistence)
- **Coverage Target**: 80% line coverage per NFR requirements (specified in PRD Non-Functional Requirements)

**Form Handling Best Practices:**
- Axum `Form<T>` extractor provides type-safe form parsing with automatic validation
- Use `serde` derives for deserialization and `validator` derives for field-level validation
- Custom deserializers needed for HTML array inputs (e.g., `ingredient_name[]` → Vec<String>)
- Reference: [Axum forms documentation](https://docs.rs/axum/latest/axum/extract/struct.Form.html)

**Error Handling Patterns:**
- AppError enum centralizes error handling across all routes
- `IntoResponse` implementation maps errors to appropriate HTTP status codes and user-friendly HTML pages
- Domain errors (e.g., RecipeError::RecipeLimitExceeded) map to HTTP 422 Unprocessable Entity
- Infrastructure errors (e.g., DatabaseError) map to HTTP 500 Internal Server Error
- Reference: Story 1.5 uses similar error handling pattern for user profile routes

**TwinSpark Progressive Enhancement:**
- Instruction reordering can use TwinSpark `ts-action` attribute for AJAX-style updates
- Alternative: Minimal vanilla JavaScript for drag-and-drop without external libraries
- Ensure graceful degradation: form submission works without JavaScript enabled

### Source Tree Components to Touch

**Files to Modify:**
- `templates/pages/recipe-form.html` - Add instruction reordering UI (drag handles or arrows)
- `src/routes/recipes.rs` - Refactor form parsing, add AppError handling
- `src/error.rs` - NEW FILE - Define AppError enum and IntoResponse impl
- `crates/recipe/tests/recipe_tests.rs` - Add unit tests for RecipeAggregate
- `tests/recipe_integration_tests.rs` - Add integration tests for HTTP routes
- `e2e/tests/recipe.spec.ts` - NEW FILE - Add E2E Playwright tests
- `.github/workflows/ci.yml` - Add cargo tarpaulin coverage step
- `docs/stripe-setup.md` - NEW FILE - Stripe setup guide
- `README.md` - Add link to Stripe setup guide

**Files to Reference:**
- `docs/stories/story-2.1.md` - Action items source (lines 463-492)
- `docs/stories/story-1.7.md` - Stripe setup instructions (Completion Notes section)
- `src/routes/profile.rs` - Example of AppError pattern (Story 1.5)
- `crates/user/tests/aggregate_tests.rs` - Example unit test structure

### Project Structure Notes

**Alignment with unified project structure:**
- Tests organized per Rust conventions: unit tests in `crates/*/tests/`, integration tests in `/tests/`, E2E in `/e2e/`
- Error handling centralized in `src/error.rs` following DRY principle
- Documentation in `/docs/` directory (Stripe guide joins existing tech specs)
- No conflicts detected with existing architecture patterns

**Testing Infrastructure:**
- Cargo workspace supports shared test utilities via `[dev-dependencies]`
- Playwright configured in `/e2e/` with TypeScript (existing E2E framework from Story 1.7)
- Tarpaulin coverage tool generates HTML reports for local review and CI artifacts

### References

- **Source**: `docs/stories/story-2.1.md` lines 463-492 (Action Items section)
- **Source**: `docs/stories/story-1.7.md` lines 524-532 (Documentation - PENDING)
- **Architecture**: `docs/solution-architecture.md` section 1.1 (Testing stack: Playwright for E2E)
- **PRD**: Non-Functional Requirements specify 80% code coverage target
- **Tech Spec Epic 2**: `docs/tech-spec-epic-2.md` section on Recipe Management System implementation patterns

## Dev Agent Record

### Context Reference

- `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-2.11.xml`

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
