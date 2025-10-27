# Story 10.4: Documentation Updates

Status: Approved

## Story

As a technical writer,
I want to complete comprehensive user and developer documentation,
so that users understand how to use the enhanced meal planning features and developers can maintain and extend the system.

## Acceptance Criteria

1. **AC1**: User guide created: `docs/user-guide-meal-planning.md`
   - Verify: File exists with sections: Introduction, Generating Plans, Navigation, Regeneration, Preferences, Accompaniments
   - Verify: Non-technical language (no code examples, aimed at end users)

2. **AC2**: User guide covers all key features
   - Verify: Section "Generating Multi-Week Meal Plans" explains how to use generation form
   - Verify: Section "Navigating Between Weeks" explains Previous/Next Week buttons
   - Verify: Section "Regenerating Meal Plans" covers single week and all future weeks regeneration
   - Verify: Section "Setting Meal Planning Preferences" explains breakfast/lunch/dinner toggles and side dish preference
   - Verify: Section "Accompaniments" explains `can_be_side_dish` and `needs_side_dish` recipe settings

3. **AC3**: API documentation updated: `docs/api/meal-planning-routes.md`
   - Verify: File exists with route signatures for all meal planning routes
   - Verify: Each route includes: HTTP method, path, request params, response codes, example requests

4. **AC4**: Architecture document updated with "as-built" notes
   - Verify: `docs/solution-architecture.md` includes section "Epic 10 Enhancements" or similar
   - Verify: Updates describe: multi-week plan structure, rotation state handling, preference application

5. **AC5**: README.md updated with new features
   - Verify: `README.md` includes "Features" section mentioning multi-week meal planning
   - Verify: Brief description of preferences and accompaniment settings

6. **AC6**: Screenshots added to user guide
   - Verify: At least 3 screenshots in user guide (meal calendar, preferences form, shopping list)
   - Verify: Screenshots captured via Playwright screenshot API (consistent styling)

7. **AC7**: Code comments added to complex algorithm functions
   - Verify: `crates/meal-plan/src/algorithm.rs` includes doc comments for `generate_multi_week`, `apply_rotation_state`
   - Verify: Comments explain algorithm logic (not just "what" but "why")

8. **AC8**: Deployment guide updated (migration steps, env vars)
   - Verify: `docs/deployment.md` includes migration steps for Epic 10 schema changes
   - Verify: New environment variables documented (if any added for preferences)

## Tasks / Subtasks

