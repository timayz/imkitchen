# Story 7.6: Shopping List Generation

Status: Done

## Story

As a **meal planning system**,
I want to **generate aggregated shopping lists from weekly meal assignments**,
so that **users can shop efficiently for all ingredients in one trip**.

## Acceptance Criteria

1. Function `generate_shopping_list_for_week` implemented
2. Loads recipes from assignments (main + accompaniments)
3. Aggregates ingredients (extracts from all recipes in week)
4. Groups by category (Produce, Dairy, Meat, Grains, Pantry, Frozen)
5. Combines duplicates (2 onions + 1 onion = 3 onions, same ingredient name)
6. Returns `ShoppingList` with categorized items
7. Includes both main AND accompaniment ingredients
8. Unit tests cover aggregation and categorization

## Tasks / Subtasks

- [x] Implement shopping list generation function (AC: 1)
  - [x] Create function in `crates/meal_planning/src/algorithm.rs`
  - [x] Signature: `pub fn generate_shopping_list_for_week(meal_assignments: &[MealAssignment], recipes: &[Recipe], week_start_date: String) -> ShoppingList`
  - [x] Return `ShoppingList` struct

- [x] Load recipes from assignments (AC: 2)
  - [x] Extract `recipe_id` from each `MealAssignment`
  - [x] Also extract `accompaniment_recipe_id` if `Some`
  - [x] Look up full `Recipe` structs from recipes slice
  - [x] Collect all recipes (mains + accompaniments) for ingredient extraction

- [x] Extract ingredients from all recipes (AC: 3)
  - [x] Iterate through loaded recipes
  - [x] Access `recipe.ingredients: Vec<Ingredient>`
  - [x] Collect all `Ingredient` structs into flat list
  - [x] Total ingredients = sum across all 21 meals + accompaniments

- [x] Aggregate duplicate ingredients (AC: 5)
  - [x] Group ingredients by `ingredient.name` (case-insensitive)
  - [x] Sum `ingredient.quantity` for duplicates
  - [x] Keep first `unit` (assume consistent units, conversion out of scope)
  - [x] Track `from_recipe_ids` for traceability
  - [x] Example: "onion 2 whole" + "onion 1 whole" = "onion 3 whole"

- [x] Categorize ingredients (AC: 4)
  - [x] Map ingredient names to categories:
    - **Produce:** vegetables, fruits (onion, tomato, apple, lettuce)
    - **Dairy:** milk, cheese, butter, yogurt, cream
    - **Meat:** chicken, beef, pork, fish, seafood
    - **Grains:** rice, pasta, bread, flour, oats
    - **Pantry:** oils, spices, canned goods, condiments
    - **Frozen:** frozen vegetables, ice cream
  - [x] Create `ShoppingCategory` per category
  - [x] Assign ingredients to categories
  - [x] Use simple keyword matching (e.g., "chicken" → Meat category)

- [x] Construct ShoppingList result (AC: 6)
  - [x] Generate UUID for `id`
  - [x] Set `meal_plan_id` (from assignments)
  - [x] Set `week_start_date`
  - [x] Create `categories: Vec<ShoppingCategory>`
  - [x] Each category has `name` and `items: Vec<ShoppingItem>`
  - [x] Return `ShoppingList`

- [x] Include main and accompaniment ingredients (AC: 7)
  - [x] Verify both `recipe_id` and `accompaniment_recipe_id` recipes loaded
  - [x] Test that accompaniment ingredients appear in final list

- [x] Write unit tests (AC: 8)
  - [x] Test single meal assignment (3 ingredients)
  - [x] Test duplicate aggregation (2 onions + 1 onion = 3)
  - [x] Test categorization (chicken → Meat, onion → Produce)
  - [x] Test full week (21 assignments) with realistic recipes
  - [x] Test accompaniment ingredient inclusion
  - [x] Test empty meal assignments returns empty shopping list
  - [x] Test case-insensitive ingredient matching ("Onion" vs "onion")
  - [x] Test uncategorized ingredients go to "Other" category
  - [x] Test ShoppingList structure fields

