# Story 2.5: Automatic Recipe Tagging

Status: Approved

## Story

As a user creating a recipe,
I want the system to automatically tag my recipe,
so that I can discover and filter recipes by attributes without manual tagging.

## Acceptance Criteria

1. System analyzes recipe data on save
2. Complexity tag assigned based on: ingredient count, instruction steps, advance prep requirements (Simple: <8 ingredients, <6 steps, no advance prep; Moderate: 8-15 ingredients or 6-10 steps; Complex: >15 ingredients or >10 steps or advance prep required)
3. Cuisine tag inferred from ingredient patterns (e.g., soy sauce + ginger = Asian, oregano + tomato = Italian)
4. Dietary tags auto-assigned: vegetarian (no meat/fish), vegan (no animal products), gluten-free (no wheat/flour)
5. Tags displayed on recipe card and detail page
6. Tags used for discovery filtering and meal planning optimization
7. Manual tag override available if auto-tagging incorrect

## Tasks / Subtasks

- [ ] Implement complexity calculation algorithm (AC: 2)
  - [ ] Create `RecipeComplexityCalculator` domain service in `crates/recipe/src/tagging.rs`
  - [ ] Scoring formula: (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)
  - [ ] Map score to enum: Simple (<30), Moderate (30-60), Complex (>60)
  - [ ] advance_prep_multiplier: 0 if none, 50 if <4 hours, 100 if >=4 hours
  - [ ] Calculate on RecipeCreated and RecipeUpdated events

- [ ] Implement cuisine tag inference (AC: 3)
  - [ ] Create `CuisineInferenceService` in `crates/recipe/src/tagging.rs`
  - [ ] Define ingredient pattern matchers for major cuisines:
    - Italian: tomato + (oregano|basil) + (pasta|parmesan)
    - Asian: soy sauce + (ginger|garlic) + (rice|noodles)
    - Mexican: (cumin|chili|cilantro) + (beans|tortilla)
    - Indian: (curry|turmeric|garam masala)
    - Mediterranean: olive oil + (lemon|feta|olives)
  - [ ] Return best matching cuisine or None if no clear match
  - [ ] Use case-insensitive ingredient name matching

- [ ] Implement dietary tag detection (AC: 4)
  - [ ] Create `DietaryTagDetector` in `crates/recipe/src/tagging.rs`
  - [ ] Define restricted ingredient lists:
    - Non-vegetarian: chicken, beef, pork, fish, lamb, turkey, seafood
    - Non-vegan: meat + dairy (milk, cheese, butter, cream, eggs, yogurt, honey)
    - Non-gluten-free: flour, wheat, bread, pasta (unless specified as gluten-free variant)
  - [ ] Auto-assign tags based on absence of restricted ingredients
  - [ ] Tags: vegetarian, vegan, gluten-free (not mutually exclusive)

- [ ] Add RecipeTagged event and aggregate field (AC: 1, 5)
  - [ ] Define `RecipeTagged` event in `crates/recipe/src/events.rs`
  - [ ] Fields: complexity: Complexity, cuisine: Option<String>, dietary_tags: Vec<String>
  - [ ] Add tags field to RecipeAggregate: `pub tags: RecipeTags`
  - [ ] Implement evento event handler: `recipe_tagged`
  - [ ] Trigger tagging on RecipeCreated and RecipeUpdated

- [ ] Update read model for tag storage (AC: 5, 6)
  - [ ] Add columns to `recipes` table: complexity TEXT, cuisine TEXT, dietary_tags TEXT (JSON array)
  - [ ] Create evento subscription handler for RecipeTagged event
  - [ ] Project tags to read model on event
  - [ ] Index columns for filtering: complexity, cuisine

- [ ] Add tag display to templates (AC: 5)
  - [ ] Update `templates/components/recipe-card.html`: display complexity badge and cuisine tag
  - [ ] Update `templates/pages/recipe-detail.html`: display all tags (complexity, cuisine, dietary)
  - [ ] Badge styling: Simple (green), Moderate (yellow), Complex (red)
  - [ ] Dietary tags as small pills (e.g., "Vegetarian", "Gluten-Free")

- [ ] Add tag filtering to discovery and recipe list (AC: 6)
  - [ ] Update `src/routes/discover.rs`: add query params for tag filtering
  - [ ] Filter options: complexity=simple|moderate|complex, cuisine=italian|asian|mexican|..., dietary=vegetarian|vegan|gluten-free
  - [ ] Update read model queries to filter by tags
  - [ ] Display active filters in UI with clear/remove options

- [ ] Implement manual tag override (AC: 7)
  - [ ] Add "Edit Tags" section in recipe edit form
  - [ ] Allow user to override auto-assigned tags
  - [ ] Store manual overrides in RecipeAggregate
  - [ ] UpdateRecipeTags command with manual_override flag
  - [ ] Display override indicator on recipe detail (e.g., "Manually tagged")

- [ ] Write unit tests for tagging services (TDD)
  - [ ] Test complexity calculation with various ingredient/step counts
  - [ ] Test cuisine inference with known ingredient patterns
  - [ ] Test dietary tag detection with restricted ingredient lists
  - [ ] Test edge cases: empty ingredients, ambiguous patterns, multiple cuisine matches

- [ ] Write integration tests for tag projection (TDD)
  - [ ] Test RecipeTagged event triggers read model update
  - [ ] Test tag filtering queries return correct recipes
  - [ ] Test manual override persists and displays correctly

## Dev Notes

### Architecture Patterns and Constraints

