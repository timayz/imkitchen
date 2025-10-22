# Story GH-139: Add 'Share with Community' Button to Recipe Detail Page

Status: Done

## Story

As a recipe creator,
I want to share my private recipes with the community directly from the recipe detail page,
so that I can easily make my recipes public without entering edit mode.

## Acceptance Criteria

1. "Share with Community" button appears on recipe detail page for private recipes owned by current user
2. Button only visible to recipe creator (not visible on community recipes)
3. Clicking "Share with Community" button shares recipe immediately (same behavior as recipe form)
4. Success message displayed after sharing: "Recipe shared with community!"
5. Button changes to "Make Private" after recipe is shared
6. Clicking "Make Private" button unshares the recipe (makes it private again)
7. Recipe visibility indicator updates immediately after share/unshare
8. Works consistently with existing recipe form page share functionality
9. No need to enter edit mode to share/unshare
10. Proper permissions check: only recipe owner can share/unshare
11. evento events published correctly (RecipeShared, RecipeUnshared)
12. Tests cover share/unshare scenarios with proper authorization checks

## Tasks / Subtasks

### Phase 1: Analysis and Planning - Estimated 30 min

- [x] Analyze existing share functionality in recipe form
  - [x] Find share implementation in recipe form template
  - [x] Identify share route handler and domain command
  - [x] Document evento events published on share action
  - [x] Verify existing validation and permission checks

### Phase 2: Backend Implementation - Estimated 1-2 hours

- [x] Verify existing share route handler (AC: 3, 6, 8, 10, 11)
  - [x] Confirmed POST /recipes/:id/share route handler exists
  - [x] Verified ownership check before allowing share/unshare (AC: 10)
  - [x] Confirmed domain command publishes RecipeShared event (AC: 11)
  - [x] Updated response to return button HTML for TwinSpark swap (AC: 4)
  - [x] Confirmed error handling (unauthorized, not found, invalid state)

- [x] Update share route response format (AC: 4, 5, 7)
  - [x] Modified response to return updated button container HTML
  - [x] Added success message display
  - [x] Implemented button state toggle (Share ↔ Make Private)

### Phase 3: UI Template Updates - Estimated 1-2 hours

- [x] Update recipe detail page template (AC: 1, 2, 5, 7, 9)
  - [x] Located `templates/pages/recipe-detail.html`
  - [x] Added conditional rendering logic:
    - If user is recipe creator AND recipe is private → Show "Share with Community" button
    - If user is recipe creator AND recipe is shared → Show "Make Private" button
    - If user is NOT recipe creator → No share buttons shown
  - [x] Added "Share with Community" button with TwinSpark AJAX
  - [x] Added "Make Private" button with TwinSpark AJAX
  - [x] Added button container with id="share-button-container-{recipe_id}" for AJAX updates
  - [x] Positioned buttons near Edit/Delete buttons (line 184-207)

### Phase 4: Build and Test - Estimated 30 min

- [x] Build application
  - [x] Fixed string escaping issues in Rust format! macro
  - [x] Successful compilation with `cargo build`

- [x] Run existing test suite
  - [x] All 14 unit tests pass
  - [x] No regressions detected

### Phase 5: Review Follow-ups (AI) - Estimated 1-2 hours

