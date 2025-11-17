# Story 2.4: Recipe JSON Import - File Upload & Validation

Status: ready

## Story

As a user,
I want to bulk import recipes from JSON files via drag-and-drop,
So that I can quickly populate my recipe library from exported data.

## Acceptance Criteria

1. Recipe import route accepts multiple JSON files (max 10MB per file, 20 files per batch)
2. Drag-and-drop UI with file picker fallback
3. Validation against recipe schema: all required AND optional fields must be present and valid
4. Malicious content detection (script injection, oversized payloads)
5. Streaming parser for large files to prevent memory issues
6. Invalid recipes skipped with detailed error messages collected
7. Duplicate detection blocks recipes with matching name or similar ingredients
8. Imported recipes stored as private (not shared) by default
9. Tests verify validation, malicious content rejection, and duplicate blocking

## Acceptance Criteria

1. Recipe import route accepts multiple JSON files (max 10MB per file, 20 files per batch)
2. Drag-and-drop UI with file picker fallback
3. Validation against recipe schema: all required AND optional fields must be present and valid
4. Malicious content detection (script injection, oversized payloads)
5. Streaming parser for large files to prevent memory issues
6. Invalid recipes skipped with detailed error messages collected
7. Duplicate detection blocks recipes with matching name or similar ingredients
8. Imported recipes stored as private (not shared) by default
9. Tests verify validation, malicious content rejection, and duplicate blocking

## Tasks / Subtasks

