use std::collections::HashMap;

/// Ingredient Aggregation Service
///
/// Stateless domain service that normalizes ingredient names, converts units,
/// and sums quantities for shopping list generation.
///
/// This service handles complex aggregation logic like:
/// - "chicken 2lbs" + "chicken 1lb" = "chicken 3lbs"
/// - "milk 1 cup" + "milk 240ml" = "milk 2 cups" (480ml converted to cups)
/// - "onion 1 whole" + "onion 1 cup diced" = 2 separate line items (incompatible units)
pub struct IngredientAggregationService;

impl IngredientAggregationService {
    /// Aggregate ingredients by normalizing names, converting units, and summing quantities
    ///
    /// Returns a list of (name, quantity, unit) tuples
    ///
    /// # Arguments
    /// * `ingredients` - List of (name, quantity, unit) tuples
    ///
    /// # Returns
    /// * Ok(Vec<(String, f32, String)>) - Aggregated ingredients
    /// * Err(String) - Aggregation error
    pub fn aggregate(
        ingredients: Vec<(String, f32, String)>,
    ) -> Result<Vec<(String, f32, String)>, String> {
        // Group ingredients by (normalized_name, normalized_unit)
        let mut groups: HashMap<(String, String), f32> = HashMap::new();

        for (name, quantity, unit) in ingredients {
            // Normalize ingredient name (lowercase, trim whitespace)
            let normalized_name = Self::normalize_name(&name);

            // Normalize unit and convert quantity to base unit
            let (normalized_unit, normalized_quantity) = Self::normalize_unit(&unit, quantity)?;

            // Group by (name, unit) and sum quantities
            let key = (normalized_name, normalized_unit);
            *groups.entry(key).or_insert(0.0) += normalized_quantity;
        }

        // Convert back to list
        let mut result: Vec<(String, f32, String)> = groups
            .into_iter()
            .map(|((name, unit), quantity)| (name, quantity, unit))
            .collect();

        // Sort by name for consistent ordering
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }

    /// Normalize ingredient name (lowercase, trim whitespace)
    fn normalize_name(name: &str) -> String {
        name.trim().to_lowercase()
    }

