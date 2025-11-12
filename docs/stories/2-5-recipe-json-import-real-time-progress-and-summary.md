# Story 2.5: Recipe JSON Import - Real-Time Progress & Summary

Status: drafted

## Story

As a user,
I want to see real-time progress during recipe import,
So that I understand the import status and can review results.

## Acceptance Criteria

1. Real-time progress display: "Imported X recipes, Y failed, Z remaining..."
2. Twinspark polling updates progress without blocking UI
3. Summary report after completion: success count, failed count, duplicate count
4. Detailed error list for failed recipes (missing fields, validation errors)
5. Success message with link to recipe library when complete
6. Progress state cleared on page reload (no persistent history)
7. Tests verify progress updates and summary accuracy

## Tasks / Subtasks

- [ ] Create import progress state management (AC: #1, #6)
  - [ ] Create ImportProgress struct with import_id, total, imported, failed, remaining, status
  - [ ] Store progress in memory (HashMap<import_id, ImportProgress>) or temporary table
  - [ ] Clear progress after completion or on page reload

- [ ] Update import handler to track progress (AC: #1)
  - [ ] Generate unique import_id (ULID) at start
  - [ ] Initialize ImportProgress with total file count
  - [ ] Update progress after each recipe processed
  - [ ] Increment imported/failed counts accordingly
  - [ ] Store detailed error messages for failed recipes

- [ ] Create progress polling endpoint (AC: #2)
  - [ ] Add GET /recipes/import/progress/{import_id} route
  - [ ] Return current ImportProgress state
  - [ ] Render `templates/partials/recipes/import-progress.html` partial
  - [ ] Include imported/failed/remaining counts
  - [ ] Show processing status or completion status

- [ ] Implement Twinspark polling for progress (AC: #2)
  - [ ] Add ts-req="/recipes/import/progress/{import_id}" to import form response
  - [ ] Use ts-trigger="load delay 1s" for polling interval
  - [ ] Update UI with progress partial on each poll
  - [ ] Stop polling when status = "completed" or "failed"

- [ ] Create import summary view (AC: #3, #4)
  - [ ] Create `templates/partials/recipes/import-summary.html`
  - [ ] Display success count, failed count, duplicate count
  - [ ] Show detailed error list with recipe names and error messages
  - [ ] Group errors by type (validation, duplicate, malicious)
  - [ ] Include "View Recipe Library" link

- [ ] Add success message and navigation (AC: #5)
  - [ ] Show success banner when import completes
  - [ ] Include link to /recipes (user's recipe library)
  - [ ] Display total imported count prominently
  - [ ] Use Tailwind success styling (green background)

- [ ] Implement progress cleanup (AC: #6)
  - [ ] Clear ImportProgress from memory/table after completion
  - [ ] Set TTL on progress entries (e.g., 1 hour)
  - [ ] Return "Import not found" if progress expired
  - [ ] Handle page reload gracefully (show empty state)

- [ ] Write unit tests for progress tracking (AC: #7)
  - [ ] Test progress updates as recipes processed
  - [ ] Test imported/failed counts accuracy
  - [ ] Test remaining count decreases correctly
  - [ ] Test completion status set when done

- [ ] Write integration tests for polling (AC: #7)
  - [ ] Start import, poll progress endpoint multiple times
  - [ ] Verify progress updates between polls
  - [ ] Verify summary appears when complete
  - [ ] Test progress cleanup after completion

- [ ] Write E2E test for progress flow (AC: #7)
  - [ ] Create Playwright test in `tests/e2e/import_progress.spec.ts`
  - [ ] Upload files → see initial progress "0/5 imported"
  - [ ] Wait for polling → see updates "3/5 imported"
  - [ ] See final summary → "5 imported, 0 failed"
  - [ ] Click link to recipe library

## Dev Notes

### Architecture Patterns

**Real-Time Progress with Twinspark Polling (per CLAUDE.md):**
- Server-side state management (no WebSockets needed)
- Twinspark polls endpoint every 1 second
- Server returns HTML partial with updated counts
- Example polling markup:
```html
<div id="import-progress"
     ts-req="/recipes/import/progress/{{import_id}}"
     ts-trigger="load delay 1s">
  <p>Importing recipes: {{imported}}/{{total}}</p>
  <p>Failed: {{failed}}</p>
</div>
```

**Progress State Management:**
- In-memory HashMap for MVP (simple, no persistence needed)
- Key: import_id (ULID)
- Value: ImportProgress struct
- Alternative: temporary SQLite table (more robust for restarts)

**Polling Stop Condition:**
- When status = "completed", return summary partial without ts-trigger
- Twinspark stops polling when ts-trigger absent
- User sees final summary, polling ceases

### Project Structure Notes

**Files to Create:**
```
src/routes/recipes/
└── import.rs         # Add progress tracking to existing import handler

src/
└── import_state.rs   # New file for ImportProgress state management

templates/partials/recipes/
├── import-progress.html  # Real-time progress display
└── import-summary.html   # Final results summary
```

**ImportProgress Structure:**
```rust
pub struct ImportProgress {
    pub import_id: String,
    pub total: usize,
    pub imported: usize,
    pub failed: usize,
    pub duplicates: usize,
    pub errors: Vec<ImportError>,
    pub status: ImportStatus,  // Processing | Completed | Failed
}

pub enum ImportStatus {
    Processing,
    Completed,
    Failed,
}
```

### Technical Constraints

**Polling Interval (per epics.md AC #2):**
- 1 second delay between polls (ts-trigger="load delay 1s")
- Balances real-time feel with server load
- Total import time typically <30 seconds for 20 files

**Progress Calculation:**
- total = number of files uploaded
- imported = successfully created recipes
- failed = validation/malicious errors
- duplicates = duplicate detection blocks
- remaining = total - (imported + failed + duplicates)

**Memory Management:**
- Store progress in static HashMap with RwLock
- Clear after completion or 1 hour TTL
- Memory footprint: ~1KB per import × max concurrent imports
- For MVP: assume <100 concurrent imports

**Error Detail Format:**
```rust
pub struct ImportError {
    pub file_name: String,
    pub recipe_index: usize,
    pub error_type: ErrorType,  // Validation | Duplicate | Malicious
    pub message: String,
}
```

### Mockup Reference

**Visual Reference:** `mockups/import.html` (per epics.md line 172)
- Real-time progress: "Imported 15 recipes, 2 failed, 3 remaining..."
- Progress bar optional (counts sufficient for MVP)
- Error list collapsible section
- Success message with green banner

### References

- [Source: docs/PRD.md#FR010] Real-time progress requirement
- [Source: docs/epics.md#Story-2.5] Story acceptance criteria
- [Source: CLAUDE.md#TwinSpark-API-Reference] Polling with ts-trigger
- [Source: mockups/import.html] Visual progress design

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