## Dev Notes

### Architecture Patterns

**Shopping List Generation Flow:**
```
1. Extract recipe IDs from meal_assignments (main + accompaniment)
2. Look up full Recipe structs
3. Flatten ingredients from all recipes
4. Aggregate duplicates by name (case-insensitive)
5. Categorize ingredients (keyword matching)
6. Group into ShoppingCategory structs
7. Return ShoppingList with categorized items
```

**Ingredient Aggregation Logic:**
```rust
use std::collections::HashMap;

// Group by lowercase name
let mut aggregated: HashMap<String, (f32, String, Vec<RecipeId>)> = HashMap::new();

for ingredient in all_ingredients {
    let key = ingredient.name.to_lowercase();
    let entry = aggregated.entry(key).or_insert((0.0, ingredient.unit.clone(), vec![]));
    entry.0 += ingredient.quantity;
    entry.2.push(ingredient.recipe_id);
}

// Convert to ShoppingItem vec
```

**Category Mapping Strategy:**
- **Simple Keyword Matching:** Check if ingredient name contains keywords
- **Fallback:** Uncategorized ingredients go to "Other" category
- **Trade-off:** Not perfect (e.g., "chocolate milk" might match "milk" → Dairy), but good enough for MVP
- **Future Enhancement:** Explicit ingredient category database

**Unit Conversion (Out of Scope):**
- MVP assumes consistent units (e.g., all "onions" use "whole")
- "1 cup flour" + "2 cups flour" = "3 cups flour" ✅
- "1 cup flour" + "100g flour" = separate items (not converted) ⚠️
- Documented technical debt for future enhancement

### Project Structure Notes

**File Location:**
- `crates/meal_planning/src/algorithm.rs` - Shopping list generation logic

**Data Models:**
```rust
pub struct ShoppingList {
    id: String,                     // UUID
    meal_plan_id: String,
    week_start_date: Date,
    categories: Vec<ShoppingCategory>,
}

pub struct ShoppingCategory {
    name: String,                   // Produce, Dairy, Meat, Grains, Pantry, Frozen, Other
    items: Vec<ShoppingItem>,
}

pub struct ShoppingItem {
    ingredient_name: String,
    quantity: f32,
    unit: String,
    from_recipe_ids: Vec<RecipeId>, // Traceability
}

pub struct Ingredient {
    name: String,
    quantity: f32,
    unit: String,
    recipe_id: RecipeId,            // Added for traceability
}
```

**Category Keywords Map:**
```rust
const PRODUCE_KEYWORDS: &[&str] = &["onion", "tomato", "potato", "lettuce", "carrot", "apple", "banana"];
const DAIRY_KEYWORDS: &[&str] = &["milk", "cheese", "butter", "yogurt", "cream"];
const MEAT_KEYWORDS: &[&str] = &["chicken", "beef", "pork", "fish", "salmon", "shrimp", "turkey"];
const GRAINS_KEYWORDS: &[&str] = &["rice", "pasta", "bread", "flour", "oats", "quinoa"];
const PANTRY_KEYWORDS: &[&str] = &["oil", "salt", "pepper", "sugar", "vinegar", "sauce", "spice"];
const FROZEN_KEYWORDS: &[&str] = &["frozen", "ice cream"];
```

### Testing Standards

**Test Data:**
- Create test recipes with known ingredients
- Vary quantities and units for aggregation testing
- Include duplicates across different recipes

**Test Scenarios:**
1. **Single Assignment:** 1 recipe with 3 ingredients → 3 shopping items
2. **Duplicate Aggregation:** 2 recipes both use "onion 1 whole" → "onion 2 whole"
3. **Categorization:** Chicken → Meat, Onion → Produce
4. **Full Week:** 21 assignments (7 days × 3 courses) → realistic shopping list
5. **Accompaniment Inclusion:** Main + side both have ingredients → both appear
6. **Empty Input:** 0 assignments → empty shopping list (not error)
7. **Case Insensitivity:** "Onion" and "onion" aggregated together

