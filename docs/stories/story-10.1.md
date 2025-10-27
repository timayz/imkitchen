# Story 10.1: End-to-End Testing with Playwright

Status: Approved

## Story

As a QA engineer,
I want to implement a comprehensive E2E test suite using Playwright,
so that all critical user flows for the enhanced meal planning system are validated before production deployment.

## Acceptance Criteria

1. **AC1**: Playwright test suite configured in `e2e/` directory with TypeScript fixtures for authenticated sessions
   - Verify: `e2e/playwright.config.ts` exists with parallel workers (4), base URL, auth storage path
   - Verify: `e2e/fixtures/auth.ts` provides `authenticatedPage` fixture with JWT cookie

2. **AC2**: Test coverage for multi-week meal plan generation flow
   - Verify: `e2e/tests/meal-planning.spec.ts` includes test: "User generates multi-week meal plan (4 weeks)"
   - Verify: Test navigates to `/plan`, submits generation form, asserts success redirect

3. **AC3**: Test coverage for week navigation
   - Verify: Test "User navigates between weeks" clicks "Next Week" button
   - Verify: Calendar updates to show next Monday-Sunday range

4. **AC4**: Test coverage for single week regeneration
   - Verify: Test "User regenerates single week" clicks "Regenerate This Week" button
   - Verify: Meal assignments change for that week only (other weeks unchanged)

5. **AC5**: Test coverage for all future weeks regeneration
   - Verify: Test "User regenerates all future weeks" clicks "Regenerate All Future Weeks" button
   - Verify: All weeks from current week forward are regenerated

6. **AC6**: Test coverage for meal planning preferences update
   - Verify: `e2e/tests/preferences.spec.ts` navigates to `/profile/meal-planning-preferences`
   - Verify: Test toggles checkboxes (breakfast, lunch, dinner, side dishes), submits form
   - Verify: Preferences saved (next meal plan generation respects preferences)

7. **AC7**: Test coverage for recipe creation with accompaniment settings
   - Verify: `e2e/tests/recipes.spec.ts` navigates to `/recipes/new`
   - Verify: Test fills form with `can_be_side_dish` and `needs_side_dish` checkboxes
   - Verify: Recipe saved with correct accompaniment settings

8. **AC8**: Test coverage for shopping list access for specific week
   - Verify: `e2e/tests/shopping.spec.ts` navigates to `/shopping?week=2025-11-10`
   - Verify: Shopping list displays ingredients for that week only

9. **AC9**: All E2E tests pass in CI pipeline
   - Verify: GitHub Actions workflow executes `npx playwright test` successfully
   - Verify: Exit code 0 (all tests passed)

10. **AC10**: Test execution time <5 minutes
    - Verify: GitHub Actions job duration for E2E tests <5 minutes (300 seconds)

## Tasks / Subtasks

