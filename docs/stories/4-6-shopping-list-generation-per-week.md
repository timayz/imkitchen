# Story 4.6: Shopping List Generation (Per Week)

Status: drafted

## Story

As a user,
I want to generate a shopping list for a specific week,
So that I can efficiently purchase all ingredients needed for that week's meals.

## Acceptance Criteria

1. Shopping list generation command for specified week_id
2. Aggregates ingredients from all recipes in week (appetizers, mains, desserts, accompaniments)
3. Groups ingredients by category (produce, dairy, meat, pantry, etc.)
4. Displays quantities with units (2 lbs chicken, 1 cup rice)
5. Quantity optimization combines duplicate ingredients across recipes
6. Shopping list displayed as printable/shareable format
7. Tests verify ingredient aggregation, grouping, and quantity calculation

## Tasks / Subtasks

- [ ] Create shopping list route handler (AC: #1, #6)
  - [ ] Implement GET `/shopping/{week_number}` route in `src/routes/shopping.rs`
  - [ ] Extract user_id from JWT authentication
  - [ ] Parse week_number parameter from URL
  - [ ] Query meal plan for specified week
  - [ ] Generate shopping list using shopping list service
  - [ ] Render shopping list template with grouped ingredients
- [ ] Implement shopping list generation service (AC: #2, #3, #4, #5)
  - [ ] Create `src/shopping_list.rs` module with ShoppingListService
  - [ ] Query all recipe snapshots for specified week from meal_plan_recipe_snapshots
  - [ ] Parse ingredients JSON from each snapshot (appetizer, main, dessert, accompaniment)
  - [ ] Aggregate ingredients: collect all ingredients across all meals
  - [ ] Parse quantity and unit from ingredient strings (e.g., "2 lbs chicken" → qty=2, unit=lbs, item=chicken)
  - [ ] Combine duplicate ingredients: sum quantities with same unit (2 lbs + 3 lbs = 5 lbs)
  - [ ] Group ingredients by category: produce, dairy, meat, pantry, bakery, etc.
  - [ ] Return structured ShoppingList data for template
- [ ] Implement ingredient parsing logic (AC: #4, #5)
  - [ ] Create ingredient parser function: `parse_ingredient(text: &str) -> Ingredient`
  - [ ] Extract quantity (numeric value), unit (lbs, cups, oz, etc.), and item name
  - [ ] Handle various formats: "2 lbs chicken", "1 cup rice", "3 tomatoes", "salt to taste"
  - [ ] Normalize units for aggregation: convert synonyms (lb/lbs, cup/cups)
  - [ ] Handle special cases: "to taste", "pinch of", "handful"
- [ ] Create ingredient categorization logic (AC: #3)
  - [ ] Define ingredient categories: Produce, Dairy, Meat, Pantry, Bakery, Spices, Other
  - [ ] Create categorization function: `categorize_ingredient(item: &str) -> Category`
  - [ ] Use keyword matching: "chicken" → Meat, "tomato" → Produce, "milk" → Dairy
  - [ ] Default category: Other (for unrecognized items)
  - [ ] Allow extensibility for future category additions
- [ ] Create shopping list Askama template (AC: #6)
  - [ ] Create `templates/pages/shopping.html` extending base template
  - [ ] Display week header: "Shopping List for Week {week_number}"
  - [ ] Render ingredient groups as sections (Produce, Dairy, Meat, etc.)
  - [ ] Display each ingredient: quantity, unit, item name (e.g., "2 lbs chicken")
  - [ ] Add checkboxes for print-friendly shopping experience
  - [ ] Include "Print" button for browser print dialog
  - [ ] Style with Tailwind for clean, scannable layout
  - [ ] Ensure mobile-responsive for grocery store use
- [ ] Write integration tests (AC: #7)
  - [ ] Create `tests/shopping_list_test.rs`
  - [ ] Test shopping list generation for week with multiple meals
  - [ ] Test ingredient aggregation: duplicate ingredients summed correctly
  - [ ] Test quantity parsing: various formats handled (2 lbs, 1 cup, 3 items)
  - [ ] Test ingredient categorization: items grouped correctly by category
  - [ ] Test shopping list with empty meal slots (missing recipes)
  - [ ] Test shopping list with accompaniments included
  - [ ] Test edge cases: zero quantities, "to taste", malformed ingredient strings

## Dev Notes

### Architecture Patterns and Constraints

**Shopping List Service Pattern:**
- Separate service module for reusability across routes
- Pure business logic: ingredient parsing, aggregation, categorization
- No database writes - shopping lists are ephemeral (generated on-demand)
- Future optimization: cache shopping lists for performance (post-MVP)

**Ingredient Parsing Strategy:**
- Regex-based parsing for quantity/unit/item extraction
- Pattern: `(\d+\.?\d*)\s*(lbs?|cups?|oz|g|kg|tsp|tbsp)?\s*(.*)`
- Handle edge cases: fractional quantities (1.5 cups), missing units (3 tomatoes)
- Normalize units: "lb" → "lbs", "cup" → "cups" for consistent aggregation

**Quantity Aggregation Algorithm:**
```rust
// Group by (item, unit), sum quantities
let mut aggregated = HashMap::new();
for ingredient in all_ingredients {
    let key = (ingredient.item.clone(), ingredient.unit.clone());
    *aggregated.entry(key).or_insert(0.0) += ingredient.quantity;
}
```

**Ingredient Categorization:**
- Simple keyword matching for MVP (sufficient for most cases)
- Categories: Produce, Dairy, Meat, Pantry, Bakery, Spices, Other
- Example mappings:
  - Produce: tomato, onion, garlic, lettuce, carrot, etc.
  - Dairy: milk, cheese, butter, yogurt, cream, etc.
  - Meat: chicken, beef, pork, fish, turkey, etc.
  - Pantry: rice, pasta, beans, oil, vinegar, etc.
- Future enhancement: machine learning categorization (post-MVP)

**Data Structure:**
```rust
// src/shopping_list.rs
pub struct ShoppingList {
    pub week_number: usize,
    pub categories: Vec<IngredientCategory>,
}

pub struct IngredientCategory {
    pub name: String,  // "Produce", "Dairy", etc.
    pub ingredients: Vec<AggregatedIngredient>,
}

pub struct AggregatedIngredient {
    pub quantity: f64,
    pub unit: String,
    pub item: String,
}
```

**Printable Format:**
- Clean, minimal styling optimized for printing
- Checkboxes for marking purchased items
- Print CSS: hide navigation, simplify colors
- Use `@media print` rules for print-specific styling

**Performance Considerations:**
- Shopping list generation ~100ms for typical week (7 days, 21 meals)
- Ingredient parsing is O(n) where n = total ingredients
- Aggregation is O(n) with HashMap
- No database writes, query-only operation

### Project Structure Notes

**Files to Create/Modify:**
- `src/routes/shopping.rs` - Shopping list route handler (NEW)
- `src/routes/mod.rs` - Register shopping route (MODIFY)
- `src/shopping_list.rs` - Shopping list generation service (NEW)
- `src/lib.rs` - Export shopping_list module (MODIFY)
- `templates/pages/shopping.html` - Shopping list template (NEW)
- `tests/shopping_list_test.rs` - Integration tests (NEW)

**Route Registration:**
```rust
// src/routes/mod.rs
pub mod shopping;

// In router setup
.route("/shopping/{week_number}", get(shopping::get_shopping_list))
```

**Service Integration:**
```rust
// src/routes/shopping.rs
use crate::shopping_list::{ShoppingListService, ShoppingList};

pub async fn get_shopping_list(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(week_number): Path<usize>,
) -> impl IntoResponse {
    let service = ShoppingListService::new(state.pool.clone());
    let list = service.generate_for_week(&user.id, week_number).await?;

    ShoppingListTemplate { list, week_number }.into_response()
}
```

**Visual Mockup Alignment:**
- Implements `mockups/shopping-list.html` grouped ingredient display
- Categories displayed as sections: Proteins, Vegetables, Dairy, Bakery, Pantry
- Quantity aggregation matches mockup examples
- Printable checkbox list matches mockup design

**Recipe Snapshot Integration:**
- Uses meal_plan_recipe_snapshots from Epic 3 (Story 2.7, Story 3.8)
- Snapshots ensure historical accuracy: shopping list matches generated meal plan
- If recipe deleted by owner, snapshot preserves ingredients for shopping list

### References

- [Source: docs/epics.md#Story 4.6 - Acceptance Criteria and Prerequisites]
- [Source: docs/PRD.md#FR035 - Shopping list generation per week]
- [Source: docs/PRD.md#FR036 - Include ingredients from mains and accompaniments]
- [Source: docs/architecture.md#HTTP Routes - /shopping/{week_number} route contract]
- [Source: docs/architecture.md#Data Architecture - meal_plan_recipe_snapshots table]
- [Source: CLAUDE.md#Server-Side Rendering Rules - Askama template structure]

## Dev Agent Record

### Context Reference

<!-- Path(s) to story context XML will be added here by context workflow -->

### Agent Model Used

<!-- Model information will be added during implementation -->

### Debug Log References

<!-- Debug logs will be added during implementation -->

### Completion Notes List

<!-- Completion notes will be added during implementation -->

### File List

<!-- Files created/modified will be listed during implementation -->
