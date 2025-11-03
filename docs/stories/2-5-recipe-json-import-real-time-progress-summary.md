# Story 2.5: Recipe JSON Import - Real-Time Progress & Summary

Status: ready-for-dev

## Story

As a user,
I want to see real-time progress during recipe import,
so that I understand the import status and can review results.

## Acceptance Criteria

1. Real-time progress display: "Imported X recipes, Y failed, Z remaining..."
2. Twinspark polling updates progress without blocking UI
3. Summary report after completion: success count, failed count, duplicate count
4. Detailed error list for failed recipes (missing fields, validation errors)
5. Success message with link to recipe library when complete
6. Progress state cleared on page reload (no persistent history)
7. Tests verify progress updates and summary accuracy

## Tasks / Subtasks

- [ ] Create import progress storage (AC: #1, #6)
  - [ ] Define ImportProgress struct with counts and status
  - [ ] Use in-memory storage (HashMap<import_id, ImportProgress>)
  - [ ] Store: success_count, failed_count, duplicate_count, total, status (pending/completed), error_messages
  - [ ] Clear on page reload (no database persistence)

- [ ] Update import command to track progress (AC: #1)
  - [ ] Modify import_recipes command to update progress incrementally
  - [ ] After each recipe processed, update ImportProgress in storage
  - [ ] Increment success_count, failed_count, or duplicate_count
  - [ ] Calculate remaining count: total - (success + failed + duplicate)

- [ ] Create progress polling route (AC: #2)
  - [ ] Create GET /recipes/import/progress/{import_id} route
  - [ ] Query ImportProgress from in-memory storage
  - [ ] Return partial HTML template with progress counts
  - [ ] If status=completed, return summary template instead

- [ ] Create progress partial template (AC: #1, #2)
  - [ ] Create templates/partials/recipes/import-progress.html
  - [ ] Display: "Imported X recipes, Y failed, Z remaining..."
  - [ ] Add Twinspark polling attribute: ts-trigger="load delay 1s"
  - [ ] Show spinner or progress bar animation
  - [ ] Poll until status=completed

- [ ] Create summary template (AC: #3, #4, #5)
  - [ ] Create templates/partials/recipes/import-summary.html
  - [ ] Display success count, failed count, duplicate count
  - [ ] Show detailed error list with recipe names and validation errors
  - [ ] Add success message: "Import complete! {X} recipes added to your library"
  - [ ] Link to /recipes (recipe library)
  - [ ] Show "Start cooking!" CTA

- [ ] Update import form to trigger progress (AC: #2)
  - [ ] After POST /recipes/import, return import-progress.html partial
  - [ ] Partial includes import_id for polling route
  - [ ] Twinspark automatically polls /recipes/import/progress/{import_id}
  - [ ] Polling stops when summary template returned

- [ ] Handle import cancellation (AC: #6)
  - [ ] On page reload, clear ImportProgress for that import_id
  - [ ] No persistent history in database
  - [ ] User can restart import from scratch

- [ ] Write integration tests (AC: #7)
  - [ ] Test progress updates during import (check counts incrementally)
  - [ ] Test summary report accuracy (match actual success/failure counts)
  - [ ] Test error message details (verify validation errors listed)
  - [ ] Test polling behavior (simulate async import)
  - [ ] Test progress cleared on new import

## Dev Notes

- **In-Memory Storage**: Use `Arc<Mutex<HashMap<String, ImportProgress>>>` for thread-safe in-memory progress tracking [Source: docs/architecture.md#ADR-005]
- **Twinspark Polling**: Use `ts-trigger="load delay 1s"` to poll progress endpoint every 1 second until completion [Source: CLAUDE.md#Twinspark API Reference, docs/epics.md#Story 2.5]
- **No Persistence**: Progress state not stored in database; cleared on page reload or server restart [Source: docs/PRD.md#FR010, docs/epics.md#Story 2.5]
- **Partial Templates**: Progress partial swaps itself until completion, then returns summary partial to stop polling [Source: CLAUDE.md#Server-Side Rendering]
- **Import Flow**: POST /recipes/import → returns import_id + progress partial → GET /recipes/import/progress/{id} polls → returns summary when done [Source: docs/architecture.md#API Contracts]

### Project Structure Notes

- **Routes**: Update `src/routes/recipes/import.rs` with progress endpoint
- **Templates**: `templates/partials/recipes/import-progress.html`, `templates/partials/recipes/import-summary.html`
- **Storage**: In-memory HashMap in `src/routes/recipes/import.rs` (AppState or lazy_static)
- **Tests**: Add to `tests/import_test.rs`

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.5] - Full acceptance criteria and progress requirements
- [docs/PRD.md#FR010] - Real-time progress display requirement
- [docs/architecture.md#API Contracts] - Import progress route specifications
- [docs/architecture.md#ADR-005] - Streaming Parser implementation notes
- [CLAUDE.md#Twinspark API Reference] - ts-trigger polling syntax
- [CLAUDE.md#Server-Side Rendering] - Partial template usage

## Dev Agent Record

### Context Reference

- docs/stories/2-5-recipe-json-import-real-time-progress-summary.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