- [ ] Task 1: Setup Playwright infrastructure (AC: #1)
  - [ ] Subtask 1.1: Install Playwright and TypeScript dependencies (`npm install -D @playwright/test typescript`)
  - [ ] Subtask 1.2: Create `e2e/playwright.config.ts` with parallel workers (4), base URL, auth storage
  - [ ] Subtask 1.3: Create `e2e/fixtures/auth.ts` with authenticated session fixture (JWT cookie handling)
  - [ ] Subtask 1.4: Create test data fixtures in `e2e/fixtures/recipes.ts` (50 test recipes)

- [ ] Task 2: Implement meal planning E2E tests (AC: #2, #3, #4, #5)
  - [ ] Subtask 2.1: Create `e2e/tests/meal-planning.spec.ts` test file
  - [ ] Subtask 2.2: Implement test: "User generates multi-week meal plan (4 weeks)" - navigate to `/plan`, submit form, verify redirect
  - [ ] Subtask 2.3: Implement test: "User navigates between weeks" - click Next/Previous buttons, verify calendar updates
  - [ ] Subtask 2.4: Implement test: "User regenerates single week" - click regenerate button, verify only target week changes
  - [ ] Subtask 2.5: Implement test: "User regenerates all future weeks" - click regenerate all button, verify all future weeks regenerated

- [ ] Task 3: Implement preferences E2E tests (AC: #6)
  - [ ] Subtask 3.1: Create `e2e/tests/preferences.spec.ts` test file
  - [ ] Subtask 3.2: Implement test: Navigate to `/profile/meal-planning-preferences`, toggle checkboxes
  - [ ] Subtask 3.3: Verify preferences saved and applied to next meal plan generation

- [ ] Task 4: Implement recipe E2E tests (AC: #7)
  - [ ] Subtask 4.1: Create `e2e/tests/recipes.spec.ts` test file
  - [ ] Subtask 4.2: Implement test: Navigate to `/recipes/new`, fill form with accompaniment settings
  - [ ] Subtask 4.3: Verify recipe saved with correct `can_be_side_dish` and `needs_side_dish` values

- [ ] Task 5: Implement shopping list E2E tests (AC: #8)
  - [ ] Subtask 5.1: Create `e2e/tests/shopping.spec.ts` test file
  - [ ] Subtask 5.2: Implement test: Navigate to `/shopping?week=2025-11-10`, verify week-specific ingredients

- [ ] Task 6: Configure CI integration (AC: #9, #10)
  - [ ] Subtask 6.1: Update `.github/workflows/test.yml` to include Playwright test step
  - [ ] Subtask 6.2: Configure parallel execution (4 workers) for faster execution
  - [ ] Subtask 6.3: Enable video recording for failed tests only (storage optimization)
  - [ ] Subtask 6.4: Verify test execution time <5 minutes in CI
  - [ ] Subtask 6.5: Configure artifact uploads for test reports and failure videos (7-day retention)

## Dev Notes

### Architecture Patterns and Constraints

**Testing Architecture** (Test Pyramid Level 3: E2E Tests):
- Playwright provides cross-browser testing (Chromium, Firefox, WebKit)
- Tests validate full user flows from browser automation perspective
- Complements unit tests (Level 1: domain logic) and integration tests (Level 2: HTTP routes)

**Key Technologies**:
- **Playwright 1.56+**: Browser automation framework with TypeScript support
- **Test Fixtures**: Reusable setup for authenticated sessions, test data seeding
- **Parallel Execution**: 4 workers for <5 minute total execution time
- **Video Recording**: Capture failures only (storage-efficient debugging)

**TwinSpark Testing Considerations**:
- TwinSpark attributes (`ts-req`, `ts-target`, `ts-swap`) enable progressive enhancement
- Tests must wait for DOM updates after TwinSpark AJAX requests
- Use `waitForSelector` instead of fixed delays to avoid flaky tests

**Route Performance Requirements** (validated via E2E):
- Generation: P95 <5s (NFR requirement)
- Navigation: P95 <500ms
- Preferences update: P95 <300ms

### Project Structure Notes

**E2E Test Directory Structure**:
```
e2e/
├── playwright.config.ts          # Parallel workers, base URL, auth storage
├── fixtures/
│   ├── auth.ts                   # Authenticated session fixture
│   ├── recipes.ts                # Test recipe data (50 recipes)
│   └── meal-plan.ts              # Meal plan test helpers
├── tests/
│   ├── meal-planning.spec.ts     # Multi-week generation, navigation, regeneration
│   ├── preferences.spec.ts       # Meal planning preferences
│   ├── recipes.spec.ts           # Recipe creation with accompaniments
│   └── shopping.spec.ts          # Shopping list access
└── playwright-report/            # HTML report output
```

**CI/CD Integration**:
- GitHub Actions workflow: `.github/workflows/test.yml`
- Parallel job execution: Unit tests, Integration tests, E2E tests run concurrently
- Artifact uploads: Test reports, videos (7-day retention)

**Alignment with Unified Project Structure**:
- E2E tests isolated from application code (separate `e2e/` directory)
- Test fixtures follow TypeScript module patterns
- CI configuration in `.github/workflows/` per project standards

### Testing Standards Summary

**Playwright Best Practices**:
1. **Test Isolation**: Each test uses isolated authenticated session (no shared state)
2. **Deterministic Assertions**: Use `waitForSelector` instead of `setTimeout` to avoid flaky tests
3. **Parallel Execution**: Tests must pass when run in parallel (4 workers)
4. **Video Recording**: Only for failures (saves storage, aids debugging)
5. **Test Data**: Deterministic fixtures (fixed random seed for reproducibility)

**Reliability Requirements**:
- Flaky test tolerance: 0% (all tests must be deterministic)
- No automatic retries (flaky tests must be fixed, not masked)
- Clean state between tests (database seeded fresh per test)

**CI Requirements**:
- All E2E tests must pass for merge to main
- Test execution time <5 minutes (enforced via timeout)
- Exit code 0 required for deployment gate

### References

**Tech Spec**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/tech-spec-epic-10.md]
- Section "Testing Architecture" (lines 119-128): Test pyramid levels, Playwright role
- Section "Workflow 1: E2E Test Execution Flow" (lines 381-427): CI integration, test phases
- Section "Story 10.1: End-to-End Testing with Playwright" (lines 683-725): Authoritative acceptance criteria

**Epics Document**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/epics.md#L2288-2313]
- Epic 10, Story 10.1: User story statement, prerequisites, technical notes

**Architecture Dependencies**:
- [Source: docs/solution-architecture.md]: Event-sourced monolith architecture, Axum routes, evento aggregates
- [Source: docs/testing-strategy.md]: Test pyramid, coverage goals, CI/CD validation

**Related Implementation Epics**:
- Epic 6: Multi-week meal plan generation (routes under test)
- Epic 7: Week-specific regeneration (routes under test)
- Epic 8: Meal planning preferences (preferences form under test)
- Epic 9: Shopping list UI (shopping list routes under test)

## Dev Agent Record

### Context Reference

- Story Context: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-10.1.xml`

### Agent Model Used

<!-- To be filled by dev agent -->

### Debug Log References

<!-- To be filled by dev agent during implementation -->

### Completion Notes List

<!-- To be filled by dev agent during implementation -->

### File List

<!-- Files created/modified during implementation -->
