# Story 2.6: JSON Schema Documentation

Status: ready-for-dev

## Story

As a third-party developer,
I want publicly accessible versioned JSON schema documentation,
so that I can build tools that export recipes compatible with imkitchen.

## Acceptance Criteria

1. JSON schema document created with all recipe fields and types
2. Schema versioned (v1.0) and published at public URL (/api/schema/recipe/v1.0)
3. Documentation page explains schema fields, required vs optional, and example JSON
4. Schema matches HTML form validation exactly
5. Schema includes all four recipe types with type-specific fields
6. Tests verify schema endpoint accessibility

## Tasks / Subtasks

- [ ] Create JSON schema document (AC: #1, #4, #5)
  - [ ] Define JSON Schema Draft 2020-12 structure
  - [ ] Include all recipe fields from Story 2.1 (name, recipe_type, ingredients, instructions, etc.)
  - [ ] Mark required fields matching validator constraints
  - [ ] Define recipe_type enum: ["Appetizer", "MainCourse", "Dessert", "Accompaniment"]
  - [ ] Add conditional field: accepts_accompaniment (required if recipe_type="MainCourse")
  - [ ] Match HTML form validation from Story 2.1 exactly

- [ ] Create schema storage (AC: #2)
  - [ ] Create static/schemas/recipe-v1.0.json file
  - [ ] Version schema as v1.0
  - [ ] Include $schema, $id, title, description metadata

- [ ] Create schema API route (AC: #2)
  - [ ] Create GET /api/schema/recipe/v1.0 route
  - [ ] Serve static JSON file with application/json content-type
  - [ ] No authentication required (publicly accessible)
  - [ ] Add CORS headers for cross-origin access

- [ ] Create schema documentation page (AC: #3)
  - [ ] Create GET /docs/schema route
  - [ ] Create templates/pages/schema-docs.html
  - [ ] Explain all fields: name, description, type, required/optional
  - [ ] Document recipe_type values and type-specific fields
  - [ ] Include example JSON for each recipe type
  - [ ] Add "Try it" section with link to import page

- [ ] Add example recipes (AC: #3)
  - [ ] Create example JSON for Appetizer recipe
  - [ ] Create example JSON for MainCourse recipe (with accepts_accompaniment)
  - [ ] Create example JSON for Dessert recipe
  - [ ] Create example JSON for Accompaniment recipe
  - [ ] Embed examples in documentation page

- [ ] Update import page with schema link (AC: #3)
  - [ ] Add prominent link to schema documentation on templates/pages/recipes/import.html
  - [ ] Show "View Schema Documentation" button
  - [ ] Include version info: "Using schema v1.0"

- [ ] Write integration tests (AC: #6)
  - [ ] Test GET /api/schema/recipe/v1.0 returns 200
  - [ ] Test response content-type is application/json
  - [ ] Test schema is valid JSON Schema Draft 2020-12
  - [ ] Test schema accessibility without authentication
  - [ ] Test CORS headers present

- [ ] Validate schema against HTML form (AC: #4)
  - [ ] Cross-check schema required fields with CreateRecipeInput validator
  - [ ] Verify field types match (string, array, boolean, etc.)
  - [ ] Ensure recipe_type enum values consistent
  - [ ] Test accepts_accompaniment conditional logic

## Dev Notes

- **JSON Schema Standard**: Use JSON Schema Draft 2020-12 for compatibility with validation tools [Source: docs/PRD.md#FR011]
- **Public Access**: Schema endpoint requires NO authentication; intended for third-party developers [Source: docs/epics.md#Story 2.6]
- **Versioning**: Schema versioned as v1.0; future changes require new version (v1.1, v2.0, etc.) [Source: docs/PRD.md#FR011]
- **Form Validation Match**: Schema constraints MUST match HTML form and validator crate exactly to ensure consistency [Source: docs/epics.md#Story 2.6]
- **CORS Enabled**: Schema API route includes CORS headers for cross-origin requests from third-party tools [Source: docs/architecture.md#Security Architecture]
- **Conditional Fields**: accepts_accompaniment required only if recipe_type="MainCourse" (use JSON Schema "if/then" syntax) [Source: docs/PRD.md#FR006]

### Project Structure Notes

- **Schema File**: `static/schemas/recipe-v1.0.json` (static file served directly)
- **Routes**: `src/routes/schema.rs` with GET /api/schema/recipe/v1.0 and GET /docs/schema
- **Templates**: `templates/pages/schema-docs.html`
- **Tests**: Add to `tests/import_test.rs` or create `tests/schema_test.rs`

No conflicts detected. Structure aligns with unified project architecture.

### References

- [docs/epics.md#Story 2.6] - Full acceptance criteria and public access requirements
- [docs/PRD.md#FR011] - JSON schema documentation requirement
- [docs/PRD.md#FR004-FR006] - Recipe field definitions and types
- [docs/architecture.md#API Contracts] - Schema endpoint specifications
- [JSON Schema Documentation](https://json-schema.org/) - Official JSON Schema standard

## Dev Agent Record

### Context Reference

- docs/stories/2-6-json-schema-documentation.context.xml

### Agent Model Used

<!-- Will be populated during implementation -->

### Debug Log References

### Completion Notes List

### File List
