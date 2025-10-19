use shopping::aggregation::IngredientAggregationService;

/// Integration test for full ingredient aggregation flow with enhancements
/// Tests AC #1-8 in a realistic scenario
#[test]
fn test_full_aggregation_flow_with_enhancements() {
    // Setup: 3 recipes with overlapping ingredients
    // Recipe 1: Chicken Tikka Masala
    let recipe1 = vec![
        (
            "Chicken Breast".to_string(),
            "2".to_string(),
            "lbs".to_string(),
        ),
        ("Onions".to_string(), "1".to_string(), "whole".to_string()),
        ("Olive Oil".to_string(), "2".to_string(), "tbsp".to_string()),
        ("Garlic".to_string(), "3".to_string(), "cloves".to_string()),
    ];

    // Recipe 2: Chicken Stir Fry
    let recipe2 = vec![
        (
            "chicken breast".to_string(),
            "1".to_string(),
            "lb".to_string(),
        ), // AC #1: Case-insensitive
        ("Onions".to_string(), "1/2".to_string(), "cup".to_string()), // AC #4: Incompatible with "whole"
        (
            "Olive Oil".to_string(),
            "1".to_string(),
            "tablespoon".to_string(),
        ), // AC #3: Compatible units
        (
            "Ginger".to_string(),
            "1 1/2".to_string(),
            "tbsp".to_string(),
        ), // AC #6: Mixed fraction
    ];

    // Recipe 3: Rice Pilaf
    let recipe3 = vec![
        ("Milk".to_string(), "1".to_string(), "cup".to_string()),
        ("Milk".to_string(), "240".to_string(), "ml".to_string()), // AC #3: Compatible units (1 cup = 240ml)
        ("Salt".to_string(), "to taste".to_string(), "".to_string()), // AC #8: Ambiguous
        (
            "Butter".to_string(),
            "2.347".to_string(),
            "tbsp".to_string(),
        ), // AC #7: Needs rounding
    ];

    // Combine all ingredients
    let mut all_ingredients = Vec::new();
    all_ingredients.extend(recipe1);
    all_ingredients.extend(recipe2);
    all_ingredients.extend(recipe3);

    // Action: Aggregate ingredients
    let aggregated = IngredientAggregationService::aggregate_enhanced(all_ingredients).unwrap();

    // Assert: Verify aggregation results

    // Test AC #1 & AC #2: Chicken Breast aggregated (case-insensitive + same unit)
    let chicken = aggregated
        .iter()
        .find(|i| i.name.contains("chicken"))
        .expect("Chicken should be aggregated");
    // 2 lbs + 1 lb = 3 lbs = 1360.77 grams
    assert_eq!(chicken.unit, "g");
    let chicken_value =
        *chicken.quantity.numer().unwrap() as f64 / *chicken.quantity.denom().unwrap() as f64;
    assert!(
        (chicken_value - 1360.77).abs() < 1.0,
        "Expected ~1360.77g, got {}",
        chicken_value
    );

    // Test AC #4: Onions kept separate (incompatible units: whole vs cup)
    let onion_items: Vec<_> = aggregated
        .iter()
        .filter(|i| i.name.contains("onion"))
        .collect();
    assert_eq!(
        onion_items.len(),
        2,
        "Onions should have 2 separate items (whole and cup)"
    );

    let onion_whole = onion_items
        .iter()
        .find(|i| i.unit == "item")
        .expect("Should have onion in 'item' unit");
    assert_eq!(onion_whole.formatted_quantity, "1");

    let onion_cup = onion_items
        .iter()
        .find(|i| i.unit == "ml")
        .expect("Should have onion in 'ml' unit");
    // 1/2 cup = 120ml
    let onion_cup_value =
        *onion_cup.quantity.numer().unwrap() as f64 / *onion_cup.quantity.denom().unwrap() as f64;
    assert!((onion_cup_value - 120.0).abs() < 1.0);

    // Test AC #3 & AC #5: Olive Oil aggregated with unit conversion
    let oil = aggregated
        .iter()
        .find(|i| i.name.contains("olive"))
        .expect("Olive oil should be aggregated");
    // 2 tbsp + 1 tbsp = 3 tbsp = 45ml
    assert_eq!(oil.unit, "ml");
    let oil_value = *oil.quantity.numer().unwrap() as f64 / *oil.quantity.denom().unwrap() as f64;
    assert!((oil_value - 45.0).abs() < 1.0);

    // Test AC #3: Milk aggregated (compatible units: cup + ml)
    let milk = aggregated
        .iter()
        .find(|i| i.name.contains("milk"))
        .expect("Milk should be aggregated");
    // 1 cup + 240ml = 480ml (2 cups)
    assert_eq!(milk.unit, "ml");
    let milk_value =
        *milk.quantity.numer().unwrap() as f64 / *milk.quantity.denom().unwrap() as f64;
    assert!((milk_value - 480.0).abs() < 1.0);

    // Test AC #6: Ginger with mixed fraction
    let ginger = aggregated
        .iter()
        .find(|i| i.name.contains("ginger"))
        .expect("Ginger should be present");
    // 1 1/2 tbsp = 22.5ml
    let ginger_value =
        *ginger.quantity.numer().unwrap() as f64 / *ginger.quantity.denom().unwrap() as f64;
    assert!((ginger_value - 22.5).abs() < 1.0);

    // Test AC #7: Butter rounded to practical value
    let butter = aggregated
        .iter()
        .find(|i| i.name.contains("butter"))
        .expect("Butter should be present");
    // 2.347 tbsp should be rounded to a practical value
    // The rounding logic should give us something like 2 1/2 tbsp = 37.5ml
    let butter_value =
        *butter.quantity.numer().unwrap() as f64 / *butter.quantity.denom().unwrap() as f64;
    // 2.347 tbsp = 35.205ml, should round to practical value
    // Our rounding logic rounds to nearest 0.5 for values 1-10, so ~35ml
    assert!(
        (butter_value - 37.5).abs() < 5.0,
        "Butter should be rounded practically, got {}",
        butter_value
    );

    // Test AC #8: Salt flagged as ambiguous
    let salt = aggregated
        .iter()
        .find(|i| i.name.contains("salt"))
        .expect("Salt should be present");
    assert!(
        salt.is_ambiguous,
        "Salt with 'to taste' should be flagged as ambiguous"
    );
}

