# Batch Recipe Import - Planning Summary

**Date:** 2025-10-21
**Feature:** Batch recipe import from JSON file
**Epic:** Epic 2 - Recipe Management
**Story:** Story 2.12

---

## Overview

This document summarizes the planning artifacts created for the batch recipe import feature.

### User Request
"As user, I want to be able to add batch import recipes next to 'New recipe' btn. follow 'example-recipe.json' format but as array"

### Planning Artifacts Created

1. **Technical Specification**: `docs/tech-spec-batch-import.md`
   - Comprehensive technical design
   - Implementation stack (no new dependencies)
   - Source tree changes (2 new files, 4 modified files)
   - 4-phase implementation guide (TDD enforced)
   - Error handling scenarios
   - Deployment strategy with rollback plan

2. **Development Story**: `docs/stories/story-2.12.md`
   - 12 acceptance criteria
   - 4 implementation phases with detailed tasks
   - Unit, integration, and E2E test specifications
   - Manual testing scenarios
   - Dev notes with architecture patterns

3. **Cohesion Validation**: Passed ✅
   - Zero breaking changes (purely additive)
   - No new dependencies
   - Low integration risk (brownfield)
   - Clear rollback strategy

---

## Feature Summary

### What
- "Import Recipes" button next to "New recipe" button
- Upload JSON file containing array of recipes
- Each recipe follows `example-recipe.json` format
- Server validates and imports recipes
- Partial success supported (some can fail, others succeed)

### Why
- Faster recipe library population
- Reduces manual data entry burden
- Enables bulk migration from other systems
- Improves user onboarding experience

### How
- **Frontend**: Modal with file upload (Askama + TwinSpark)
- **Backend**: BatchImportRecipesCommand (evento pattern)
- **Validation**: Server-side JSON schema validation
- **Free Tier**: Enforces 10-recipe limit before import
- **Error Handling**: Per-recipe error reporting

---

## Technical Highlights

### Architecture Decisions
- ✅ **Reuses existing `create_recipe()` command** per recipe (DRY)
- ✅ **No rollback for partial failures** (simplicity, user benefit)
- ✅ **evento event sourcing** (BatchImportCompleted event)
- ✅ **Server-rendered modal** (Askama + TwinSpark AJAX)
- ✅ **Zero new dependencies** (uses existing stack)

### File Changes
```
New Files (2):
- templates/components/batch-import-modal.html
- crates/recipe/tests/batch_import_tests.rs

Modified Files (4):
- src/routes/recipes.rs (add POST /recipes/import handler)
- templates/pages/recipe-list.html (add Import button)
- crates/recipe/src/commands.rs (add BatchImportRecipesCommand)
- crates/recipe/src/events.rs (add BatchImportCompleted event)
- crates/recipe/src/lib.rs (add batch_import_recipes function)

No Database Migrations Required ✅
```

### JSON Format
```json
[
  {
    "title": "Recipe 1",
    "recipe_type": "main_course",
    "ingredients": [...],
    "instructions": [...],
    "prep_time_min": 10,
    "cook_time_min": 20,
    "advance_prep_hours": null,
    "serving_size": 4
  },
  {
    "title": "Recipe 2",
    ...
  }
]
```

---

## Implementation Roadmap

### Phase 1: Backend Foundation (TDD) - 3-4 hours
1. Write failing unit tests (6 test cases)
2. Implement `BatchImportRecipesCommand` struct
3. Implement `BatchImportCompleted` event
4. Implement `batch_import_recipes()` function
5. Verify all tests pass

### Phase 2: HTTP Route (TDD) - 2-3 hours
1. Write integration tests (6 scenarios)
2. Implement `POST /recipes/import` handler
3. Add multipart file upload handling
4. Create `BatchImportResultTemplate`
5. Register route with auth middleware
6. Verify integration tests pass

### Phase 3: UI Templates - 2-3 hours
1. Modify recipe list page (add Import button)
2. Create batch import modal template
3. Create results template
4. Add `GET /recipes/import-modal` route
5. Test modal flow manually

### Phase 4: Manual & E2E Testing - 1-2 hours
1. Create test JSON files (valid, invalid, empty, limit)
2. Execute manual testing scenarios
3. Optional: Playwright E2E test
4. Accessibility audit

**Total Estimate: 8-10 hours**

---

## Testing Strategy

