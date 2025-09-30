use super::value_objects::{Difficulty, Ingredient, NutritionalInfo};
use regex::Regex;

/// Service for calculating recipe difficulty based on complexity factors
pub struct RecipeDifficultyCalculator;

impl RecipeDifficultyCalculator {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_difficulty(
        &self,
        ingredient_count: usize,
        instruction_count: usize,
        prep_time_minutes: u32,
        cook_time_minutes: u32,
    ) -> Difficulty {
        let total_time = prep_time_minutes + cook_time_minutes;

        // Simple scoring algorithm based on complexity factors
        let mut complexity_score = 0;

        // Ingredient complexity
        if ingredient_count >= 10 {
            complexity_score += 3;
        } else if ingredient_count >= 6 {
            complexity_score += 2;
        } else if ingredient_count >= 4 {
            complexity_score += 1;
        }

        // Instruction complexity
        if instruction_count >= 10 {
            complexity_score += 3;
        } else if instruction_count >= 6 {
            complexity_score += 2;
        } else if instruction_count >= 4 {
            complexity_score += 1;
        }

        // Time complexity
        if total_time >= 120 {
            complexity_score += 3;
        } else if total_time >= 60 {
            complexity_score += 2;
        } else if total_time >= 30 {
            complexity_score += 1;
        }

        // Determine difficulty based on total score
        match complexity_score {
            0..=3 => Difficulty::Easy,
            4..=6 => Difficulty::Medium,
            _ => Difficulty::Hard,
        }
    }
}

impl Default for RecipeDifficultyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for parsing ingredient text into structured ingredient objects
pub struct IngredientParser {
    // Regex patterns for parsing common ingredient formats
    ingredient_pattern: Regex,
    fraction_pattern: Regex,
}

impl IngredientParser {
    pub fn new() -> Self {
        Self {
            // Pattern: "2 cups flour" or "1/2 teaspoon salt" or "3.5 lbs chicken breast"
            ingredient_pattern: Regex::new(r"^(\d+(?:\.\d+)?|\d+/\d+)\s+(\w+)\s+(.+)$").unwrap(),
            fraction_pattern: Regex::new(r"^(\d+)/(\d+)$").unwrap(),
        }
    }

    pub fn parse_ingredient_text(&self, text: &str) -> Result<Ingredient, String> {
        if let Some(captures) = self.ingredient_pattern.captures(text.trim()) {
            let quantity_str = captures.get(1).unwrap().as_str();
            let unit = captures.get(2).unwrap().as_str().to_string();
            let name = captures.get(3).unwrap().as_str().to_string();

            let quantity = if let Some(fraction_captures) =
                self.fraction_pattern.captures(quantity_str)
            {
                // Parse fraction
                let numerator: f64 = fraction_captures.get(1).unwrap().as_str().parse().unwrap();
                let denominator: f64 = fraction_captures.get(2).unwrap().as_str().parse().unwrap();
                numerator / denominator
            } else {
                // Parse decimal
                quantity_str
                    .parse()
                    .map_err(|_| "Invalid quantity format")?
            };

            Ingredient::new(name, quantity, unit, None)
                .map_err(|_| "Invalid ingredient data".to_string())
        } else {
            Err("Invalid ingredient format".to_string())
        }
    }
}

impl Default for IngredientParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for calculating basic nutritional information from ingredients
pub struct NutritionalCalculator {
    // Basic nutritional database (simplified for demo)
    nutrition_db: std::collections::HashMap<String, NutritionalInfo>,
}

impl NutritionalCalculator {
    pub fn new() -> Self {
        let mut nutrition_db = std::collections::HashMap::new();

        // Add some basic nutritional data per 100g/100ml
        nutrition_db.insert(
            "flour".to_string(),
            NutritionalInfo {
                calories: 364.0,
                protein: 10.3,
                carbohydrates: 76.3,
                fat: 1.0,
                fiber: 2.7,
                sugar: 0.3,
                sodium: 2.0,
            },
        );

        nutrition_db.insert(
            "sugar".to_string(),
            NutritionalInfo {
                calories: 387.0,
                protein: 0.0,
                carbohydrates: 100.0,
                fat: 0.0,
                fiber: 0.0,
                sugar: 100.0,
                sodium: 0.0,
            },
        );

        nutrition_db.insert(
            "butter".to_string(),
            NutritionalInfo {
                calories: 717.0,
                protein: 0.9,
                carbohydrates: 0.1,
                fat: 81.1,
                fiber: 0.0,
                sugar: 0.1,
                sodium: 643.0,
            },
        );

        Self { nutrition_db }
    }

    pub fn estimate_nutrition(&self, ingredients: &[Ingredient]) -> NutritionalInfo {
        let mut total_nutrition = NutritionalInfo::default();

        for ingredient in ingredients {
            if let Some(base_nutrition) = self.nutrition_db.get(&ingredient.name.to_lowercase()) {
                // Simple calculation assuming standard conversion rates
                // This is a basic approximation - real implementation would have
                // proper unit conversions and more comprehensive database
                let quantity_factor =
                    self.get_quantity_factor(&ingredient.unit, ingredient.quantity);

                total_nutrition.calories += base_nutrition.calories * quantity_factor;
                total_nutrition.protein += base_nutrition.protein * quantity_factor;
                total_nutrition.carbohydrates += base_nutrition.carbohydrates * quantity_factor;
                total_nutrition.fat += base_nutrition.fat * quantity_factor;
                total_nutrition.fiber += base_nutrition.fiber * quantity_factor;
                total_nutrition.sugar += base_nutrition.sugar * quantity_factor;
                total_nutrition.sodium += base_nutrition.sodium * quantity_factor;
            }
        }

        total_nutrition
    }

    fn get_quantity_factor(&self, unit: &str, quantity: f64) -> f64 {
        // Basic unit conversions to 100g equivalent
        // This is simplified - real implementation would have comprehensive unit conversion
        match unit.to_lowercase().as_str() {
            "cup" | "cups" => quantity * 1.25, // Approximate 125g per cup for flour
            "teaspoon" | "teaspoons" => quantity * 0.04, // Approximate 4g per teaspoon
            "tablespoon" | "tablespoons" => quantity * 0.12, // Approximate 12g per tablespoon
            "lb" | "lbs" | "pound" | "pounds" => quantity * 4.54, // 454g per pound
            "oz" | "ounce" | "ounces" => quantity * 0.28, // 28g per ounce
            "g" | "gram" | "grams" => quantity * 0.01, // Direct conversion to 100g
            "kg" | "kilogram" | "kilograms" => quantity * 10.0, // 1000g per kg
            _ => quantity * 1.0,               // Default assumption
        }
    }
}

impl Default for NutritionalCalculator {
    fn default() -> Self {
        Self::new()
    }
}