    /// Normalize unit and convert quantity to base unit
    ///
    /// Conversion table:
    /// - Volume: cups ↔ ml, tbsp ↔ tsp
    /// - Weight: lbs ↔ oz, g ↔ kg
    ///
    /// Base units:
    /// - Volume: ml
    /// - Weight: grams (g)
    /// - Count: item/whole/piece
    ///
    /// Incompatible units are kept separate (e.g., "1 whole" vs "1 cup diced")
    fn normalize_unit(unit: &str, quantity: f32) -> Result<(String, f32), String> {
        let normalized_unit = unit.trim().to_lowercase();

        // Volume conversions to ml (base unit)
        let (base_unit, base_quantity) = match normalized_unit.as_str() {
            // Volume units -> ml
            "cup" | "cups" => ("ml".to_string(), quantity * 240.0),
            "tbsp" | "tablespoon" | "tablespoons" => ("ml".to_string(), quantity * 15.0),
            "tsp" | "teaspoon" | "teaspoons" => ("ml".to_string(), quantity * 5.0),
            "ml" | "milliliter" | "milliliters" => ("ml".to_string(), quantity),
            "l" | "liter" | "liters" => ("ml".to_string(), quantity * 1000.0),

            // Weight units -> grams
            "g" | "gram" | "grams" => ("g".to_string(), quantity),
            "kg" | "kilogram" | "kilograms" => ("g".to_string(), quantity * 1000.0),
            "oz" | "ounce" | "ounces" => ("g".to_string(), quantity * 28.35),
            "lb" | "lbs" | "pound" | "pounds" => ("g".to_string(), quantity * 453.59),

            // Count units (no conversion)
            "whole" | "item" | "items" | "piece" | "pieces" | "clove" | "cloves" | "" => {
                ("item".to_string(), quantity)
            }

            // Unknown units kept as-is (incompatible with others)
            other => (other.to_string(), quantity),
        };

        Ok((base_unit, base_quantity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_same_ingredient_same_unit() {
        // Test: "chicken 2lbs" + "chicken 1lb" = "chicken 3lbs"
        let ingredients = vec![
            ("chicken".to_string(), 2.0, "lbs".to_string()),
            ("chicken".to_string(), 1.0, "lb".to_string()),
        ];

        let result = IngredientAggregationService::aggregate(ingredients).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "chicken");
        assert!((result[0].1 - 1360.77).abs() < 0.1); // 3 lbs ≈ 1360.77 grams
        assert_eq!(result[0].2, "g");
    }

    #[test]
    fn test_aggregate_unit_conversion() {
        // Test: "milk 1 cup" + "milk 240ml" = "milk 2 cups" (480ml)
        let ingredients = vec![
            ("milk".to_string(), 1.0, "cup".to_string()),
            ("milk".to_string(), 240.0, "ml".to_string()),
        ];

        let result = IngredientAggregationService::aggregate(ingredients).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "milk");
        assert_eq!(result[0].1, 480.0); // 2 cups = 480ml
        assert_eq!(result[0].2, "ml");
    }

    #[test]
    fn test_aggregate_incompatible_units_kept_separate() {
        // Test: "onion 1 whole" + "onion 1 cup diced" = 2 separate items (incompatible units)
        let ingredients = vec![
            ("onion".to_string(), 1.0, "whole".to_string()),
            ("onion".to_string(), 1.0, "cup".to_string()),
        ];

        let result = IngredientAggregationService::aggregate(ingredients).unwrap();

        // Should have 2 separate items (whole vs cup)
        assert_eq!(result.len(), 2);

        // Find the items (order may vary)
        let whole_item = result.iter().find(|item| item.2 == "item").unwrap();
        let cup_item = result.iter().find(|item| item.2 == "ml").unwrap();

        assert_eq!(whole_item.0, "onion");
        assert_eq!(whole_item.1, 1.0);

        assert_eq!(cup_item.0, "onion");
        assert_eq!(cup_item.1, 240.0); // 1 cup = 240ml
    }

    #[test]
    fn test_aggregate_different_ingredients_no_aggregation() {
        // Test: Different ingredients should not be aggregated
        let ingredients = vec![
            ("chicken".to_string(), 2.0, "lbs".to_string()),
            ("beef".to_string(), 1.0, "lb".to_string()),
        ];

        let result = IngredientAggregationService::aggregate(ingredients).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_aggregate_fractional_quantities() {
        // Test: "flour 1/2 cup" + "flour 1/4 cup" = "flour 3/4 cup"
        let ingredients = vec![
            ("flour".to_string(), 0.5, "cup".to_string()),
            ("flour".to_string(), 0.25, "cups".to_string()),
        ];

        let result = IngredientAggregationService::aggregate(ingredients).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "flour");
        assert_eq!(result[0].1, 180.0); // 0.75 cups = 180ml
        assert_eq!(result[0].2, "ml");
    }

    #[test]
    fn test_normalize_name() {
        assert_eq!(
            IngredientAggregationService::normalize_name("  Chicken  "),
            "chicken"
        );
        assert_eq!(
            IngredientAggregationService::normalize_name("TOMATO"),
            "tomato"
        );
    }

    #[test]
    fn test_normalize_unit_volume() {
        let (unit, qty) = IngredientAggregationService::normalize_unit("cup", 1.0).unwrap();
        assert_eq!(unit, "ml");
        assert_eq!(qty, 240.0);

        let (unit, qty) = IngredientAggregationService::normalize_unit("tbsp", 1.0).unwrap();
        assert_eq!(unit, "ml");
        assert_eq!(qty, 15.0);

        let (unit, qty) = IngredientAggregationService::normalize_unit("tsp", 1.0).unwrap();
        assert_eq!(unit, "ml");
        assert_eq!(qty, 5.0);
    }

    #[test]
    fn test_normalize_unit_weight() {
        let (unit, qty) = IngredientAggregationService::normalize_unit("lbs", 1.0).unwrap();
        assert_eq!(unit, "g");
        assert!((qty - 453.59).abs() < 0.1);

        let (unit, qty) = IngredientAggregationService::normalize_unit("oz", 1.0).unwrap();
        assert_eq!(unit, "g");
        assert!((qty - 28.35).abs() < 0.1);

        let (unit, qty) = IngredientAggregationService::normalize_unit("kg", 1.0).unwrap();
        assert_eq!(unit, "g");
        assert_eq!(qty, 1000.0);
    }

    #[test]
    fn test_normalize_unit_count() {
        let (unit, qty) = IngredientAggregationService::normalize_unit("whole", 1.0).unwrap();
        assert_eq!(unit, "item");
        assert_eq!(qty, 1.0);

        let (unit, qty) = IngredientAggregationService::normalize_unit("clove", 3.0).unwrap();
        assert_eq!(unit, "item");
        assert_eq!(qty, 3.0);
    }
}