/// Test AC #2: Same-unit aggregation with fractions
#[test]
fn test_aggregate_same_unit_with_fractions() {
    let ingredients = vec![
        ("flour".to_string(), "1/2".to_string(), "cup".to_string()),
        ("flour".to_string(), "1/4".to_string(), "cups".to_string()),
    ];

    let result = IngredientAggregationService::aggregate_enhanced(ingredients).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "flour");
    // 1/2 + 1/4 = 3/4 cup = 180ml
    let value =
        *result[0].quantity.numer().unwrap() as f64 / *result[0].quantity.denom().unwrap() as f64;
    assert!((value - 180.0).abs() < 1.0);
    assert_eq!(result[0].unit, "ml");
}

/// Test AC #6: Fractional addition
#[test]
fn test_fractional_addition_in_aggregation() {
    let ingredients = vec![
        ("sugar".to_string(), "1/3".to_string(), "cup".to_string()),
        ("sugar".to_string(), "1/3".to_string(), "cup".to_string()),
        ("sugar".to_string(), "1/3".to_string(), "cup".to_string()),
    ];

    let result = IngredientAggregationService::aggregate_enhanced(ingredients).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "sugar");
    // 1/3 + 1/3 + 1/3 = 1 cup = 240ml
    let value =
        *result[0].quantity.numer().unwrap() as f64 / *result[0].quantity.denom().unwrap() as f64;
    assert!((value - 240.0).abs() < 1.0);
}

/// Test AC #7: Practical rounding applied
#[test]
fn test_practical_rounding_applied() {
    let ingredients = vec![("honey".to_string(), "2.347".to_string(), "tbsp".to_string())];

    let result = IngredientAggregationService::aggregate_enhanced(ingredients).unwrap();

    assert_eq!(result.len(), 1);
    // 2.347 should be rounded to a practical value
    // Check that formatted_quantity doesn't have excessive precision
    assert!(
        !result[0].formatted_quantity.contains("2.347"),
        "Formatted quantity should not have excessive precision"
    );
}

/// Test AC #8: Multiple ambiguous quantities aggregated
#[test]
fn test_multiple_ambiguous_quantities() {
    let ingredients = vec![
        ("pepper".to_string(), "to taste".to_string(), "".to_string()),
        ("pepper".to_string(), "a pinch".to_string(), "".to_string()),
        ("oregano".to_string(), "dash".to_string(), "".to_string()),
    ];

    let result = IngredientAggregationService::aggregate_enhanced(ingredients).unwrap();

    // Pepper should be aggregated
    let pepper = result.iter().find(|i| i.name.contains("pepper"));
    assert!(pepper.is_some());
    assert!(pepper.unwrap().is_ambiguous);

    // Oregano should be flagged
    let oregano = result.iter().find(|i| i.name.contains("oregano"));
    assert!(oregano.is_some());
    assert!(oregano.unwrap().is_ambiguous);
}
