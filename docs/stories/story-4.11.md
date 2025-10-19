# Story 4.11: Ingredient Quantity Aggregation Logic

Status: Approved

## Story

As a **system**,
I want **to accurately aggregate ingredient quantities**,
so that **shopping lists show correct totals**.

## Acceptance Criteria

1. System identifies duplicate ingredients by normalized name (case-insensitive, trim whitespace)
2. Quantities with same unit added directly (2 cups + 1 cup = 3 cups)
3. Quantities with compatible units converted then added (1 cup + 240ml = 2 cups)
4. Incompatible units kept separate (1 whole onion + 1 cup diced onion = separate line items)
5. Unit conversion table: cups↔ml, tablespoons↔teaspoons, lbs↔oz, grams↔kg
6. Fractional quantities handled: 1/2 cup + 1/4 cup = 3/4 cup
7. Aggregated quantities rounded to practical values (avoid "2.347 cups" → "2 1/3 cups")
8. Ambiguous quantities flagged for manual review (e.g., "a pinch" + "to taste")

## Tasks / Subtasks

- [ ] Implement ingredient name normalization (AC: 1)
  - [ ] Create `normalize_ingredient_name()` function
  - [ ] Case-insensitive comparison (lowercase conversion)
  - [ ] Trim leading/trailing whitespace
  - [ ] Remove duplicate spaces
  - [ ] Unit tests for name normalization edge cases

- [ ] Implement same-unit quantity aggregation (AC: 2)
  - [ ] Create `aggregate_same_unit()` function
  - [ ] Parse quantity strings to numeric values
  - [ ] Sum quantities with matching units
  - [ ] Format aggregated quantity as string
  - [ ] Unit tests for same-unit aggregation

- [ ] Create unit conversion table (AC: 3, 5)
  - [ ] Define ConversionFactor struct
  - [ ] Volume conversions: cups↔ml (1 cup = 240ml)
  - [ ] Volume conversions: tablespoons↔teaspoons (1 tbsp = 3 tsp)
  - [ ] Weight conversions: lbs↔oz (1 lb = 16 oz)
  - [ ] Weight conversions: grams↔kg (1000g = 1kg)
  - [ ] Create `convert_unit()` function
  - [ ] Unit tests for each conversion pair

- [ ] Implement compatible unit aggregation (AC: 3)
  - [ ] Create `aggregate_compatible_units()` function
  - [ ] Check if units are convertible
  - [ ] Convert to common unit (e.g., both to cups)
  - [ ] Sum converted quantities
  - [ ] Return aggregated result in preferred unit
  - [ ] Unit tests for compatible unit aggregation

- [ ] Handle incompatible units (AC: 4)
  - [ ] Create `are_units_compatible()` function
  - [ ] Return false for incompatible units (whole vs diced, count vs volume)
  - [ ] Keep incompatible items as separate line items
  - [ ] Unit tests for incompatibility detection

- [ ] Implement fractional quantity handling (AC: 6)
  - [ ] Add fraction parsing library (fraction crate)
  - [ ] Parse fractional strings ("1/2", "3/4", "1 1/2")
  - [ ] Perform fraction arithmetic (addition)
  - [ ] Simplify fractions (4/8 → 1/2)
  - [ ] Format fractions for display
  - [ ] Unit tests for fraction arithmetic

- [ ] Implement quantity rounding (AC: 7)
  - [ ] Create `round_to_practical_value()` function
  - [ ] Round to nearest 1/4, 1/3, 1/2 for small quantities
  - [ ] Round to nearest whole number for large quantities
  - [ ] Avoid excessive precision (max 2 decimal places)
  - [ ] Unit tests for rounding logic

- [ ] Flag ambiguous quantities (AC: 8)
  - [ ] Create ambiguous quantity detector
  - [ ] Detect non-numeric quantities ("a pinch", "to taste", "dash")
  - [ ] Store ambiguous items with warning flag
  - [ ] Display warning icon in UI for manual review
  - [ ] Unit tests for ambiguous quantity detection

- [ ] Create IngredientAggregationService (AC: all)
  - [ ] Define service struct in shopping crate
  - [ ] Implement `aggregate_ingredients()` main entry point
  - [ ] Integrate normalization, conversion, and aggregation logic
  - [ ] Return aggregated ingredients with categories
  - [ ] Integration tests for full aggregation flow