**Event Sourcing with evento:**
- RecipeTagged event emitted after RecipeCreated/RecipeUpdated
- Tags calculated by domain services (RecipeComplexityCalculator, CuisineInferenceService, DietaryTagDetector)
- Tag state stored in RecipeAggregate and projected to read model
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, ADR-001]

**Domain Service Pattern:**
- Tagging logic encapsulated in domain services (not in aggregate)
- Services are stateless, pure functions taking recipe data as input
- Called during command handler execution before event emission
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure]

**Complexity Calculation Formula:**
- Weighted scoring: ingredients (30%), steps (40%), advance prep (30%)
- Rationale: Step count indicates procedural complexity, advance prep adds logistical burden
- Thresholds: Simple <30, Moderate 30-60, Complex >60
- [Source: docs/epics.md#Story 3.12, docs/tech-spec-epic-2.md]

**Cuisine Inference (Pattern Matching):**
- Keyword-based pattern matching on ingredient names
- Multi-cuisine recipes may match multiple patterns → select highest confidence match
- No match returns None (cuisine remains unspecified)
- Future enhancement: machine learning model for improved accuracy
- [Source: docs/epics.md#Story 2.5, line 370]

**Dietary Tag Detection:**
- Conservative approach: only assign tags when confident (no restricted ingredients found)
- Does NOT claim vegan if recipe uses eggs (even if otherwise plant-based)
- Gluten-free requires explicit ingredient checks (flour, wheat, bread, pasta)
- False negatives acceptable (missing vegan tag), false positives unacceptable (claiming gluten-free when not)
- [Source: docs/epics.md#Story 2.5, lines 368-372]

**Tag Filtering Integration:**
- Meal planning algorithm (Epic 3) consumes complexity tag for scheduling
- Discovery page uses tags for user filtering (cuisine, dietary preferences)
- Tags improve recipe discoverability and user experience
- [Source: docs/tech-spec-epic-2.md#In Scope, lines 55-60]

**Manual Override Pattern:**
- User can override auto-assigned tags if incorrect
- Override stored in aggregate with `manual_override: true` flag
- Auto-tagging skipped on subsequent updates if manual override present
- Provides escape hatch for edge cases and user corrections
- [Source: docs/epics.md#Story 2.5, line 376]

### Project Structure Notes

**Codebase Alignment:**

**Domain Crate:**
- Crate: `crates/recipe/`
- Tagging Module: `crates/recipe/src/tagging.rs` (domain services)
- Services: `RecipeComplexityCalculator`, `CuisineInferenceService`, `DietaryTagDetector`
- Event: `RecipeTagged` in `crates/recipe/src/events.rs`
- Aggregate field: `pub tags: RecipeTags` in `RecipeAggregate`
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]

**Event Handlers:**
- Aggregate handler: `recipe_tagged` in `crates/recipe/src/aggregate.rs`
- Subscription handler: `project_recipe_tagged` in `crates/recipe/src/read_model.rs`
- Registered in `src/main.rs` with other recipe subscriptions
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships, lines 422-462]

**Database Schema:**
- Update `recipes` table with columns: complexity TEXT, cuisine TEXT, dietary_tags TEXT
- Migration file: `migrations/009_add_recipe_tags.sql`
- Indexes: CREATE INDEX idx_recipes_complexity ON recipes(complexity)
- Indexes: CREATE INDEX idx_recipes_cuisine ON recipes(cuisine)
- [Source: docs/solution-architecture.md#3.1 Database Schema]

**Templates:**
- Update: `templates/components/recipe-card.html` (display complexity badge, cuisine tag)
- Update: `templates/pages/recipe-detail.html` (display all tags)
- Update: `templates/pages/recipe-form.html` (manual tag override section)
- [Source: docs/solution-architecture.md#7.1 Component Structure]

**Route Handlers:**
- Update: `src/routes/recipes.rs` (call tagging services in create/update handlers)
- Update: `src/routes/discover.rs` (add tag filtering query params)
- [Source: docs/solution-architecture.md#2.3 Page Routing and Navigation]

**Testing:**
- Unit tests: `crates/recipe/tests/tagging_tests.rs` (test domain services)
- Integration tests: `tests/recipe_tagging_integration_tests.rs` (test full flow)
- [Source: docs/solution-architecture.md#15 Testing Strategy]

**Lessons from Previous Stories:**
- Use evento::update pattern for emitting events after aggregate creation
- Structured logging for all tagging operations (include recipe_id, tags assigned)
- Write tests first (TDD) before implementation
- Document domain service reasoning in code comments
- [Source: Story 2.4 completion notes, Technical Correction section]

### References

- **Event Sourcing Pattern**: [docs/solution-architecture.md#3.2 Data Models and Relationships, lines 383-442]
- **Domain Services**: [docs/solution-architecture.md#11.1 Domain Crate Structure, lines 1374-1443]
- **Complexity Calculation**: [docs/epics.md#Story 3.12: Recipe Complexity Calculation, lines 798-819]
- **Automatic Tagging Requirements**: [docs/epics.md#Story 2.5, lines 358-377]
- **Cuisine Inference Logic**: [docs/tech-spec-epic-2.md#Automatic Tagging, lines 55-60]
- **Dietary Tag Detection**: [docs/epics.md#Story 2.5, lines 369-372]
- **Testing Strategy**: [docs/solution-architecture.md#15 Testing Strategy, lines 1951-2066]
- **Epic Acceptance Criteria**: [docs/epics.md#Story 2.5, lines 358-377]

## Dev Agent Record

### Context Reference

- [Story Context 2.5](../story-context-2.5.xml) - Generated 2025-10-14

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