- [ ] Define import data structures (AC: #3)
  - [ ] Create ImportRecipeSchema struct matching RecipeCreated fields
  - [ ] Add validator derives for all required fields
  - [ ] Include all optional fields (must be present but can be null/empty)
  - [ ] Create ImportResult enum (Success, ValidationError, DuplicateError, MaliciousContent)

- [ ] Implement streaming JSON parser (AC: #1, #5)
  - [ ] Create `src/routes/recipes/import.rs`
  - [ ] Accept multipart form data with tokio streaming
  - [ ] Limit file size to 10MB per file (reject larger)
  - [ ] Limit batch to maximum 20 files (reject more)
  - [ ] Use tokio::task::spawn for async file processing

- [ ] Implement malicious content detection (AC: #4)
  - [ ] Scan JSON for script tags: `<script>`, `</script>`
  - [ ] Scan for eval patterns: `eval(`, `Function(`
  - [ ] Check payload size limits before parsing
  - [ ] Reject files with potential XSS or injection vectors
  - [ ] Log rejected files with security warnings

- [ ] Implement recipe validation (AC: #3, #6)
  - [ ] Parse JSON into ImportRecipeSchema struct
  - [ ] Run validator on each recipe
  - [ ] Collect validation errors with field names and reasons
  - [ ] Skip invalid recipes, continue processing valid ones
  - [ ] Accumulate all errors for summary report

- [ ] Implement duplicate detection (AC: #7)
  - [ ] Query existing recipes for user by name (exact match)
  - [ ] Check ingredient similarity using basic comparison (e.g., 80% overlap)
  - [ ] Skip duplicates with detailed message: "Recipe '{name}' already exists"
  - [ ] Collect duplicate count for summary

- [ ] Create import command for valid recipes (AC: #8)
  - [ ] Add bulk_import_recipes method to Command
  - [ ] Loop through validated recipes
  - [ ] Emit RecipeCreated event for each with is_shared = false
  - [ ] Use metadata with user_id and unique request_id per batch

- [ ] Create import form UI with drag-and-drop (AC: #2)
  - [ ] Create `templates/pages/recipes/import.html`
  - [ ] Implement drag-and-drop zone with HTML5 drag events
  - [ ] Add file picker button as fallback
  - [ ] Display selected files list before upload
  - [ ] Show file size and count validation before submit

- [ ] Create import POST route handler (AC: #1)
  - [ ] Add POST /recipes/import route
  - [ ] Extract multipart form data
  - [ ] Validate file count and sizes
  - [ ] Process files with streaming parser
  - [ ] Store import_id for progress tracking (next story)
  - [ ] Return initial response with import_id

- [ ] Implement error collection and reporting (AC: #6)
  - [ ] Create ImportError struct with file_name, recipe_index, error_type, message
  - [ ] Collect all errors during processing
  - [ ] Return detailed error list in summary response
  - [ ] Group errors by type (validation, duplicate, malicious)

- [ ] Write unit tests for validation (AC: #9)
  - [ ] Test valid JSON with all required fields → success
  - [ ] Test missing required field → validation error
  - [ ] Test invalid recipe_type → validation error
  - [ ] Test all optional fields present but empty → success

- [ ] Write unit tests for malicious content detection (AC: #9)
  - [ ] Test JSON with <script> tag → rejected
  - [ ] Test JSON with eval( pattern → rejected
  - [ ] Test oversized payload (>10MB) → rejected
  - [ ] Test normal JSON → accepted

- [ ] Write unit tests for duplicate detection (AC: #9)
  - [ ] Create existing recipe with name "Thai Curry"
  - [ ] Import JSON with same name → duplicate error
  - [ ] Import JSON with similar ingredients → duplicate error
  - [ ] Import JSON with different name/ingredients → success

- [ ] Write integration tests for bulk import (AC: #9)
  - [ ] Import batch of 5 valid recipes → all created
  - [ ] Import batch with 3 valid, 2 invalid → 3 created, 2 skipped
  - [ ] Verify all imported recipes have is_shared = false
  - [ ] Verify RecipeCreated events emitted for each

- [ ] Write E2E test for import flow (AC: #9)
  - [ ] Create Playwright test in `tests/e2e/import.spec.ts`
  - [ ] Test drag-and-drop file upload
  - [ ] Test file picker fallback
  - [ ] Test import with validation errors → see error list

## Dev Notes

### Architecture Patterns

**Streaming File Upload (per architecture.md ADR-005):**
- Use tokio::task::spawn with async/await for file processing
- Stream JSON parsing to avoid loading entire 10MB file into memory
- Constant memory usage regardless of file size
- Example:
```rust
let stream = field.map(|res| res.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));
let body = Body::wrap_stream(stream);
let reader = StreamReader::new(body);
```

**Malicious Content Detection (per PRD NFR013):**
- Scan for script injection before parsing JSON
- Check Content-Type header = application/json
- Validate payload size before streaming
- Reject files with suspicious patterns
- Log security events for audit

**Duplicate Detection Strategy (per epics.md AC #7):**
- Exact name match: `SELECT COUNT(*) FROM recipes WHERE owner_id = ? AND name = ?`
- Ingredient similarity: basic string overlap comparison (80% threshold)
- Simple algorithm for MVP (can enhance post-launch)

**Import as Private (per epics.md AC #8):**
- All imported recipes have is_shared = false by default
- User can manually share recipes later via recipe edit form
- Prevents accidental public sharing of imported data

### Project Structure Notes

**Files to Create:**
```
src/routes/recipes/
└── import.rs         # Streaming upload and validation logic

templates/pages/recipes/
└── import.html       # Drag-and-drop UI

templates/partials/recipes/
├── import-progress.html   # Real-time progress (Story 2.5)
└── import-summary.html    # Results summary (Story 2.5)
```

**Dependencies to Add:**
```toml
[dependencies]
tokio = { version = "1.42", features = ["fs", "io-util"] }
multer = "3.1"  # Multipart form data parsing
futures-util = "0.3"  # Stream utilities
```

### Technical Constraints

**File Size Limits (per PRD FR007):**
- Maximum 10MB per individual file
- Maximum 20 files per batch
- Total batch size ≤ 200MB
- Enforced before streaming begins

**JSON Schema Validation (per epics.md AC #3):**
- ALL fields must be present in JSON (required AND optional)
- Optional fields can be null or empty string
- Strict validation prevents schema drift
- Example valid JSON:
```json
{
  "recipe_type": "MainCourse",
  "name": "Thai Green Curry",
  "ingredients": ["coconut milk", "curry paste"],
  "instructions": "Cook curry paste...",
  "dietary_restrictions": ["gluten-free"],
  "cuisine_type": "Thai",
  "complexity": "medium",
  "advance_prep_text": null,
  "accepts_accompaniment": true
}
```

**Error Handling Strategy:**
- Continue processing valid recipes even if some fail
- Collect all errors for batch summary
- Return HTTP 200 with partial success (not 400/500)
- User sees: "23 imported, 2 failed, 1 duplicate"

**Security Patterns (per PRD NFR013):**
```rust
fn detect_malicious_content(json_str: &str) -> bool {
    json_str.contains("<script")
        || json_str.contains("</script>")
        || json_str.contains("eval(")
        || json_str.contains("Function(")
}
```

### Mockup Reference

**Visual Reference:** `mockups/import.html` (per epics.md line 172)
- Drag-and-drop zone with file picker fallback
- Schema documentation link
- Real-time progress display (Story 2.5)
- Imported/failed/remaining counts

### References

- [Source: docs/PRD.md#FR007-FR010] Recipe import functional requirements
- [Source: docs/PRD.md#NFR013-NFR014] Security and streaming requirements
- [Source: docs/epics.md#Story-2.4] Story acceptance criteria
- [Source: docs/architecture.md#ADR-005] Streaming parser decision
- [Source: CLAUDE.md#Server-Side-Rendering] Twinspark for progress updates
- [Source: mockups/import.html] Visual design reference

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
