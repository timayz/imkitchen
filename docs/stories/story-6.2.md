# Story 6.2: Update Recipe Domain Model

Status: Approved

## Story

As a **backend developer**,
I want **to extend Recipe aggregate with accompaniment and cuisine fields**,
so that **recipes support the new meal planning algorithm**.

## Acceptance Criteria

1. Recipe struct updated with fields: accepts_accompaniment, preferred_accompaniments, accompaniment_category, cuisine, dietary_tags
2. New enums created: AccompanimentCategory, Cuisine, DietaryTag
3. RecipeCreated event updated with new fields
4. RecipeAccompanimentSettingsUpdated event created
5. Evento aggregator trait implemented
6. All fields have serde Serialize/Deserialize derives
7. All fields have bincode Encode/Decode derives
8. Unit tests cover event handlers for new fields
9. Compilation succeeds with zero warnings
10. Existing recipe tests pass (backwards compatibility)

## Tasks / Subtasks

- [ ] Create new enum types (AC: 2)
  - [ ] Create `AccompanimentCategory` enum with variants: Pasta, Rice, Fries, Salad, Bread, Vegetable, Other
  - [ ] Add derives: Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode
  - [ ] Create `Cuisine` enum with 13 predefined variants (Italian, Indian, Mexican, Chinese, Japanese, French, American, Mediterranean, Thai, Korean, Vietnamese, Greek, Spanish) + Custom(String)
  - [ ] Add derives: Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Encode, Decode
  - [ ] Create `DietaryTag` enum with variants: Vegetarian, Vegan, GlutenFree, DairyFree, NutFree, Halal, Kosher
  - [ ] Add derives: Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Encode, Decode

- [ ] Update Recipe aggregate struct (AC: 1, 6, 7)
  - [ ] Add `accepts_accompaniment: bool` field to Recipe struct
  - [ ] Add `preferred_accompaniments: Vec<AccompanimentCategory>` field
  - [ ] Add `accompaniment_category: Option<AccompanimentCategory>` field
  - [ ] Verify `cuisine: Option<Cuisine>` field exists (added in earlier migration)
  - [ ] Verify `dietary_tags: Vec<DietaryTag>` field exists (added in earlier migration)
  - [ ] Ensure all new fields have serde derives (Serialize, Deserialize)
  - [ ] Ensure all new fields have bincode derives (Encode, Decode)

- [ ] Update RecipeCreated event (AC: 3, 6, 7)
  - [ ] Add `accepts_accompaniment: bool` to RecipeCreated event struct
  - [ ] Add `preferred_accompaniments: Vec<AccompanimentCategory>` to event
  - [ ] Add `accompaniment_category: Option<AccompanimentCategory>` to event
  - [ ] Add `cuisine: Option<Cuisine>` to event
  - [ ] Add `dietary_tags: Vec<DietaryTag>` to event
  - [ ] Add `#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]` to event
  - [ ] Add `#[derive(serde::Serialize, serde::Deserialize)]` to event

- [ ] Create RecipeAccompanimentSettingsUpdated event (AC: 4, 6, 7)
  - [ ] Create new event struct with fields: recipe_id, accepts_accompaniment, preferred_accompaniments
  - [ ] Add `#[derive(evento::AggregatorName, bincode::Encode, bincode::Decode)]`
  - [ ] Add `#[derive(serde::Serialize, serde::Deserialize)]`
  - [ ] Add timestamp field: updated_at
  - [ ] Implement event handler in Recipe aggregate

- [ ] Implement evento aggregator trait (AC: 5)
  - [ ] Implement `#[evento::handler(Recipe)]` for RecipeCreated with new fields
  - [ ] Implement `#[evento::handler(Recipe)]` for RecipeAccompanimentSettingsUpdated
  - [ ] Verify aggregate state properly reconstructs from event stream
  - [ ] Test aggregate replay with new event fields

- [ ] Update Recipe read model projections (AC: 8)
  - [ ] Update `recipe_list` projection to include accepts_accompaniment indicator
  - [ ] Update `recipe_detail` projection with all new fields
  - [ ] Update `recipe_filter_counts` projection if cuisine filtering needed
  - [ ] Create subscription handler for RecipeAccompanimentSettingsUpdated
  - [ ] Test projection updates fire correctly on event

- [ ] Create unit tests for new functionality (AC: 8, 10)
  - [ ] Test RecipeCreated event with all new fields serializes/deserializes correctly
  - [ ] Test RecipeAccompanimentSettingsUpdated event serialization
  - [ ] Test Recipe aggregate applies RecipeCreated with new fields
  - [ ] Test Recipe aggregate applies RecipeAccompanimentSettingsUpdated
  - [ ] Test enum serialization: AccompanimentCategory, Cuisine, DietaryTag
  - [ ] Test Cuisine::Custom(String) variant serialization
  - [ ] Test backwards compatibility: old RecipeCreated events (without new fields) still deserialize

