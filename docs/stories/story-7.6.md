# Story 7.6: Shopping List Generation

Status: Draft

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

- [ ] Implement shopping list generation function (AC: 1)
  - [ ] Create function in `crates/meal_planning/src/algorithm.rs`
  - [ ] Signature: `pub fn generate_shopping_list_for_week(meal_assignments: &[MealAssignment], recipes: &[Recipe], week_start_date: Date) -> ShoppingList`
  - [ ] Return `ShoppingList` struct

- [ ] Load recipes from assignments (AC: 2)
  - [ ] Extract `recipe_id` from each `MealAssignment`
  - [ ] Also extract `accompaniment_recipe_id` if `Some`
  - [ ] Look up full `Recipe` structs from recipes slice
  - [ ] Collect all recipes (mains + accompaniments) for ingredient extraction

- [ ] Extract ingredients from all recipes (AC: 3)
  - [ ] Iterate through loaded recipes
  - [ ] Access `recipe.ingredients: Vec<Ingredient>`
  - [ ] Collect all `Ingredient` structs into flat list
  - [ ] Total ingredients = sum across all 21 meals + accompaniments

- [ ] Aggregate duplicate ingredients (AC: 5)
  - [ ] Group ingredients by `ingredient.name` (case-insensitive)
  - [ ] Sum `ingredient.quantity` for duplicates
  - [ ] Keep first `unit` (assume consistent units, conversion out of scope)
  - [ ] Track `from_recipe_ids` for traceability
  - [ ] Example: "onion 2 whole" + "onion 1 whole" = "onion 3 whole"

- [ ] Categorize ingredients (AC: 4)
  - [ ] Map ingredient names to categories:
    - **Produce:** vegetables, fruits (onion, tomato, apple, lettuce)
    - **Dairy:** milk, cheese, butter, yogurt, cream
    - **Meat:** chicken, beef, pork, fish, seafood
    - **Grains:** rice, pasta, bread, flour, oats
    - **Pantry:** oils, spices, canned goods, condiments
    - **Frozen:** frozen vegetables, ice cream
  - [ ] Create `ShoppingCategory` per category
  - [ ] Assign ingredients to categories
  - [ ] Use simple keyword matching (e.g., "chicken" → Meat category)

- [ ] Construct ShoppingList result (AC: 6)
  - [ ] Generate UUID for `id`
  - [ ] Set `meal_plan_id` (from assignments)
  - [ ] Set `week_start_date`
  - [ ] Create `categories: Vec<ShoppingCategory>`
  - [ ] Each category has `name` and `items: Vec<ShoppingItem>`
  - [ ] Return `ShoppingList`

- [ ] Include main and accompaniment ingredients (AC: 7)
  - [ ] Verify both `recipe_id` and `accompaniment_recipe_id` recipes loaded
  - [ ] Test that accompaniment ingredients appear in final list

- [ ] Write unit tests (AC: 8)
  - [ ] Test single meal assignment (3 ingredients)
  - [ ] Test duplicate aggregation (2 onions + 1 onion = 3)
  - [ ] Test categorization (chicken → Meat, onion → Produce)
  - [ ] Test full week (21 assignments) with realistic recipes
  - [ ] Test accompaniment ingredient inclusion
  - [ ] Test empty meal assignments returns empty shopping list
  - [ ] Test case-insensitive ingredient matching ("Onion" vs "onion")

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

<!-- Story context XML will be added by story-context workflow -->

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List