- [ ] Add integration tests (AC: all)
  - [ ] Test: Same ingredient different cases aggregates (AC #1)
  - [ ] Test: Same unit quantities sum correctly (AC #2)
  - [ ] Test: Compatible units convert and sum (AC #3)
  - [ ] Test: Incompatible units remain separate (AC #4)
  - [ ] Test: All unit conversions work bidirectionally (AC #5)
  - [ ] Test: Fractions add correctly (AC #6)
  - [ ] Test: Quantities round to practical values (AC #7)
  - [ ] Test: Ambiguous quantities flagged (AC #8)

## Dev Notes

### Architecture Patterns and Constraints

**Ingredient Aggregation Algorithm**:
```rust
// High-level algorithm
fn aggregate_ingredients(recipes: Vec<Recipe>) -> Vec<AggregatedIngredient> {
    let mut ingredient_map: HashMap<String, Vec<Ingredient>> = HashMap::new();

    // Step 1: Group by normalized name
    for recipe in recipes {
        for ingredient in recipe.ingredients {
            let normalized_name = normalize_ingredient_name(&ingredient.name);
            ingredient_map.entry(normalized_name)
                .or_insert(Vec::new())
                .push(ingredient);
        }
    }

    // Step 2: Aggregate quantities within each group
    let mut aggregated = Vec::new();
    for (normalized_name, ingredients) in ingredient_map {
        let aggregated_ingredient = aggregate_ingredient_group(ingredients);
        aggregated.push(aggregated_ingredient);
    }

    // Step 3: Assign categories
    for ingredient in &mut aggregated {
        ingredient.category = assign_category(&ingredient.name);
    }

    aggregated
}

fn aggregate_ingredient_group(ingredients: Vec<Ingredient>) -> AggregatedIngredient {
    // Group by unit compatibility
    let mut unit_groups = group_by_unit_compatibility(ingredients);

    // Aggregate each unit group
    let mut quantities = Vec::new();
    for group in unit_groups {
        let total = sum_quantities_with_conversion(&group);
        let rounded = round_to_practical_value(total);
        quantities.push(rounded);
    }

    // If multiple incompatible units, create separate line items
    if quantities.len() > 1 {
        // Return multiple AggregatedIngredient instances
    }

    AggregatedIngredient {
        name: ingredients[0].name.clone(),
        quantity: quantities[0],
        unit: quantities[0].unit,
        category: assign_category(&ingredients[0].name),
    }
}
```

**Unit Conversion Table**:
```rust
lazy_static! {
    static ref CONVERSION_TABLE: HashMap<(Unit, Unit), f64> = {
        let mut m = HashMap::new();
        // Volume conversions
        m.insert((Unit::Cup, Unit::Milliliter), 240.0);
        m.insert((Unit::Milliliter, Unit::Cup), 1.0 / 240.0);
        m.insert((Unit::Tablespoon, Unit::Teaspoon), 3.0);
        m.insert((Unit::Teaspoon, Unit::Tablespoon), 1.0 / 3.0);
        m.insert((Unit::Cup, Unit::Tablespoon), 16.0);
        m.insert((Unit::Tablespoon, Unit::Cup), 1.0 / 16.0);

        // Weight conversions
        m.insert((Unit::Pound, Unit::Ounce), 16.0);
        m.insert((Unit::Ounce, Unit::Pound), 1.0 / 16.0);
        m.insert((Unit::Kilogram, Unit::Gram), 1000.0);
        m.insert((Unit::Gram, Unit::Kilogram), 1.0 / 1000.0);

        m
    };
}

enum Unit {
    // Volume
    Cup,
    Milliliter,
    Tablespoon,
    Teaspoon,

    // Weight
    Pound,
    Ounce,
    Kilogram,
    Gram,

    // Count
    Whole,
    Piece,

    // Other
    Pinch,
    Dash,
    ToTaste,
}
```

**Fraction Handling**:
Use the `fraction` crate for precise fractional arithmetic:
```rust
use fraction::Fraction;

fn parse_quantity(quantity_str: &str) -> Result<Fraction, ParseError> {
    // Handle formats: "2", "1/2", "1 1/2", "0.5"
    if quantity_str.contains(" ") {
        // Mixed fraction: "1 1/2"
        let parts: Vec<&str> = quantity_str.split_whitespace().collect();
        let whole: i64 = parts[0].parse()?;
        let fractional = Fraction::from_str(parts[1])?;
        Ok(Fraction::from(whole) + fractional)
    } else if quantity_str.contains("/") {
        // Pure fraction: "1/2"
        Fraction::from_str(quantity_str)
    } else {
        // Decimal or whole number: "2" or "0.5"
        let value: f64 = quantity_str.parse()?;
        Ok(Fraction::from(value))
    }
}

fn format_quantity(fraction: Fraction) -> String {
    // Convert to mixed fraction if > 1
    if fraction >= Fraction::from(1) {
        let whole = fraction.floor();
        let fractional = fraction - Fraction::from(whole);

        if fractional == Fraction::from(0) {
            format!("{}", whole)
        } else {
            format!("{} {}", whole, fractional)
        }
    } else {
        format!("{}", fraction)
    }
}
```

**Practical Rounding**:
```rust
fn round_to_practical_value(quantity: Fraction) -> Fraction {
    let value = quantity.to_f64();

    if value < 1.0 {
        // Round to nearest 1/4, 1/3, 1/2
        let quarters = (value * 4.0).round() / 4.0;
        let thirds = (value * 3.0).round() / 3.0;
        let halves = (value * 2.0).round() / 2.0;

        // Choose closest
        // ... implementation
        Fraction::from(quarters)
    } else if value < 10.0 {
        // Round to nearest 0.5
        Fraction::from((value * 2.0).round() / 2.0)
    } else {
        // Round to nearest whole number
        Fraction::from(value.round())
    }
}
```

### Source Tree Components to Touch

**New Files to Create**:
```
crates/shopping/src/aggregation.rs
   IngredientAggregationService struct
   aggregate_ingredients() main entry point
   normalize_ingredient_name() function
   aggregate_ingredient_group() function
   group_by_unit_compatibility() function
   sum_quantities_with_conversion() function

crates/shopping/src/conversion.rs
   Unit enum (Cup, Ml, Tbsp, Tsp, Lb, Oz, Kg, G, Whole, Piece, Pinch, Dash, ToTaste)
   ConversionFactor struct
   CONVERSION_TABLE lazy_static
   convert_unit() function
   are_units_compatible() function
   get_conversion_factor() function

crates/shopping/src/fraction.rs
   Wrapper around fraction crate
   parse_quantity() function
   format_quantity() function
   round_to_practical_value() function

crates/shopping/src/ambiguous.rs
   AMBIGUOUS_QUANTITIES constant (set of non-numeric keywords)
   is_ambiguous_quantity() function

tests/ingredient_aggregation_tests.rs (NEW FILE)
   Integration tests for Story 4.11
   Test cases for all 8 acceptance criteria
```

**Existing Files to Modify**:
```
crates/shopping/src/lib.rs
   Export new modules: aggregation, conversion, fraction, ambiguous

crates/shopping/src/commands.rs
   Update GenerateShoppingList command handler
   Call IngredientAggregationService.aggregate_ingredients()

crates/shopping/Cargo.toml
   Add dependency: fraction = "0.15"
   Add dependency: lazy_static = "1.4"
```

**Dependencies to Add**:
- `fraction = "0.15"` - Precise fractional arithmetic
- `lazy_static = "1.4"` - Static conversion table initialization

### Testing Standards Summary

**TDD Approach**:
1. Write failing test for ingredient name normalization (AC #1)
2. Implement normalize_ingredient_name() function
3. Write failing test for same-unit aggregation (AC #2)
4. Implement aggregate_same_unit() function
5. Write failing test for unit conversion (AC #3, #5)
6. Implement unit conversion table and convert_unit() function
7. Write failing test for incompatible units (AC #4)
8. Implement are_units_compatible() function
9. Write failing test for fraction handling (AC #6)
10. Implement fraction parsing and arithmetic
11. Write failing test for practical rounding (AC #7)
12. Implement round_to_practical_value() function
13. Write failing test for ambiguous quantities (AC #8)
14. Implement ambiguous quantity detection
15. Write integration test for full aggregation flow

**Test Coverage Targets**:
- aggregation.rs: 85%
- conversion.rs: 90%
- fraction.rs: 85%
- ambiguous.rs: 80%
- Integration tests covering all 8 acceptance criteria

**Unit Test Examples**:
```rust
#[test]
fn test_normalize_ingredient_name_case_insensitive() {
    assert_eq!(normalize_ingredient_name("Chicken Breast"), "chicken breast");
    assert_eq!(normalize_ingredient_name("ONIONS"), "onions");
}

#[test]
fn test_normalize_ingredient_name_trim_whitespace() {
    assert_eq!(normalize_ingredient_name("  tomatoes  "), "tomatoes");
    assert_eq!(normalize_ingredient_name("garlic  cloves"), "garlic cloves");
}

#[test]
fn test_aggregate_same_unit() {
    let ing1 = Ingredient { name: "chicken breast".to_string(), quantity: "2".to_string(), unit: Unit::Pound };
    let ing2 = Ingredient { name: "chicken breast".to_string(), quantity: "1".to_string(), unit: Unit::Pound };

    let result = aggregate_same_unit(vec![ing1, ing2]);

    assert_eq!(result.quantity, "3");
    assert_eq!(result.unit, Unit::Pound);
}

#[test]
fn test_convert_cups_to_ml() {
    let result = convert_unit(1.0, Unit::Cup, Unit::Milliliter);
    assert_eq!(result, Some(240.0));
}

#[test]
fn test_aggregate_compatible_units() {
    let ing1 = Ingredient { quantity: "1".to_string(), unit: Unit::Cup };
    let ing2 = Ingredient { quantity: "240".to_string(), unit: Unit::Milliliter };

    let result = aggregate_compatible_units(vec![ing1, ing2]);

    assert_eq!(result.quantity, "2"); // 1 cup + 240ml = 2 cups
    assert_eq!(result.unit, Unit::Cup);
}

#[test]
fn test_incompatible_units_separate() {
    let ing1 = Ingredient { name: "onion".to_string(), quantity: "1".to_string(), unit: Unit::Whole };
    let ing2 = Ingredient { name: "onion".to_string(), quantity: "1".to_string(), unit: Unit::Cup };

    assert!(!are_units_compatible(&Unit::Whole, &Unit::Cup));

    // Should return 2 separate line items
    let results = aggregate_ingredient_group(vec![ing1, ing2]);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_fraction_addition() {
    let frac1 = parse_quantity("1/2").unwrap();
    let frac2 = parse_quantity("1/4").unwrap();

    let sum = frac1 + frac2;

    assert_eq!(format_quantity(sum), "3/4");
}

#[test]
fn test_practical_rounding() {
    let value = Fraction::from(2.347);
    let rounded = round_to_practical_value(value);

    // Should round to 2 1/3
    assert_eq!(format_quantity(rounded), "2 1/3");
}

#[test]
fn test_ambiguous_quantity_detected() {
    assert!(is_ambiguous_quantity("a pinch"));
    assert!(is_ambiguous_quantity("to taste"));
    assert!(is_ambiguous_quantity("dash"));
    assert!(!is_ambiguous_quantity("2 cups"));
}
```

**Integration Test Example**:
```rust
#[tokio::test]
async fn test_full_ingredient_aggregation_flow() {
    // Setup: 3 recipes with overlapping ingredients
    let recipe1 = Recipe {
        ingredients: vec![
            Ingredient { name: "Chicken Breast".to_string(), quantity: "2".to_string(), unit: Unit::Pound },
            Ingredient { name: "Onions".to_string(), quantity: "1".to_string(), unit: Unit::Whole },
            Ingredient { name: "Olive Oil".to_string(), quantity: "2".to_string(), unit: Unit::Tablespoon },
        ],
    };

    let recipe2 = Recipe {
        ingredients: vec![
            Ingredient { name: "chicken breast".to_string(), quantity: "1".to_string(), unit: Unit::Pound },
            Ingredient { name: "Onions".to_string(), quantity: "1/2".to_string(), unit: Unit::Cup }, // Incompatible with "whole"
            Ingredient { name: "Olive Oil".to_string(), quantity: "1".to_string(), unit: Unit::Tablespoon },
        ],
    };

    let recipe3 = Recipe {
        ingredients: vec![
            Ingredient { name: "Milk".to_string(), quantity: "1".to_string(), unit: Unit::Cup },
            Ingredient { name: "Milk".to_string(), quantity: "240".to_string(), unit: Unit::Milliliter }, // Should aggregate to 2 cups
            Ingredient { name: "Salt".to_string(), quantity: "to taste".to_string(), unit: Unit::ToTaste }, // Ambiguous
        ],
    };

    // Action: Aggregate ingredients
    let service = IngredientAggregationService::new();
    let aggregated = service.aggregate_ingredients(vec![recipe1, recipe2, recipe3]);

    // Assert: Verify aggregation results

    // Chicken Breast: 2 + 1 = 3 lbs
    let chicken = aggregated.iter().find(|i| i.name.contains("chicken")).unwrap();
    assert_eq!(chicken.quantity, "3");
    assert_eq!(chicken.unit, Unit::Pound);

    // Onions: 1 whole AND 1/2 cup (incompatible, separate line items)
    let onion_whole = aggregated.iter().find(|i| i.name.contains("onion") && i.unit == Unit::Whole).unwrap();
    assert_eq!(onion_whole.quantity, "1");

    let onion_cup = aggregated.iter().find(|i| i.name.contains("onion") && i.unit == Unit::Cup).unwrap();
    assert_eq!(onion_cup.quantity, "1/2");

    // Olive Oil: 2 + 1 = 3 tbsp
    let oil = aggregated.iter().find(|i| i.name.contains("olive")).unwrap();
    assert_eq!(oil.quantity, "3");
    assert_eq!(oil.unit, Unit::Tablespoon);

    // Milk: 1 cup + 240ml = 2 cups (compatible units)
    let milk = aggregated.iter().find(|i| i.name.contains("milk")).unwrap();
    assert_eq!(milk.quantity, "2");
    assert_eq!(milk.unit, Unit::Cup);

    // Salt: Flagged as ambiguous
    let salt = aggregated.iter().find(|i| i.name.contains("salt")).unwrap();
    assert!(salt.is_ambiguous);
}
```

### Project Structure Notes

**Alignment with solution-architecture.md**:

This story implements the core domain logic for the shopping crate, specifically the ingredient aggregation algorithm referenced in section 11.3 (Key Integrations - Inter-Domain Communication).

**Module Organization**:
- Shopping crate: New `aggregation`, `conversion`, `fraction`, `ambiguous` modules
- Algorithm complexity: O(n * m) where n = recipes, m = ingredients per recipe (acceptable for MVP <100 recipes)
- Unit conversion: O(1) lookup via HashMap
- Fraction arithmetic: Handled by `fraction` crate (precise rational number arithmetic)

**Naming Conventions**:
- Service: `IngredientAggregationService` (PascalCase)
- Functions: `aggregate_ingredients()`, `normalize_ingredient_name()` (snake_case)
- Modules: `aggregation`, `conversion`, `fraction`, `ambiguous` (snake_case)
- Enums: `Unit` (PascalCase), variants: `Cup`, `Milliliter` (PascalCase)

**Detected Conflicts/Variances**:
- Recipe ingredient storage format not yet defined in Epic 2 implementation
- Assumption: Ingredient struct has `name: String`, `quantity: String`, `unit: String` fields
- Resolution: Verify with Story 2.1 implementation, adjust aggregation logic if needed

**Lessons Learned from Story 4.1**:
- Shopping list generation depends on ingredient aggregation algorithm
- This story provides the core algorithm used by GenerateShoppingList command
- Algorithm must be testable in isolation (domain service pattern)
- Use evento executor pattern for database queries within aggregation

### References

**Technical Specifications**:
- [Source: docs/tech-spec-epic-4.md#Ingredient Aggregation Algorithm] - Algorithm overview, complexity analysis
- [Source: docs/solution-architecture.md#3.2 Data Models and Relationships] - Ingredient schema
- [Source: docs/solution-architecture.md#11.1 Domain Crate Structure] - Shopping crate organization

**Epic Context**:
- [Source: docs/epics.md#Story 4.11] - User story, acceptance criteria, technical notes
- [Source: docs/epics.md#Epic 4: Shopping and Preparation Orchestration] - Shopping list goals

**Related Stories**:
- [Source: docs/stories/story-2.1.md] - Create Recipe - prerequisite, defines ingredient data structure
- [Source: docs/stories/story-4.1.md] - Generate Weekly Shopping List - uses aggregation algorithm
- [Source: docs/stories/story-4.2.md] - Category-Based Ingredient Grouping - category assignment logic

**External References**:
- [fraction crate documentation](https://docs.rs/fraction/latest/fraction/)
- [lazy_static crate documentation](https://docs.rs/lazy_static/latest/lazy_static/)
- [Unit conversion standards (NIST)](https://www.nist.gov/pml/owm/metric-si/unit-conversion)

## Dev Agent Record

### Context Reference

- /home/snapiz/projects/github/timayz/imkitchen/docs/story-context-4.11.xml

### Agent Model Used

Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| Date | Author | Change Summary |
|------|--------|----------------|
| 2025-10-18 | Bob (SM) | Initial story creation from epics.md and tech-spec-epic-4.md |
| 2025-10-18 | Bob (SM) | Story context generated - story-context-4.11.xml created |
| 2025-10-18 | Jonathan | Status updated to Approved |
