# Story 2.4: Recipe JSON Import - File Upload & Validation

Status: ready-for-dev

## Story

As a user,
I want to bulk import recipes from JSON files via drag-and-drop,
so that I can quickly populate my recipe library from exported data.

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
  - [ ] Define ImportedRecipe struct matching JSON schema
  - [ ] Implement validator constraints for all fields
  - [ ] Add serde derive for JSON deserialization
  - [ ] Match HTML form validation from Story 2.1

- [ ] Create recipe import command (AC: #3, #6, #7, #8)
  - [ ] Define ImportRecipesInput with file data
  - [ ] Implement import_recipes command (batch processing)
  - [ ] Validate each recipe using validator crate
  - [ ] Check for duplicates (matching name or similar ingredients)
  - [ ] Skip invalid recipes, collect error messages
  - [ ] Emit RecipeCreated events for valid recipes (is_shared=false)
  - [ ] Return ImportResult with success count, failed count, errors list

- [ ] Implement streaming JSON parser (AC: #1, #5)
  - [ ] Create src/routes/recipes/import.rs
  - [ ] Use tokio for async file processing
  - [ ] Implement streaming parser (serde_json::from_reader or similar)
  - [ ] Process recipes incrementally to avoid 200MB memory spike
  - [ ] Enforce 10MB max per file, 20 files per batch
  - [ ] Set timeout (30s per file) to prevent DoS

- [ ] Implement malicious content detection (AC: #4)
  - [ ] Scan recipe fields for script tags (<script>, <iframe>, etc.)
  - [ ] Check for eval patterns, onclick handlers
  - [ ] Validate payload size (reject if > 10MB after decompression)
  - [ ] Reject if suspicious patterns detected
  - [ ] Log malicious content attempts

- [ ] Create import UI route and template (AC: #2)
  - [ ] Create GET /recipes/import route
  - [ ] Create templates/pages/recipes/import.html
  - [ ] Implement drag-and-drop zone with file picker fallback
  - [ ] Show file list with size validation
  - [ ] Use Twinspark ts-req for upload
  - [ ] Display schema documentation link

- [ ] Handle import submission (AC: #1, #6)
  - [ ] Create POST /recipes/import route
  - [ ] Accept multipart/form-data with multiple files
  - [ ] Validate content-type: application/json
  - [ ] Parse each file with streaming parser
  - [ ] Call import_recipes command for each file
  - [ ] Return import_id for progress tracking (Story 2.5)

- [ ] Implement duplicate detection (AC: #7)
  - [ ] Query recipes table for matching name (case-insensitive)
  - [ ] Use fuzzy matching for similar ingredients (e.g., Levenshtein distance)
  - [ ] Block duplicate if exact name match OR 80%+ ingredient similarity
  - [ ] Add to error list with clear message: "Duplicate recipe: {name}"

- [ ] Store import results for progress tracking (AC: #6)
  - [ ] Create import_results table or in-memory cache
  - [ ] Store import_id, success_count, failed_count, error_messages
  - [ ] Make available for Story 2.5 progress polling

- [ ] Update recipe list to show imported recipes (AC: #8)
  - [ ] Imported recipes visible in user's recipe list
  - [ ] Show "Imported" badge or metadata
  - [ ] Filter: is_shared=false (private by default)

- [ ] Write integration tests (AC: #9)
  - [ ] Test valid JSON import (all 4 recipe types)
  - [ ] Test validation failures (missing required fields)
  - [ ] Test malicious content rejection (script injection)
  - [ ] Test duplicate detection (name and ingredient matching)
  - [ ] Test file size limits (10MB per file, 20 files per batch)
  - [ ] Test streaming parser with large files
  - [ ] Verify imported recipes are private (is_shared=false)

## Dev Notes

- **Streaming Parser**: Use tokio tasks with serde_json streaming to handle 10MB files without memory spikes [Source: docs/architecture.md#ADR-005]
- **Security**: OWASP compliance requires input validation, malicious content detection, and DoS prevention [Source: docs/architecture.md#Security Architecture, docs/PRD.md#NFR013]
- **Duplicate Detection**: Check name exact match OR fuzzy ingredient similarity (80%+ threshold) [Source: docs/PRD.md#FR009, docs/epics.md#Story 2.4]
- **Batch Limits**: Max 10MB per file, 20 files per batch enforced at upload [Source: docs/PRD.md#FR007]
- **Private by Default**: Imported recipes have is_shared=false; users must explicitly share [Source: docs/PRD.md#FR008, docs/epics.md#Story 2.4]
- **Progress Tracking**: Store import_id and results for Story 2.5 progress polling [Source: docs/epics.md#Story 2.5]

### Project Structure Notes

- **Routes**: `src/routes/recipes/import.rs` (GET/POST)
- **Templates**: `templates/pages/recipes/import.html`
- **Commands**: Add import_recipes method to `crates/imkitchen-recipe/src/command.rs`
- **Parser**: Streaming JSON parser in import route handler
- **Storage**: import_results table or in-memory cache (Redis alternative for MVP)
- **Tests**: Add to `tests/recipes_test.rs` and create `tests/import_test.rs`

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.4] - Full acceptance criteria and security requirements
- [docs/PRD.md#FR007-FR009] - Bulk import functional requirements
- [docs/PRD.md#NFR013-NFR014] - Security and streaming parser requirements
- [docs/architecture.md#ADR-005] - Streaming Parser for Recipe Import
- [docs/architecture.md#Security Architecture] - OWASP compliance and file upload security
- [docs/architecture.md#API Contracts] - Import route specifications

## Dev Agent Record

### Context Reference

- docs/stories/2-4-recipe-json-import-file-upload-validation.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