**Edge Cases:**
- Same ingredient, different units (not converted, separate items)
- Uncategorized ingredients → "Other" category
- Missing recipe in lookup (should not happen, but handle gracefully)

### References

- [Tech Spec: Section 3.6 - Shopping List Generation](../tech-spec-epic-7.md#services-and-modules)
- [Tech Spec: Section 5.1 - AC Story 7.6](../tech-spec-epic-7.md#acceptance-criteria-authoritative)
- [Tech Spec: Shopping List Flow](../tech-spec-epic-7.md#workflows-and-sequencing)
- [Domain Models: ShoppingList, ShoppingCategory, ShoppingItem](../tech-spec-epic-7.md#data-models-and-contracts)
- [Tech Spec: Performance Target <100ms](../tech-spec-epic-7.md#performance)
- [Story 7.4: Single Week Generation](./story-7.4.md) - MealAssignment source
- [Story 7.5: Multi-Week Generation](./story-7.5.md) - Integration point

## Dev Agent Record

### Context Reference

- Story Context XML: `/home/snapiz/projects/github/timayz/imkitchen/docs/story-context-7.6.xml`

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

1. **Implementation Complete** - All 8 acceptance criteria satisfied:
   - AC-1: `generate_shopping_list_for_week` function implemented in `crates/meal_planning/src/algorithm.rs`
   - AC-2: Loads recipes from both `recipe_id` and `accompaniment_recipe_id` fields
   - AC-3: Aggregates ingredients from all recipes across all 21 meal assignments
   - AC-4: Categorizes into 7 categories: Produce, Dairy, Meat, Grains, Pantry, Frozen, Other
   - AC-5: Combines duplicates with case-insensitive name matching
   - AC-6: Returns `ShoppingList` with categorized `ShoppingItem` structs
   - AC-7: Verified accompaniment ingredients included via dedicated test
   - AC-8: 10 comprehensive unit tests covering all scenarios

2. **Data Structures Created**:
   - `Recipe` struct with full ingredient data for shopping list generation
   - `ShoppingList`, `ShoppingCategory`, `ShoppingItem` structs
   - All types properly exported in `crates/meal_planning/src/lib.rs`

3. **Algorithm Design**:
   - Uses HashMap for efficient duplicate aggregation (O(n) complexity)
   - Keyword-based categorization with priority ordering (Frozen checked before Produce)
   - Sorts categories and items alphabetically for consistent UX
   - Tracks `from_recipe_ids` for ingredient traceability

4. **Testing Coverage**:
   - 10 new unit tests written (all passing)
   - 107 total tests in meal_planning crate (all passing)
   - Edge cases covered: empty assignments, case-insensitive matching, uncategorized ingredients
   - Full week scenario (21 assignments) verified

5. **Technical Decisions**:
   - Removed "ice" keyword from FROZEN_KEYWORDS to prevent "rice" from matching
   - Prioritized FROZEN category before PRODUCE to correctly categorize "frozen peas"
   - Used lowercase normalization for case-insensitive ingredient matching
   - UUID generation for shopping list IDs using `uuid::Uuid::new_v4()`

### File List

- `crates/meal_planning/src/algorithm.rs` - Shopping list generation implementation + 10 unit tests
- `crates/meal_planning/src/lib.rs` - Module exports updated
- `docs/stories/story-7.6.md` - Story file updated to Implemented status

---

## Senior Developer Review (AI)

**Reviewer:** Jonathan
**Date:** 2025-10-26
**Outcome:** ✅ **Approved**

### Summary

Story 7.6 delivers a robust shopping list generation algorithm that successfully aggregates ingredients from weekly meal assignments, categorizes them into logical groups, and handles edge cases gracefully. The implementation demonstrates strong software engineering practices with comprehensive test coverage (10 new tests, 107 total passing), efficient O(n) aggregation using HashMap, and thoughtful handling of case-insensitive matching and categorization priority.

**Key Strengths:**
- All 8 acceptance criteria fully satisfied with evidence in tests
- Excellent test coverage including edge cases (empty input, uncategorized ingredients, case-insensitive matching)
- Clean functional design with clear data flow and well-documented algorithm
- Performance-conscious implementation (HashMap aggregation, sorted output)
- Proper integration with existing evento domain model (reuses `Ingredient`, `MealAssignment`)

**Minor Observations:**
- Keyword-based categorization is simple but effective for MVP; future ML-based classification could improve accuracy
- Unit conversion intentionally out of scope (documented trade-off accepted)

### Key Findings

**✅ No blocking issues found**

#### Low Severity Observations

1. **[Low] Keyword Collision Risk** (`crates/meal_planning/src/algorithm.rs:1335-1381`)
   - **Finding:** Removed "ice" from FROZEN_KEYWORDS to prevent "rice" false positive. This is a known limitation of substring matching.
   - **Impact:** Edge cases like "spiced rice" could still mismatch if future keywords overlap
   - **Recommendation:** Consider word-boundary regex matching for short keywords (2-3 chars) in future iteration
   - **Rationale:** Current solution is pragmatic for MVP; full NLP categorization would be over-engineering

2. **[Low] Category Priority Hardcoded** (`crates/meal_planning/src/algorithm.rs:1386-1417`)
   - **Finding:** Category check order matters (Frozen before Produce). This coupling could cause maintenance issues if categories expand
   - **Recommendation:** Document category priority explicitly in code comments, or refactor to declarative priority config
   - **Rationale:** Current approach works but lacks explicitness

### Acceptance Criteria Coverage

| AC # | Criterion | Status | Evidence |
|------|-----------|--------|----------|
| AC-1 | Function `generate_shopping_list_for_week` implemented | ✅ **Pass** | `crates/meal_planning/src/algorithm.rs:1287` |
| AC-2 | Loads recipes from assignments (main + accompaniments) | ✅ **Pass** | Lines 1293-1299: Extracts both `recipe_id` and `accompaniment_recipe_id` |
| AC-3 | Aggregates ingredients | ✅ **Pass** | Lines 1307-1314: Flattens ingredients from all recipes |
| AC-4 | Groups by category | ✅ **Pass** | Lines 1331-1427: 7 categories with keyword matching (Produce, Dairy, Meat, Grains, Pantry, Frozen, Other) |
| AC-5 | Combines duplicates | ✅ **Pass** | Lines 1316-1329: HashMap aggregation with case-insensitive keys, verified by `test_duplicate_ingredient_aggregation` |
| AC-6 | Returns `ShoppingList` | ✅ **Pass** | Lines 1448-1454: Returns properly structured `ShoppingList` with UUID, week_start_date, categories |
| AC-7 | Includes accompaniment ingredients | ✅ **Pass** | Verified by `test_accompaniment_ingredient_inclusion` (lines 3168-3222) |
| AC-8 | Unit tests cover scenarios | ✅ **Pass** | 10 comprehensive tests (lines 2960-3426) covering all ACs + edge cases |

### Test Coverage and Gaps

**Test Quality: Excellent** ✅

**Coverage Analysis:**
- **10 new unit tests** added (all passing)
- **107 total tests** in meal_planning crate (all passing)
- **Edge cases covered:**
  - Empty meal assignments → empty list (`test_empty_meal_assignments`)
  - Case-insensitive matching ("Onion" + "onion") → aggregated (`test_case_insensitive_ingredient_matching`)
  - Uncategorized ingredients → "Other" category (`test_uncategorized_ingredients_to_other`)
  - Full week 21 assignments → correct aggregation (`test_full_week_generation`)
  - Accompaniment inclusion (`test_accompaniment_ingredient_inclusion`)
  - Multiple category verification (`test_ingredient_categorization`)

**Test Pattern Compliance:**
- ✅ Uses `unsafe_oneshot` for evento subscriptions (as per user instructions)
- ✅ Deterministic test data (no flakiness)
- ✅ Clear assertions with descriptive failure messages
- ✅ Helper functions reduce duplication (`create_test_recipe_with_ingredients`, `create_test_meal_assignment`)

**No Gaps Identified** - All scenarios from Dev Notes testing section are covered

### Architectural Alignment

**✅ Fully Aligned** with Epic 7 Tech Spec and project architecture:

1. **Domain Logic Purity:** No external I/O dependencies, pure function design ✅
2. **Event Sourcing Integration:** Reuses existing domain types (`Ingredient`, `MealAssignment`) from evento aggregates ✅
3. **Clean Architecture:** Algorithm in `crates/meal_planning/src/algorithm.rs`, properly exported via `lib.rs` ✅
4. **Performance Target:** O(n) HashMap aggregation meets <100ms target for single week (Epic 7 Tech Spec section 3.6) ✅
5. **Data Model Consistency:** `ShoppingItem` structure matches tech spec contracts ✅
6. **Tailwind 4.1+:** N/A for backend Rust code ✅

**Design Decisions Alignment:**
- ✅ Keyword-based categorization (documented trade-off vs ML approach)
- ✅ Case-insensitive normalization (lowercase keys in HashMap)
- ✅ UUID generation for shopping list IDs (Uuid::new_v4())
- ✅ Alphabetical sorting for UX consistency

### Security Notes

**✅ No security issues** - Algorithm operates on trusted domain data:

1. **Input Validation:** Accepts slices of typed structs (`&[MealAssignment]`, `&[Recipe]`) - type safety enforced by Rust compiler
2. **No User Input Processing:** Function called by application layer with validated data
3. **No Injection Risks:** Pure data transformation, no SQL/eval/template rendering
4. **Resource Limits:** Bounded by week size (21 assignments max) and recipe count - no unbounded allocation
5. **No Secrets:** No credential handling, configuration, or sensitive data exposure

**Dependencies:** Uses std library + `uuid` crate (standard, well-audited dependency)

### Best-Practices and References

**Rust Best Practices Compliance:**

1. **✅ Idiomatic Rust:**
   - Borrows slices (`&[...]`) instead of owned `Vec` - efficient
   - Uses `HashMap` for O(1) lookups - performant
   - Immutable by default, clear mutation intent
   - Descriptive variable names (`all_ingredients`, `aggregated`)

2. **✅ Error Handling:**
   - No panics in production code paths
   - Test assertions use descriptive messages
   - Graceful handling of empty inputs (returns empty list, not error)

3. **✅ Documentation:**
   - Comprehensive function-level docs with AC references
   - Inline comments explain algorithm steps
   - Type documentation for public structs

4. **✅ Testing:**
   - Unit tests follow arrange-act-assert pattern
   - Test names clearly describe scenarios
   - No test interdependencies

**References:**
- [Rust HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html) - Used correctly for aggregation
- [Epic 7 Tech Spec](../tech-spec-epic-7.md#services-and-modules) - Shopping list function signature matches spec
- [Evento Framework](https://docs.rs/evento) - Properly integrates with event-sourced domain models

### Action Items

**✅ No action items required** - Implementation is production-ready

**Optional Future Enhancements** (not blocking):
1. **[Enhancement]** Consider word-boundary regex for short keywords (e.g., `\bice\b` vs substring "ice") to prevent collisions like "rice"
   - **Severity:** Low
   - **Owner:** Product/Engineering (prioritize in backlog if categorization accuracy becomes user complaint)
   - **Related:** AC-4 categorization logic

2. **[Tech Debt]** Document category priority order in algorithm comments
   - **Severity:** Low
   - **Owner:** Dev team
   - **File:** `crates/meal_planning/src/algorithm.rs:1383-1417`
   - **Context:** FROZEN checked before PRODUCE to prevent "frozen peas" → Produce mismatch

**Change Log:**
- 2025-10-26: Story 7.6 implemented and reviewed - Status updated to Done
- 2025-10-26: Senior Developer Review notes appended