- [ ] Task 1: Create user guide (AC: #1, #2)
  - [ ] Subtask 1.1: Create `docs/user-guide-meal-planning.md` with document structure
  - [ ] Subtask 1.2: Write "Introduction" section: overview of multi-week meal planning feature
  - [ ] Subtask 1.3: Write "Generating Multi-Week Meal Plans" section: form usage, number of weeks selection
  - [ ] Subtask 1.4: Write "Navigating Between Weeks" section: Previous/Next Week buttons, week selector
  - [ ] Subtask 1.5: Write "Regenerating Meal Plans" section: single week vs all future weeks, use cases
  - [ ] Subtask 1.6: Write "Setting Meal Planning Preferences" section: breakfast/lunch/dinner toggles, side dish preference
  - [ ] Subtask 1.7: Write "Accompaniments" section: explain can_be_side_dish and needs_side_dish recipe settings
  - [ ] Subtask 1.8: Review user guide for non-technical language (no code, jargon-free)

- [ ] Task 2: Create API documentation (AC: #3)
  - [ ] Subtask 2.1: Create or update `docs/api/meal-planning-routes.md`
  - [ ] Subtask 2.2: Document route: `POST /plan/generate-multi-week` (request form, response redirect)
  - [ ] Subtask 2.3: Document route: `GET /plan?week=YYYY-MM-DD` (query params, HTML response)
  - [ ] Subtask 2.4: Document route: `POST /plan/regenerate-week?week=YYYY-MM-DD` (query params, response)
  - [ ] Subtask 2.5: Document route: `POST /plan/regenerate-future` (no params, response)
  - [ ] Subtask 2.6: Document route: `GET /profile/meal-planning-preferences` (HTML form response)
  - [ ] Subtask 2.7: Document route: `POST /profile/meal-planning-preferences` (form fields, response)
  - [ ] Subtask 2.8: Document route: `GET /shopping?week=YYYY-MM-DD` (query params, HTML response)
  - [ ] Subtask 2.9: Add example requests and responses for each route

- [ ] Task 3: Update architecture documentation (AC: #4)
  - [ ] Subtask 3.1: Open `docs/solution-architecture.md`
  - [ ] Subtask 3.2: Add "Epic 10 Enhancements" section (or similar heading)
  - [ ] Subtask 3.3: Document multi-week plan structure: `start_weeks` array, `meal_assignments` per week
  - [ ] Subtask 3.4: Document rotation state handling: used recipes tracking, reset logic
  - [ ] Subtask 3.5: Document preference application: how preferences affect meal generation
  - [ ] Subtask 3.6: Document testing architecture additions: E2E tests, performance tests, regression tests

- [ ] Task 4: Update README.md (AC: #5)
  - [ ] Subtask 4.1: Open `README.md`
  - [ ] Subtask 4.2: Add or update "Features" section with multi-week meal planning
  - [ ] Subtask 4.3: Add brief description of meal planning preferences (breakfast/lunch/dinner toggles)
  - [ ] Subtask 4.4: Add brief description of accompaniment settings (side dishes)
  - [ ] Subtask 4.5: Add link to user guide for detailed usage instructions

- [ ] Task 5: Capture screenshots for user guide (AC: #6)
  - [ ] Subtask 5.1: Create Playwright script: `e2e/screenshots/capture-screenshots.ts`
  - [ ] Subtask 5.2: Capture screenshot: Meal plan calendar (4-week view) → save as `docs/images/meal-plan-calendar.png`
  - [ ] Subtask 5.3: Capture screenshot: Meal planning preferences form → save as `docs/images/preferences-form.png`
  - [ ] Subtask 5.4: Capture screenshot: Shopping list for specific week → save as `docs/images/shopping-list.png`
  - [ ] Subtask 5.5: Embed screenshots in user guide with descriptive captions
  - [ ] Subtask 5.6: Create `docs/images/` directory if needed (add to Git LFS if large files)

- [ ] Task 6: Add code comments to algorithm functions (AC: #7)
  - [ ] Subtask 6.1: Open `crates/meal-plan/src/algorithm.rs`
  - [ ] Subtask 6.2: Add doc comment to `generate_multi_week`: explain multi-week generation logic, rotation state initialization
  - [ ] Subtask 6.3: Add doc comment to `apply_rotation_state`: explain how rotation prevents recipe repetition
  - [ ] Subtask 6.4: Add doc comment to `apply_preferences`: explain how preferences filter meal courses
  - [ ] Subtask 6.5: Add inline comments for complex logic sections (e.g., date calculations, recipe selection heuristics)
  - [ ] Subtask 6.6: Run `cargo doc` to verify doc comments render correctly

- [ ] Task 7: Update deployment guide (AC: #8)
  - [ ] Subtask 7.1: Open or create `docs/deployment.md`
  - [ ] Subtask 7.2: Document Epic 10 database migrations (if any schema changes)
  - [ ] Subtask 7.3: Document new environment variables (if any added for preferences or features)
  - [ ] Subtask 7.4: Document deployment sequence: migration → application deployment → verification
  - [ ] Subtask 7.5: Document rollback procedure (if migration fails or errors occur)
  - [ ] Subtask 7.6: Add post-deployment verification steps (smoke tests, health checks)

## Dev Notes

### Architecture Patterns and Constraints

**Documentation Strategy**:
- **User Documentation**: Non-technical, task-oriented, screenshot-heavy
- **API Documentation**: Developer-focused, includes request/response examples
- **Architecture Documentation**: "As-built" notes capturing implementation decisions
- **Code Documentation**: Rustdoc comments explaining "why" (not just "what")

**Documentation Tools**:
- **Markdown**: All documentation in Markdown for version control and easy diffing
- **Playwright Screenshots**: Automated screenshot capture for consistency (not manual screenshots)
- **Rustdoc**: Generate HTML documentation from doc comments (`cargo doc`)
- **Git LFS**: Use for large screenshot files (if needed)

**User Guide Structure** (Non-Technical):
```
docs/user-guide-meal-planning.md
├── Introduction (what is multi-week meal planning)
├── Generating Multi-Week Meal Plans (how to use form)
├── Navigating Between Weeks (Previous/Next buttons)
├── Regenerating Meal Plans (single week vs all future weeks)
├── Setting Meal Planning Preferences (toggles, side dish preference)
└── Accompaniments (can_be_side_dish, needs_side_dish)
```

**API Documentation Structure** (Developer-Focused):
```
docs/api/meal-planning-routes.md
├── POST /plan/generate-multi-week
├── GET /plan?week=YYYY-MM-DD
├── POST /plan/regenerate-week?week=YYYY-MM-DD
├── POST /plan/regenerate-future
├── GET /profile/meal-planning-preferences
├── POST /profile/meal-planning-preferences
└── GET /shopping?week=YYYY-MM-DD
```

### Project Structure Notes

**Documentation Directory Structure**:
```
docs/
├── user-guide-meal-planning.md      # End-user documentation (non-technical)
├── api/
│   └── meal-planning-routes.md      # API route documentation (developer)
├── solution-architecture.md         # Architecture "as-built" notes
├── deployment.md                    # Deployment guide (DevOps)
├── images/
│   ├── meal-plan-calendar.png       # Screenshot: Meal plan calendar
│   ├── preferences-form.png         # Screenshot: Preferences form
│   └── shopping-list.png            # Screenshot: Shopping list
└── known-issues.md                  # Known issues (Story 10.3)
```

**Playwright Screenshot Automation**:
```typescript
// e2e/screenshots/capture-screenshots.ts
import { test, expect } from '@playwright/test';

test('capture meal plan calendar screenshot', async ({ page }) => {
  await page.goto('/plan');
  await page.screenshot({ path: 'docs/images/meal-plan-calendar.png' });
});
```

**Rustdoc Comments Example**:
```rust
/// Generates a multi-week meal plan for the specified user.
///
/// # Algorithm
/// 1. Load user's favorite recipes (filtered by preferences)
/// 2. Initialize rotation state (track used recipes to prevent repetition)
/// 3. For each week:
///    - Generate daily meal assignments (appetizer, main, dessert)
///    - Apply rotation state to avoid consecutive repetition
///    - Update rotation state with used recipes
/// 4. Return generated plan with all assignments
///
/// # Parameters
/// - `user_id`: User to generate plan for
/// - `num_weeks`: Number of weeks to generate (1-8)
/// - `preferences`: Meal planning preferences (breakfast/lunch/dinner flags)
///
/// # Returns
/// - `MealPlan`: Generated plan with assignments for all weeks
pub fn generate_multi_week(
    user_id: String,
    num_weeks: u32,
    preferences: MealPlanPreferences,
) -> Result<MealPlan, GenerationError> {
    // Implementation...
}
```

**Alignment with Unified Project Structure**:
- All documentation in `docs/` directory (versioned with code)
- Screenshots in `docs/images/` (optionally Git LFS for large files)
- API documentation in `docs/api/` subdirectory (organized by domain)
- Deployment guide in `docs/deployment.md` (DevOps reference)

### Testing Standards Summary

**Documentation Quality Criteria**:
1. **User Guide**: Non-technical language, task-oriented, screenshot-heavy
2. **API Documentation**: Complete route signatures, request/response examples
3. **Architecture Docs**: Capture implementation decisions (not just design intent)
4. **Code Comments**: Explain "why" (not just "what"), use Rustdoc format
5. **Deployment Guide**: Step-by-step instructions, rollback procedures

**Screenshot Best Practices**:
1. **Automated Capture**: Use Playwright screenshot API (consistent styling, reproducible)
2. **Descriptive Filenames**: `meal-plan-calendar.png` (not `screenshot-1.png`)
3. **Alt Text**: Add captions in Markdown for accessibility
4. **Version Control**: Commit screenshots to Git (or Git LFS if large)
5. **Update Strategy**: Re-capture screenshots if UI changes

**Rustdoc Best Practices**:
1. **Summary Line**: First line is concise summary (appears in function list)
2. **Algorithm Explanation**: For complex functions, explain algorithm steps
3. **Parameters**: Document each parameter (name, type, purpose)
4. **Returns**: Document return type and meaning
5. **Examples**: Add code examples for public API functions (optional for MVP)

### References

**Tech Spec**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/tech-spec-epic-10.md]
- Section "Story 10.4: Documentation Updates" (lines 791-827): Authoritative acceptance criteria
- Section "Dependencies and Integrations - External Tool Dependencies" (lines 612-623): Playwright for screenshots

**Epics Document**: [Source: /home/snapiz/projects/github/timayz/imkitchen/docs/epics.md#L2363-2385]
- Epic 10, Story 10.4: User story statement, prerequisites, technical notes

**Architecture Dependencies**:
- [Source: docs/solution-architecture.md]: Current architecture documentation to be updated
- [Source: docs/deployment.md]: Current deployment guide to be updated

**Related Documentation**:
- [Source: README.md]: Project overview (to be updated with new features)
- [Source: docs/PRD.md]: Product requirements (source for user guide content)

**API Route Implementations**:
- Epic 6: Multi-week meal plan generation routes
- Epic 7: Week-specific regeneration routes
- Epic 8: Meal planning preferences routes
- Epic 9: Shopping list routes

## Dev Agent Record

### Context Reference

- Story Context: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-10.4.xml`

### Agent Model Used

<!-- To be filled by dev agent -->

### Debug Log References

<!-- To be filled by dev agent during implementation -->

### Completion Notes List

<!-- To be filled by dev agent during implementation -->

### File List

<!-- Files created/modified during implementation -->