- [ ] Verify compilation and existing tests (AC: 9, 10)
  - [ ] Run `cargo build --package recipe` and verify zero warnings
  - [ ] Run `cargo test --package recipe` and verify all existing tests pass
  - [ ] Run `cargo clippy --package recipe` and verify no lints
  - [ ] Verify evento integration compiles without errors

## Dev Notes

### Architecture Context

This story implements domain model updates for Epic 6: Enhanced Meal Planning System, specifically adding **accompaniment support** and **cuisine/dietary metadata** to the Recipe aggregate. These changes enable the multi-week meal planning algorithm to:

1. **Pair main courses with appropriate sides** (rice, pasta, fries, salad, bread, vegetables)
2. **Filter recipes by dietary tags** (vegetarian, vegan, gluten-free, etc.)
3. **Track cuisine variety** across weekly meal plans (Italian, Indian, Mexican, etc.)

### Source Document References

- [Source: docs/architecture-update-meal-planning-enhancements.md#2-accompaniment-recipe-type-system] - Accompaniment design and data model
- [Source: docs/architecture-update-meal-planning-enhancements.md#3-user-preferences-integration] - Cuisine and dietary tags specification
- [Source: docs/architecture-update-meal-planning-enhancements.md#5-domain-model-updates] - Recipe aggregate changes
- [Source: docs/epics.md#story-6.2] - Acceptance criteria and technical notes

### Key Design Decisions

**Accompaniment Optionality:**
- `accepts_accompaniment: bool` is set by the recipe creator (not the meal planning algorithm)
- Main courses control whether they accept a side dish
- No "required" accompaniment concept - always optional
- `preferred_accompaniments: Vec<AccompanimentCategory>` filters which categories pair well (e.g., tikka masala prefers rice)

**Cuisine Flexibility:**
- `Cuisine::Custom(String)` allows user-defined cuisines beyond 13 predefined variants
- Examples: "Fusion", "Regional Brazilian", "Home Cooking"
- Algorithm uses cuisine for variety tracking (avoid repeating same cuisine too frequently)

**Dietary Tags vs Dietary Restrictions:**
- `DietaryTag` on recipes: descriptive metadata ("this recipe is vegetarian")
- `DietaryRestriction` on users: constraints ("I don't eat gluten")
- Algorithm filters recipes WHERE all user restrictions match recipe tags

**Enum Design Rationale:**
- `AccompanimentCategory`: Copy trait (small enum, frequently passed by value)
- `Cuisine`: Clone only (Custom variant contains String, not Copy-safe)
- All enums: Hash + Eq for use in HashMap/HashSet (rotation state tracking)

### Event Sourcing Implications

**Backwards Compatibility:**
- Old `RecipeCreated` events (pre-Epic 6) lack new fields
- Must deserialize gracefully with default values:
  - `accepts_accompaniment: false` (main courses don't accept sides by default)
  - `preferred_accompaniments: vec![]` (empty preference list)
  - `accompaniment_category: None` (not an accompaniment)
  - `cuisine: None` (cuisine unknown)
  - `dietary_tags: vec![]` (no tags)
- Use `#[serde(default)]` or explicit Option types

**Event Stream Integrity:**
- New events (`RecipeAccompanimentSettingsUpdated`) can be added without breaking event replay
- Evento aggregator will skip unknown events gracefully
- Aggregate state reconstruction still valid for old events

**Testing Event Replay:**
```rust
// Test case: Replay old RecipeCreated event (without new fields)
let old_event = RecipeCreated {
    id: "recipe-1".into(),
    title: "Pasta Carbonara".into(),
    recipe_type: RecipeType::MainCourse,
    // Old fields only, new fields missing
};

let aggregate = Recipe::from_events(vec![old_event])?;
assert_eq!(aggregate.accepts_accompaniment, false); // Default value
assert_eq!(aggregate.cuisine, None);
```

### Project Structure Notes

**File Locations:**
- Enums: `crates/recipe/src/types.rs` or inline in `crates/recipe/src/aggregate.rs`
- Recipe struct: `crates/recipe/src/aggregate.rs`
- Events: `crates/recipe/src/events.rs`
- Tests: `crates/recipe/src/tests.rs` or `tests/recipe_aggregate_tests.rs`

**Module Organization:**
```
crates/recipe/
├── src/
│   ├── lib.rs
│   ├── aggregate.rs         # Recipe struct (update here)
│   ├── events.rs            # RecipeCreated, RecipeAccompanimentSettingsUpdated
│   ├── types.rs             # AccompanimentCategory, Cuisine, DietaryTag enums
│   ├── commands.rs          # CreateRecipe, UpdateRecipe commands
│   ├── read_model.rs        # Projection updates
│   └── error.rs
├── tests/
│   └── aggregate_tests.rs   # Unit tests
```

**Naming Conventions:**
- Enums: `PascalCase` (AccompanimentCategory, Cuisine)
- Enum variants: `PascalCase` (Pasta, Rice, Vegetarian)
- Struct fields: `snake_case` (accepts_accompaniment, preferred_accompaniments)
- Events: Past tense `PascalCase` (RecipeCreated, RecipeAccompanimentSettingsUpdated)

### Serialization Standards

**Serde Configuration:**
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")] // JSON field names
pub enum AccompanimentCategory {
    Pasta,    // Serializes as "pasta"
    Rice,     // Serializes as "rice"
    Fries,    // Serializes as "fries"
    Salad,
    Bread,
    Vegetable,
    Other,
}
```

**Bincode Configuration:**
```rust
#[derive(bincode::Encode, bincode::Decode)]
pub struct RecipeCreated {
    // Fields encoded in declaration order
    // Bincode format versioned via evento stream metadata
}
```

**JSON Storage (Database):**
- `preferred_accompaniments: Vec<AccompanimentCategory>` → JSON TEXT: `["pasta", "rice"]`
- `dietary_tags: Vec<DietaryTag>` → JSON TEXT: `["vegetarian", "gluten_free"]`
- Application-layer validation (SQLite JSON functions optional)

### Testing Strategy

**Unit Test Coverage:**
1. ✅ Enum serialization round-trips (serde JSON + bincode)
2. ✅ Event creation with all new fields
3. ✅ Event deserialization with missing fields (backwards compatibility)
4. ✅ Aggregate state reconstruction from event stream
5. ✅ Custom cuisine variant handling (Cuisine::Custom("Fusion"))
6. ✅ Empty vectors/None defaults for old events

**Test Data Examples:**
```rust
// Main course that accepts pasta or rice
let tikka_masala = RecipeCreated {
    title: "Chicken Tikka Masala".into(),
    recipe_type: RecipeType::MainCourse,
    accepts_accompaniment: true,
    preferred_accompaniments: vec![AccompanimentCategory::Pasta, AccompanimentCategory::Rice],
    accompaniment_category: None,
    cuisine: Some(Cuisine::Indian),
    dietary_tags: vec![],
    // ... other fields
};

// Accompaniment recipe
let basmati_rice = RecipeCreated {
    title: "Basmati Rice".into(),
    recipe_type: RecipeType::Accompaniment,
    accepts_accompaniment: false,
    preferred_accompaniments: vec![],
    accompaniment_category: Some(AccompanimentCategory::Rice),
    cuisine: None,
    dietary_tags: vec![DietaryTag::Vegetarian, DietaryTag::Vegan],
    // ... other fields
};
```

### Performance Considerations

**Memory Impact:**
- `AccompanimentCategory`: 1 byte (Copy enum with 7 variants)
- `Cuisine`: 24 bytes worst case (String allocation for Custom variant)
- `Vec<AccompanimentCategory>`: typically 2-3 items (24 bytes + items)
- `Vec<DietaryTag>`: typically 0-2 items (24 bytes + items)
- **Total per Recipe**: ~75-100 bytes additional memory

**Serialization Performance:**
- Bincode: zero-copy deserialization for most fields (highly optimized)
- Serde JSON: negligible overhead for small enums
- Database: JSON TEXT fields use SQLite native storage (no perf impact)

### Integration with Evento

**Aggregate Trait Implementation:**
```rust
impl evento::Aggregator for Recipe {
    type AggregatorName = RecipeAggregatorName;

    fn handle_event(&mut self, event: Self::Event) -> Result<(), Error> {
        match event {
            RecipeEvent::RecipeCreated(e) => {
                self.id = e.id;
                self.title = e.title;
                // ... existing fields
                self.accepts_accompaniment = e.accepts_accompaniment;
                self.preferred_accompaniments = e.preferred_accompaniments;
                self.accompaniment_category = e.accompaniment_category;
                self.cuisine = e.cuisine;
                self.dietary_tags = e.dietary_tags;
            }
            RecipeEvent::RecipeAccompanimentSettingsUpdated(e) => {
                self.accepts_accompaniment = e.accepts_accompaniment;
                self.preferred_accompaniments = e.preferred_accompaniments;
            }
            // ... other event handlers
        }
        Ok(())
    }
}
```

**Event Store Query Example:**
```rust
// Load recipe with evento
let recipe = evento::load::<Recipe>(&executor, &recipe_id).await?;

// Access new fields
if recipe.accepts_accompaniment {
    println!("Preferred sides: {:?}", recipe.preferred_accompaniments);
}
```

### Lessons Learned from Story 6.1

**From Story 6.1 Completion Notes:**
1. ✅ **Pre-existing Columns**: `cuisine` and `dietary_tags` already exist in database schema (migration 01_v0.2.sql)
   - **Action**: Verify Recipe struct already has these fields; do NOT add duplicates
   - **Implication**: Only add 3 NEW fields to Recipe struct (accepts_accompaniment, preferred_accompaniments, accompaniment_category)

2. ✅ **Status Field Constraint**: SQLite CHECK constraint workaround required
   - **Not Applicable**: Recipe domain has no status constraints

3. ✅ **Rollback Testing**: Always test rollback before forward migration
   - **Action**: Ensure evento event stream can handle old events without new fields

4. ✅ **Migration Performance**: Target <5 seconds on development dataset
   - **Action**: No new database changes in this story (schema already updated in 6.1)

### Acceptance Criteria Verification Checklist

Before marking story as "Done", verify:

- [ ] **AC1**: Run `rg "accepts_accompaniment|preferred_accompaniments|accompaniment_category" crates/recipe/src/aggregate.rs` → 3 field declarations found
- [ ] **AC2**: Run `rg "enum AccompanimentCategory|enum Cuisine|enum DietaryTag" crates/recipe/src/types.rs` → 3 enum definitions found
- [ ] **AC3**: Run `rg "struct RecipeCreated" crates/recipe/src/events.rs` → includes all 5 new fields
- [ ] **AC4**: Run `rg "struct RecipeAccompanimentSettingsUpdated" crates/recipe/src/events.rs` → event defined
- [ ] **AC5**: Run `cargo build --package recipe` → compiles successfully
- [ ] **AC6**: Run `rg "#\[derive.*Serialize.*Deserialize" crates/recipe/src/` → all structs/enums have serde derives
- [ ] **AC7**: Run `rg "#\[derive.*Encode.*Decode" crates/recipe/src/events.rs` → all events have bincode derives
- [ ] **AC8**: Run `cargo test --package recipe` → includes tests for new event handlers
- [ ] **AC9**: Run `cargo build --package recipe 2>&1 | grep warning` → zero warnings
- [ ] **AC10**: Run `cargo test --package recipe` → all tests pass (including pre-Epic 6 tests)

### Dependencies and Blockers

**Prerequisites:**
- ✅ Story 6.1 (Database Schema Migration) - **Status: Done**
  - Database columns `accepts_accompaniment`, `preferred_accompaniments`, `accompaniment_category` exist
  - Database columns `cuisine`, `dietary_tags` exist (from earlier migration)

**No Blockers**: All database schema changes complete. Domain model updates can proceed independently.

**Downstream Dependencies** (Stories blocked by this one):
- Story 6.3: Update MealPlan Domain Model (needs Recipe fields for accompaniment pairing)
- Story 6.5: Create Rotation State Management Module (needs Cuisine enum for tracking)
- Epic 7: Multi-Week Algorithm Implementation (needs all domain models complete)

### References

- [Source: docs/architecture-update-meal-planning-enhancements.md#2.3-data-model-changes] - Recipe struct specification
- [Source: docs/architecture-update-meal-planning-enhancements.md#2.4-events] - RecipeCreated and RecipeAccompanimentSettingsUpdated event schemas
- [Source: docs/architecture-update-meal-planning-enhancements.md#5.2-crate-recipe] - Files to update
- [Source: docs/solution-architecture-compact.md#11-component-overview] - Domain crates structure
- [Source: docs/solution-architecture-compact.md#17-implementation-guidance] - Naming conventions
- [Evento Documentation](https://docs.rs/evento/latest/evento/) - Aggregator trait and event handling
- [Serde Documentation](https://serde.rs/) - Serialization/deserialization patterns
- [Bincode Documentation](https://docs.rs/bincode/latest/bincode/) - Binary encoding for evento events

## Dev Agent Record

### Context Reference

- `docs/story-context-6.2.xml` (Generated: 2025-10-25)

### Agent Model Used

claude-sonnet-4-5-20250929

### Debug Log References

### Completion Notes List

### File List
