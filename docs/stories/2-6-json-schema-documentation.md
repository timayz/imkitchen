# Story 2.6: JSON Schema Documentation

Status: drafted

## Story

As a third-party developer,
I want publicly accessible versioned JSON schema documentation,
So that I can build tools that export recipes compatible with imkitchen.

## Acceptance Criteria

1. JSON schema document created with all recipe fields and types
2. Schema versioned (v1.0) and published at public URL (/api/schema/recipe/v1.0)
3. Documentation page explains schema fields, required vs optional, and example JSON
4. Schema matches HTML form validation exactly
5. Schema includes all four recipe types with type-specific fields
6. Tests verify schema endpoint accessibility

## Tasks / Subtasks

- [ ] Create JSON schema definition file (AC: #1, #5)
  - [ ] Create `static/schemas/recipe-v1.0.json`
  - [ ] Define schema using JSON Schema Draft 07 or later
  - [ ] Include all recipe fields: recipe_type, name, ingredients, instructions, etc.
  - [ ] Define recipe_type enum: Appetizer, MainCourse, Dessert, Accompaniment
  - [ ] Add type-specific conditionals (accepts_accompaniment only for MainCourse)

- [ ] Mark required vs optional fields (AC: #1, #4)
  - [ ] Required: recipe_type, name, ingredients, instructions
  - [ ] Optional: dietary_restrictions, cuisine_type, complexity, advance_prep_text, accepts_accompaniment
  - [ ] Note: Per Story 2.4 AC #3, all fields must be present in import JSON (can be null/empty)

- [ ] Add field descriptions and examples (AC: #3)
  - [ ] Add description for each field explaining purpose
  - [ ] Include format examples (e.g., ingredients as array of strings)
  - [ ] Document accepts_accompaniment conditional logic
  - [ ] Note dietary_restrictions format: array of strings

- [ ] Create schema serving endpoint (AC: #2)
  - [ ] Add GET /api/schema/recipe/v1.0 route
  - [ ] Serve `static/schemas/recipe-v1.0.json` with Content-Type: application/json
  - [ ] Add CORS headers for third-party access
  - [ ] Cache-Control header for CDN caching

- [ ] Create schema documentation page (AC: #3)
  - [ ] Create `templates/pages/schema-docs.html`
  - [ ] Display human-readable field explanations
  - [ ] Show required vs optional indicators
  - [ ] Include complete example JSON (all fields populated)
  - [ ] Add download link to JSON schema file

- [ ] Add example JSON to documentation (AC: #3)
  - [ ] Create example with MainCourse showing accepts_accompaniment
  - [ ] Create example with Appetizer (no accepts_accompaniment)
  - [ ] Show all field types correctly formatted
  - [ ] Include comments explaining each field

- [ ] Ensure schema matches form validation (AC: #4)
  - [ ] Review CreateRecipeInput validator rules
  - [ ] Ensure schema field types match Rust types
  - [ ] Verify enum values identical (case-sensitive)
  - [ ] Test import validation against schema

- [ ] Write unit tests for schema endpoint (AC: #6)
  - [ ] Test GET /api/schema/recipe/v1.0 returns 200
  - [ ] Test Content-Type is application/json
  - [ ] Test response is valid JSON
  - [ ] Test CORS headers present

- [ ] Write integration test for schema validation (AC: #4)
  - [ ] Load schema from endpoint
  - [ ] Validate example JSON against schema
  - [ ] Ensure validator crate rules match schema
  - [ ] Test edge cases (null vs empty string for optional fields)

- [ ] Write E2E test for documentation access (AC: #6)
  - [ ] Create Playwright test in `tests/e2e/schema_docs.spec.ts`
  - [ ] Navigate to documentation page
  - [ ] Verify schema endpoint link present
  - [ ] Verify example JSON displayed
  - [ ] Test download link works

## Dev Notes

### Architecture Patterns

**JSON Schema Versioning (per epics.md AC #2):**
- Use semantic versioning: v1.0, v1.1, v2.0, etc.
- Schema URL includes version: `/api/schema/recipe/v1.0`
- Breaking changes → major version bump
- New optional fields → minor version bump
- Maintain backward compatibility for old versions

**Static File Serving:**
- Store schema in `static/schemas/` directory
- Axum serves static files from /static route
- Alternative: Embed schema in binary using include_str! macro

**CORS Configuration:**
- Allow all origins for schema endpoint (public API)
- Enable GET method only
- Headers: Access-Control-Allow-Origin: *

### Project Structure Notes

**Files to Create:**
```
static/schemas/
└── recipe-v1.0.json    # JSON schema definition

templates/pages/
└── schema-docs.html    # Documentation page

src/routes/
└── schema.rs           # Schema serving endpoint
```

**Example JSON Schema Structure:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "imkitchen Recipe Schema",
  "version": "1.0",
  "type": "object",
  "required": ["recipe_type", "name", "ingredients", "instructions"],
  "properties": {
    "recipe_type": {
      "type": "string",
      "enum": ["Appetizer", "MainCourse", "Dessert", "Accompaniment"],
      "description": "Type of recipe"
    },
    "name": {
      "type": "string",
      "description": "Recipe name"
    },
    "ingredients": {
      "type": "array",
      "items": {"type": "string"},
      "description": "List of ingredients"
    },
    "accepts_accompaniment": {
      "type": "boolean",
      "description": "Only for MainCourse type"
    }
  }
}
```

### Technical Constraints

**Schema Validation Library:**
- Use jsonschema-rs or serde_json for runtime validation
- Validate imported JSON against schema before processing
- Schema serves as single source of truth

**Field Presence Requirement (per Story 2.4 AC #3):**
- ALL fields must be present in import JSON
- Optional fields can have null or empty values
- This prevents schema drift and ensures consistent data
- Example: `"advance_prep_text": null` (valid)

**Type-Specific Fields:**
- accepts_accompaniment only applies to MainCourse
- Schema can use if/then/else conditionals:
```json
"if": {
  "properties": {"recipe_type": {"const": "MainCourse"}}
},
"then": {
  "required": ["accepts_accompaniment"]
}
```

**Documentation Presentation:**
- Use Tailwind tables for field reference
- Color-code required (red badge) vs optional (blue badge)
- Syntax highlight example JSON with Prism.js or similar
- Include "Try Import" link to /recipes/import page

### Mockup Reference

**Visual Reference:** `mockups/import.html` (per epics.md line 172)
- Link to "View JSON Schema Documentation"
- Schema docs explain structure and requirements

### References

- [Source: docs/PRD.md#FR011] JSON schema documentation requirement
- [Source: docs/epics.md#Story-2.6] Story acceptance criteria
- [Source: docs/epics.md#Story-2.4-AC-3] Field presence validation requirement
- [Source: JSON Schema specification] https://json-schema.org/

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

_To be filled by dev agent_

### Debug Log References

### Completion Notes List

### File List