### Unit Tests (`crates/recipe/tests/batch_import_tests.rs`)
- ✅ Valid batch import (all succeed)
- ✅ Empty array rejection
- ✅ Free tier limit enforcement
- ✅ Partial success (some fail, some succeed)
- ✅ Invalid recipe_type rejection
- ✅ Missing required fields rejection

### Integration Tests (`tests/batch_import_integration_tests.rs`)
- ✅ POST /recipes/import success
- ✅ Invalid JSON syntax error
- ✅ Empty array error
- ✅ Free tier limit exceeded error
- ✅ Partial failure handling
- ✅ Authentication required

### Manual Testing
- ✅ Upload valid JSON (2-3 recipes)
- ✅ Upload invalid JSON
- ✅ Upload empty array
- ✅ Test free tier limit (9 recipes + import 2)
- ✅ Test partial failure (1 valid + 1 invalid)
- ✅ Verify recipes appear after refresh

---

## Error Handling Matrix

| Scenario | Status | User Message |
|----------|--------|--------------|
| All recipes valid | 200 OK | "3 recipes imported successfully!" |
| Invalid JSON syntax | 422 | "Invalid JSON format. Please check your file syntax." |
| Not an array | 422 | "Expected array of recipes. Root element must be [...]" |
| Empty array | 422 | "No recipes found in file" |
| Missing title (Recipe #2) | 200 OK | "2 successful, 1 failed: Recipe #2: Missing required field 'title'" |
| Free tier exceeded | 403 | "Import would exceed free tier limit (10 recipes). You have 9/10 recipes." |
| Server error | 500 | "Server error during import. Please try again." |

---

## Deployment Plan

### Pre-Deployment Checklist
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual testing completed
- [ ] Accessibility audit passed
- [ ] Code review completed

### Deployment Steps
1. No database migrations required ✅
2. Optional: Enable feature flag `batch_import_enabled: true`
3. Deploy via existing CI/CD pipeline
4. Verify "Import Recipes" button visible

### Rollback Options
- **Option 1**: Hide button via feature flag
- **Option 2**: Disable route (503 Service Unavailable)
- **Option 3**: Rollback Docker image

### Monitoring (OpenTelemetry)
- `batch_import_requests_total` (counter)
- `batch_import_success_count` (histogram)
- `batch_import_failure_count` (histogram)
- `batch_import_duration_seconds` (histogram)

**Alerts:**
- Alert if failure rate > 50%
- Alert if import duration > 10 seconds

---

## Success Criteria

### Functional
- ✅ User can upload JSON file with multiple recipes
- ✅ Valid recipes imported successfully
- ✅ Invalid recipes reported with specific errors
- ✅ Free tier limit enforced
- ✅ Results clearly displayed to user

### Non-Functional
- ✅ No breaking changes to existing functionality
- ✅ Performance acceptable (<2 seconds for <10 recipes)
- ✅ Accessible (keyboard navigation, screen reader compatible)
- ✅ Secure (auth required, input validation)
- ✅ Testable (80%+ code coverage)

### Business
- ✅ Reduces user onboarding time
- ✅ Enables bulk recipe migration
- ✅ Maintains free tier monetization (10-recipe limit)
- ✅ Low development/maintenance cost (reuses existing patterns)

---

## Next Steps

### Ready to Begin Implementation

1. **Start with Phase 1** (Backend Foundation - TDD)
   - Follow the test-first approach in Story 2.12
   - Reference tech spec for implementation details

2. **Use `/bmad:bmm:agents:dev` for development**
   - Developer agent can follow the story tasks
   - Story includes all necessary technical details

3. **Resources Available**
   - Tech Spec: `docs/tech-spec-batch-import.md`
   - Story: `docs/stories/story-2.12.md`
   - Example Format: `example-recipe.json`

### Questions or Clarifications?
- Review cohesion validation report (all checks passed ✅)
- Consult architecture docs if needed
- Story tasks are comprehensive and self-contained

---

## Document References

- **Tech Spec**: `/docs/tech-spec-batch-import.md`
- **Story**: `/docs/stories/story-2.12.md`
- **Epic**: `/docs/epics.md` (Epic 2: Recipe Management)
- **Architecture**: `/docs/solution-architecture.md`
- **UX Spec**: `/docs/ux-specification.md`
- **Example Format**: `/example-recipe.json`

---

_Planning completed: 2025-10-21_
_Status: Approved for implementation_
_Priority: Medium_
_Estimate: 8-10 hours_