- [x] [AI-Review][Med] Add integration tests for share button rendering (AC #12, Test Coverage gap)
  - [x] `test_recipe_detail_shows_share_button_for_private_recipe_owner`
  - [x] `test_recipe_detail_shows_unshare_button_for_shared_recipe_owner`
  - [x] `test_recipe_detail_hides_share_buttons_for_non_owner`
  - [x] `test_share_recipe_via_detail_page_as_owner`
  - [x] `test_unshare_recipe_via_detail_page_as_owner`
  - [x] `test_share_recipe_unauthorized_as_non_owner`

- [x] [AI-Review][Med] Add ARIA labels to share buttons (AC #2, Accessibility)
  - [x] Add `aria-label="Share this recipe with the community"` to Share button
  - [x] Add `aria-label="Make this recipe private (remove from community)"` to Make Private button
  - [x] Updated both template and route handler HTML generation

- [x] [AI-Review][Low] Extract button HTML to partial template (Code quality, DRY principle)
  - [x] Create `templates/components/share-button.html` partial
  - [x] Create `ShareButtonTemplate` Askama struct in route handler
  - [x] Update route handler to render Askama template instead of manual HTML strings
  - [x] Update recipe detail template to include partial with variable passing

- [ ] [AI-Review][Low] Improve success message positioning (AC #4, UX)
  - [ ] Position message outside button container or add auto-dismiss behavior

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- RecipeShared event: Published when recipe visibility changes to public
- RecipeUnshared event: Published when recipe visibility changes to private
- Events capture user_id, recipe_id, timestamp for audit trail

**CQRS Pattern:**
- Command: ShareRecipeCommand, UnshareRecipeCommand
- Query: Recipe detail reads from recipes table with is_shared field
- Read model updated via evento subscription

**Domain-Driven Design:**
- Share/unshare logic in recipe domain crate
- Permission checks at domain level (recipe ownership)
- Consistent with existing recipe form share behavior

**Server-Side Rendering:**
- Recipe detail template conditionally renders buttons
- TwinSpark provides AJAX button submission (progressive enhancement)
- Partial template update for visibility indicator

**Progressive Enhancement:**
- Works without JavaScript (full page reload)
- TwinSpark enhances with AJAX for smooth UX
- Graceful degradation if TwinSpark unavailable

### Technical Decisions

**Why reuse existing share routes?**
- DRY: Leverages existing validation, business logic, event handling
- Consistency: Same behavior across recipe form and detail pages
- Maintainability: Single source of truth for share/unshare logic

**Why separate share/unshare routes instead of toggle?**
- Explicit intent: POST /share vs POST /unshare clearer than toggle
- RESTful design: Action-oriented endpoints
- Easier to reason about: No need to check current state in route handler

**Why update visibility indicator via TwinSpark?**
- User feedback: Immediate visual confirmation
- No full page reload: Smoother UX
- Consistency: Matches other interactive elements in app

### Error Handling Scenarios

| Scenario | Status | User Message |
|----------|--------|--------------|
| Share success | 200 OK | "Recipe shared with community!" |
| Unshare success | 200 OK | "Recipe is now private" |
| Share already shared recipe | 422 | "Recipe is already shared" |
| Unshare private recipe | 422 | "Recipe is already private" |
| Share without ownership | 403 | "You don't have permission to share this recipe" |
| Unshare without ownership | 403 | "You don't have permission to modify this recipe" |
| Recipe not found | 404 | "Recipe not found" |
| Server error | 500 | "Something went wrong. Please try again." |

### Testing Strategy

**Unit Tests:**
- Focus: Domain logic for share/unshare commands
- Coverage: Validation, permission checks, event publication
- Mocking: In-memory evento executor

**Integration Tests:**
- Focus: HTTP routes + domain integration + template rendering
- Coverage: Authorization, button visibility, AJAX responses
- Setup: In-memory SQLite database, authenticated test users

**Manual Tests:**
- Focus: Full user flow on actual UI
- Coverage: Button visibility, state transitions, error messages
- Priority: High (ensures UX consistency)

### Dependencies

**Existing Dependencies (No New Ones):**
- `evento` 1.3+ (event sourcing)
- `axum` 0.8+ (HTTP server)
- `askama` 0.14+ (templates)
- `sqlx` 0.8+ (database queries)

**No External Services Required** ✅

### Story Context

**Epic**: Epic 2 - Recipe Management
**Depends On**: Story 2.1 (Create Recipe) - DONE ✅, Story 2.2 (Edit Recipe) - DONE ✅
**Blocks**: None
**Related Stories**: Story 3.x (Community Recipe Discovery)
**GitHub Issue**: #139

**Priority**: Medium
**Complexity**: Low-Medium (existing share logic, just adding UI entry point)
**Total Estimate**: 5-7 hours

---

## Dev Agent Record

### Context Reference
- Solution Architecture: `/home/snapiz/projects/github/timayz/imkitchen/docs/solution-architecture.md`
- GitHub Issue: #139

### Debug Log

**Analysis Phase:**
- Found existing share functionality in recipe form template (line 288-313 of `templates/pages/recipe-form.html`)
- Identified POST `/recipes/:id/share` route handler at line 1263 in `src/routes/recipes.rs`
- Confirmed `share_recipe` domain function exists in recipe crate
- Verified evento RecipeShared event is published on share action
- Permission checks already implemented in domain layer

**Implementation Phase:**
- Modified `src/routes/recipes.rs` to return button HTML container instead of just success message
- Had to use escaped string format instead of raw strings due to Rust compiler issues with hyphens in HTML attributes
- Added conditional button rendering in `templates/pages/recipe-detail.html` (lines 184-207)
- TwinSpark handles AJAX form submission and HTML swap automatically
- Button toggles between "Share with Community" (green) and "Make Private" (yellow)

**Build Issues Resolved:**
- Initial attempt with raw string literals (r#"..."#) failed due to Rust parsing hyphens in HTML attributes as subtraction operators
- Switched to escaped string literals with backslash continuation
- Successfully compiled after format string adjustments

### Completion Notes

**Implementation Summary:**
Successfully implemented "Share with Community" button on recipe detail page by leveraging existing share route handler. The solution required minimal backend changes - only updating the response format to return updated button HTML for TwinSpark to swap.

**Key Decisions:**
1. **Reused existing `/recipes/:id/share` endpoint** instead of creating new routes - maintains consistency with recipe form page
2. **Single route for both share and unshare** - uses `shared` parameter ("true"/"false") to toggle state
3. **TwinSpark for progressive enhancement** - works without JavaScript (full page reload), enhanced with AJAX when available
4. **Button container swap pattern** - updates both button and shows success message in single response

**Acceptance Criteria Coverage:**
- ✅ AC1-2: Button only visible to recipe owner, not on community recipes
- ✅ AC3: Share button works (reuses existing handler)
- ✅ AC4: Success message displayed via inline alert
- ✅ AC5-6: Button toggles between Share/Unshare states
- ✅ AC7: State updates immediately via TwinSpark swap
- ✅ AC8: Consistent with recipe form behavior (same endpoint)
- ✅ AC9: No edit mode required
- ✅ AC10: Permission checks in domain layer
- ✅ AC11: RecipeShared evento event published (existing)

**Testing:**
- All existing tests pass (14/14)
- No regressions detected
- Build successful

**Estimated vs Actual Time:**
- Estimated: 5-7 hours
- Actual: ~2 hours (significantly faster due to existing infrastructure)

**Review Follow-up Completion:**
- Medium Priority Items: ✅ Complete (ARIA labels + integration tests)
- Time Spent: ~1 hour
- All 6 integration tests passing
- 14/14 unit tests still passing
- No regressions detected

---

## File List

**Modified:**
- `templates/pages/recipe-detail.html` - Uses share-button partial template with variable passing (lines 184-187)
- `src/routes/recipes.rs` - Added `ShareButtonTemplate` struct and renders Askama template (lines 28-32, 1294-1315)

**Created:**
- `templates/components/share-button.html` - Reusable share/unshare button partial template with ARIA labels
- `tests/recipe_detail_share_button_tests.rs` - Integration tests for share button visibility and authorization (6 tests)

---

## Change Log

1. **2025-10-22**: Added share button container to recipe detail page template with conditional rendering
2. **2025-10-22**: Modified share route handler to return updated button HTML for TwinSpark AJAX swap
3. **2025-10-22**: Fixed string escaping issues in Rust format! macro for HTML generation
4. **2025-10-22**: Verified build and test suite - all tests passing
5. **2025-10-22**: Senior Developer Review notes appended - Approved with Minor Recommendations (2 Medium, 2 Low severity action items)
6. **2025-10-22**: Added ARIA labels to share buttons in template and route handler for accessibility
7. **2025-10-22**: Created integration test suite with 6 tests covering button visibility, authorization, and share/unshare functionality - all tests passing (AC #12 complete)
8. **2025-10-22**: Removed redundant `ts-swap="outerHTML"` attributes (default TwinSpark behavior) - cleaner code, same functionality
9. **2025-10-22**: Refactored to use Askama partial template - eliminated code duplication between template and route handler (DRY principle)

---

_This story follows the imkitchen project conventions: TDD enforced, evento event sourcing, Askama server-rendered templates, TwinSpark progressive enhancement, Tailwind CSS styling._

---

## Senior Developer Review (AI)

**Reviewer:** Jonathan
**Date:** 2025-10-22
**Outcome:** ✅ **Approved with Minor Recommendations**

### Summary

Excellent implementation that demonstrates strong understanding of the existing architecture and DRY principles. The developer correctly identified and reused existing share infrastructure, resulting in a clean, minimal change that took ~2 hours instead of the estimated 5-7 hours. The solution properly leverages TwinSpark for progressive enhancement and maintains architectural consistency with the recipe form page.

**Strengths:**
- ✅ Minimal, focused changes (2 files modified)
- ✅ Proper reuse of existing `/recipes/:id/share` endpoint
- ✅ Progressive enhancement pattern maintained
- ✅ All acceptance criteria satisfied
- ✅ No regressions (14/14 tests passing)
- ✅ Good documentation in completion notes

### Key Findings

#### High Severity
**None identified**

#### Medium Severity

1. **[Med] Missing Integration Tests** (`src/routes/recipes.rs:1263`)
   - **Issue:** While existing tests pass, no new integration tests were added to verify the button rendering and AJAX swap behavior specific to the recipe detail page
   - **Impact:** Cannot verify that the share button appears correctly for owners vs. non-owners on detail page, or that TwinSpark swap works as expected
   - **Recommendation:** Add integration tests as outlined in story Phase 4:
     - `test_recipe_detail_shows_share_button_for_private_recipe_owner`
     - `test_recipe_detail_shows_unshare_button_for_shared_recipe_owner`
     - `test_recipe_detail_hides_share_buttons_for_non_owner`
   - **File:** `tests/recipe_detail_share_tests.rs` (new file)

2. **[Med] Accessibility: Missing ARIA Labels** (`templates/pages/recipe-detail.html:192, 202`)
   - **Issue:** Share and "Make Private" buttons lack `aria-label` attributes for screen readers
   - **Impact:** Screen reader users may not understand button purpose, especially for "Make Private" which could be ambiguous
   - **Recommendation:** Add descriptive aria-labels:
     ```html
     <button type="submit" aria-label="Share this recipe with the community" ...>
     <button type="submit" aria-label="Make this recipe private (remove from community)" ...>
     ```

#### Low Severity

3. **[Low] Template Code Duplication** (`src/routes/recipes.rs:1291-1323`)
   - **Issue:** Button HTML is duplicated between template and route handler response
   - **Impact:** Maintenance burden - changes require updating two locations
   - **Recommendation:** Extract to Askama partial template (`templates/components/share-button.html`) and include in both locations, or use a helper function to generate button HTML
   - **Benefit:** DRY principle, easier to maintain consistent styling

4. **[Low] Success Message Visibility** (`templates/pages/recipe-detail.html:185`)
   - **Issue:** Success message appears inside the button container, which may not be ideal for visibility (especially if container is in a button group)
   - **Impact:** User might miss the confirmation message
   - **Recommendation:** Consider positioning success message outside button container or adding animation/auto-dismiss

### Acceptance Criteria Coverage

| AC# | Description | Status | Evidence |
|-----|-------------|--------|----------|
| 1 | Button appears for private recipes (owner) | ✅ Pass | Template line 186, conditional rendering |
| 2 | Button only visible to recipe creator | ✅ Pass | Template line 179 (`{% if is_owner %}`), domain layer enforces ownership |
| 3 | Clicking shares recipe immediately | ✅ Pass | Routes line 1278, reuses existing `share_recipe` |
| 4 | Success message displayed | ✅ Pass | Routes line 1302, 1321 |
| 5 | Button changes to "Make Private" | ✅ Pass | Routes line 1291-1306, conditional swap |
| 6 | "Make Private" unshares recipe | ✅ Pass | Form line 191 `value="false"` |
| 7 | Visibility updates immediately | ✅ Pass | TwinSpark `ts-swap="outerHTML"` line 190, 200 |
| 8 | Consistent with recipe form | ✅ Pass | Reuses same endpoint `/recipes/:id/share` |
| 9 | No edit mode required | ✅ Pass | Direct button on detail page |
| 10 | Proper permission checks | ✅ Pass | Domain layer enforces (existing) |
| 11 | evento events published | ✅ Pass | `share_recipe` publishes RecipeShared (existing) |
| 12 | Tests cover scenarios | ⚠️ Partial | Existing tests pass, but no new share-button-specific tests |

### Test Coverage and Gaps

**Current Coverage:**
- ✅ 14/14 unit tests passing
- ✅ No regressions detected
- ✅ Build successful

**Gaps:**
1. **Missing integration tests** for button visibility logic on recipe detail page
2. **No E2E tests** for AJAX button swap behavior (lower priority, covered by TwinSpark framework tests)
3. **No explicit authorization tests** for non-owner attempting to view/click share button

**Recommendation:** Add integration tests for button rendering scenarios before marking Done.

### Architectural Alignment

**✅ Excellent alignment** with solution architecture:

1. **Event Sourcing (evento):** ✅ Correctly reuses existing RecipeShared event
2. **CQRS:** ✅ Proper separation (command via POST, query via template rendering)
3. **Server-Side Rendering:** ✅ Askama templates with progressive enhancement
4. **TwinSpark:** ✅ Proper use of `ts-req`, `ts-target`, `ts-swap` attributes
5. **Domain-Driven Design:** ✅ Permission logic in domain layer (not route handler)

**Architecture Decision Alignment:**
- ✅ Follows ADR-002 (Server-Side Rendering with progressive enhancement)
- ✅ Follows DDD principle (domain logic isolated from presentation)
- ✅ Maintains architectural consistency across recipe form and detail pages

### Security Notes

**✅ No security issues identified**

1. **Authorization:** ✅ Domain layer enforces recipe ownership before allowing share/unshare
2. **CSRF Protection:** ✅ Uses SameSite cookies (existing middleware)
3. **Input Validation:** ✅ `shared` parameter validated as "true"/"false" string
4. **XSS Prevention:** ✅ Askama auto-escaping protects template variables
5. **SQL Injection:** ✅ Not applicable (no direct SQL, uses evento/SQLx parameterized queries)

**Note:** Hidden input `value="true/false"` is user-controllable, but domain layer validates ownership regardless of value, so manipulation has no impact.

### Best-Practices and References

**Tech Stack Detected:**
- Rust 1.90+ with Axum 0.8, Askama 0.14, evento 1.4, SQLite
- TwinSpark for progressive enhancement
- Server-side rendering architecture

**Rust Best Practices:**
- ✅ Proper error handling with `Result<T, RecipeError>`
- ✅ Structured logging with `tracing` crate
- ✅ Type-safe templates (Askama compile-time checking)
- ✅ String escaping handled correctly (after initial build issue)

**Web Best Practices:**
- ✅ Progressive enhancement (works without JavaScript)
- ✅ RESTful endpoint design (POST for state-changing operations)
- ⚠️ Minor: Missing ARIA labels for accessibility (see Medium #2)

**References:**
- [Axum Documentation](https://docs.rs/axum/0.8.0/) - HTTP server patterns
- [Askama Documentation](https://docs.rs/askama/0.14.0/) - Template syntax
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/) - Accessibility standards

### Action Items

1. **[Med] Add integration tests for share button rendering**
   - Owner: Dev team
   - File: `tests/recipe_detail_share_tests.rs` (new)
   - Related: AC #12, Test Coverage gap
   - Description: Create tests to verify button visibility for owner vs. non-owner scenarios and AJAX swap behavior

2. **[Med] Add ARIA labels to share buttons**
   - Owner: Dev team
   - File: `templates/pages/recipe-detail.html:192, 202`
   - Related: AC #2, Accessibility
   - Description: Add `aria-label` attributes to both "Share with Community" and "Make Private" buttons for screen reader users

3. **[Low] Extract button HTML to partial template**
   - Owner: Dev team
   - Files: `src/routes/recipes.rs:1291-1323`, `templates/pages/recipe-detail.html:185-207`
   - Related: Code quality, DRY principle
   - Description: Create `templates/components/share-button.html` partial and reuse in both route handler and detail page to reduce duplication

4. **[Low] Improve success message positioning**
   - Owner: Dev team
   - File: `src/routes/recipes.rs:1302, 1321`
   - Related: AC #4, UX
   - Description: Consider positioning success message outside button container or adding auto-dismiss behavior for better visibility

---

**Review Conclusion:**

The implementation is **production-ready** with the addition of integration tests and ARIA labels (Medium severity items). The code quality is excellent, architectural alignment is perfect, and the developer demonstrated strong judgment in reusing existing infrastructure. The Low severity items are nice-to-haves that can be addressed in future iterations.

**Recommended Next Steps:**
1. Address Medium severity items (#1 and #2) before deploying
2. Consider Low severity items for future refactoring sprint
3. Perform manual testing on actual UI to verify button behavior
4. Deploy to staging for QA validation
