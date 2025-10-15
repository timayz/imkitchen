//! Recipe tagging domain services
//!
//! This module contains stateless domain services for automatic recipe tagging:
//! - RecipeComplexityCalculator: Assigns complexity based on ingredients, steps, and advance prep
//! - CuisineInferenceService: Infers cuisine from ingredient patterns
//! - DietaryTagDetector: Detects vegetarian, vegan, and gluten-free tags
//!
//! These services are called during recipe creation and update to automatically tag recipes.

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::events::{Ingredient, InstructionStep};

/// Recipe complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Encode, Decode)]
pub enum Complexity {
    Simple,
    Moderate,
    Complex,
}

impl Complexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Complexity::Simple => "simple",
            Complexity::Moderate => "moderate",
            Complexity::Complex => "complex",
        }
    }
}

/// Recipe tags structure
#[derive(Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct RecipeTags {
    pub complexity: Option<Complexity>,
    pub cuisine: Option<String>,
    pub dietary_tags: Vec<String>, // e.g., ["vegetarian", "vegan", "gluten-free"]
    pub manual_override: bool,     // true if user manually set tags
}

/// Recipe complexity calculator
///
/// Calculates complexity based on weighted formula:
/// Score = (ingredients * 0.3) + (steps * 0.4) + (advance_prep_multiplier * 0.3)
///
/// advance_prep_multiplier:
/// - 0 if no advance prep
/// - 50 if < 4 hours
/// - 100 if >= 4 hours
///
/// Thresholds:
/// - Simple: score < 30
/// - Moderate: score 30-60
/// - Complex: score > 60
pub struct RecipeComplexityCalculator;

impl RecipeComplexityCalculator {
    /// Calculate complexity for a recipe
    pub fn calculate(
        ingredients: &[Ingredient],
        instructions: &[InstructionStep],
        advance_prep_hours: Option<u32>,
    ) -> Complexity {
        let ingredient_count = ingredients.len() as f32;
        let step_count = instructions.len() as f32;

        // Calculate advance prep multiplier
        let advance_prep_multiplier = match advance_prep_hours {
            None => 0.0,
            Some(hours) if hours < 4 => 50.0,
            Some(_) => 100.0, // >= 4 hours
        };

        // Weighted scoring formula
        let score = (ingredient_count * 0.3) + (step_count * 0.4) + (advance_prep_multiplier * 0.3);

        // Map score to complexity enum
        if score < 30.0 {
            Complexity::Simple
        } else if score <= 60.0 {
            Complexity::Moderate
        } else {
            Complexity::Complex
        }
    }
}

/// Cuisine inference service
///
/// Infers cuisine from ingredient patterns using keyword matching.
/// Returns None if no clear match is found (conservative approach).
pub struct CuisineInferenceService;

impl CuisineInferenceService {
    /// Infer cuisine from ingredients
    ///
    /// Uses case-insensitive pattern matching on ingredient names.
    /// Returns the best matching cuisine or None if no clear match.
    pub fn infer(ingredients: &[Ingredient]) -> Option<String> {
        // Convert ingredients to lowercase strings for matching
        let ingredient_names: Vec<String> =
            ingredients.iter().map(|i| i.name.to_lowercase()).collect();

        // Track cuisine match scores
        let mut scores: Vec<(&str, u32)> = Vec::new();

        // Italian cuisine patterns
        let italian_score = Self::score_cuisine(
            &ingredient_names,
            &[
                "tomato",
                "oregano",
                "basil",
                "pasta",
                "parmesan",
                "mozzarella",
                "olive oil",
            ],
            2, // minimum matches required
        );
        if italian_score > 0 {
            scores.push(("Italian", italian_score));
        }

        // Asian cuisine patterns
        let asian_score = Self::score_cuisine(
            &ingredient_names,
            &[
                "soy sauce",
                "ginger",
                "garlic",
                "rice",
                "noodles",
                "sesame",
                "mirin",
            ],
            2,
        );
        if asian_score > 0 {
            scores.push(("Asian", asian_score));
        }

        // Mexican cuisine patterns
        let mexican_score = Self::score_cuisine(
            &ingredient_names,
            &[
                "cumin", "chili", "cilantro", "beans", "tortilla", "lime", "avocado",
            ],
            2,
        );
        if mexican_score > 0 {
            scores.push(("Mexican", mexican_score));
        }

        // Indian cuisine patterns
        let indian_score = Self::score_cuisine(
            &ingredient_names,
            &[
                "curry",
                "turmeric",
                "garam masala",
                "cumin",
                "coriander",
                "cardamom",
            ],
            2,
        );
        if indian_score > 0 {
            scores.push(("Indian", indian_score));
        }

        // Mediterranean cuisine patterns
        let mediterranean_score = Self::score_cuisine(
            &ingredient_names,
            &[
                "olive oil",
                "lemon",
                "feta",
                "olives",
                "tahini",
                "chickpeas",
            ],
            2,
        );
        if mediterranean_score > 0 {
            scores.push(("Mediterranean", mediterranean_score));
        }

        // Return cuisine with highest score, or None if no matches
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        scores.first().map(|(cuisine, _)| cuisine.to_string())
    }

    /// Score a cuisine based on ingredient pattern matches
    ///
    /// Returns number of matching keywords found in ingredients.
    /// Returns 0 if fewer than min_matches are found (conservative approach).
    fn score_cuisine(ingredient_names: &[String], keywords: &[&str], min_matches: usize) -> u32 {
        let matches = keywords
            .iter()
            .filter(|keyword| ingredient_names.iter().any(|name| name.contains(*keyword)))
            .count();

        if matches >= min_matches {
            matches as u32
        } else {
            0 // Not enough matches to confidently assign this cuisine
        }
    }
}

/// Dietary tag detector
///
/// Detects dietary tags based on ingredient analysis:
/// - vegetarian: no meat/fish
/// - vegan: no animal products (meat, fish, dairy, eggs, honey)
/// - gluten-free: no wheat/flour products
///
/// Conservative approach: only assigns tags when confident (no restricted ingredients found).
/// False negatives acceptable, false positives unacceptable.
pub struct DietaryTagDetector;

impl DietaryTagDetector {
    /// Detect dietary tags from ingredients
    ///
    /// Returns a list of applicable dietary tags.
    pub fn detect(ingredients: &[Ingredient]) -> Vec<String> {
        let ingredient_names: Vec<String> =
            ingredients.iter().map(|i| i.name.to_lowercase()).collect();

        let mut tags = Vec::new();

        // Check for vegetarian (no meat/fish)
        if Self::is_vegetarian(&ingredient_names) {
            tags.push("vegetarian".to_string());

            // Check for vegan (no meat/fish/dairy/eggs/honey)
            if Self::is_vegan(&ingredient_names) {
                tags.push("vegan".to_string());
            }
        }

        // Check for gluten-free (no wheat/flour products)
        if Self::is_gluten_free(&ingredient_names) {
            tags.push("gluten-free".to_string());
        }

        tags
    }

    /// Check if recipe is vegetarian (no meat/fish)
    fn is_vegetarian(ingredient_names: &[String]) -> bool {
        let non_vegetarian_keywords = [
            "chicken", "beef", "pork", "fish", "lamb", "turkey", "seafood", "shrimp", "salmon",
            "tuna", "bacon", "ham", "sausage", "meat",
        ];

        !ingredient_names.iter().any(|name| {
            non_vegetarian_keywords
                .iter()
                .any(|keyword| name.contains(keyword))
        })
    }

    /// Check if recipe is vegan (no animal products)
    fn is_vegan(ingredient_names: &[String]) -> bool {
        let non_vegan_keywords = [
            "milk", "cheese", "butter", "cream", "eggs", "yogurt", "honey", "dairy", "whey",
            "casein", "gelatin",
        ];

        !ingredient_names.iter().any(|name| {
            non_vegan_keywords
                .iter()
                .any(|keyword| name.contains(keyword))
        })
    }

    /// Check if recipe is gluten-free (no wheat/flour products)
    fn is_gluten_free(ingredient_names: &[String]) -> bool {
        let gluten_keywords = [
            "flour",
            "wheat",
            "bread",
            "pasta",
            "noodles",
            "barley",
            "rye",
            "malt",
            "couscous",
            "breadcrumb",
        ];

        // Check for gluten-free variants (e.g., "gluten-free pasta")
        let has_gluten_free_variant = ingredient_names
            .iter()
            .any(|name| name.contains("gluten-free") || name.contains("gluten free"));

        if has_gluten_free_variant {
            return true; // Recipe explicitly uses gluten-free ingredients
        }

        // Otherwise check if any gluten-containing ingredients are present
        !ingredient_names
            .iter()
            .any(|name| gluten_keywords.iter().any(|keyword| name.contains(keyword)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ingredient(name: &str) -> Ingredient {
        Ingredient {
            name: name.to_string(),
            quantity: 1.0,
            unit: "cup".to_string(),
        }
    }

    fn make_instruction(number: u32) -> InstructionStep {
        InstructionStep {
            step_number: number,
            instruction_text: format!("Step {}", number),
            timer_minutes: None,
        }
    }

    #[test]
    fn test_complexity_simple() {
        let ingredients = vec![
            make_ingredient("flour"),
            make_ingredient("sugar"),
            make_ingredient("eggs"),
            make_ingredient("milk"),
            make_ingredient("vanilla"),
        ];
        let instructions = vec![
            make_instruction(1),
            make_instruction(2),
            make_instruction(3),
            make_instruction(4),
        ];

        let complexity = RecipeComplexityCalculator::calculate(&ingredients, &instructions, None);
        assert_eq!(complexity, Complexity::Simple);
    }

    #[test]
    fn test_complexity_moderate() {
        // Score = (30 * 0.3) + (50 * 0.4) + (0 * 0.3) = 9 + 20 + 0 = 29 → Should still be Simple
        // Let's use 32 ingredients and 50 steps to get score around 40
        // Score = (32 * 0.3) + (50 * 0.4) + (0 * 0.3) = 9.6 + 20 + 0 = 29.6 → Still Simple
        // Need to get score >= 30. Let's use 35 ingredients and 50 steps
        // Score = (35 * 0.3) + (50 * 0.4) + (0 * 0.3) = 10.5 + 20 + 0 = 30.5 → Moderate
        let mut ingredients = Vec::new();
        for i in 0..35 {
            ingredients.push(make_ingredient(&format!("ingredient{}", i)));
        }
        let mut instructions = Vec::new();
        for i in 0..50 {
            instructions.push(make_instruction(i + 1));
        }

        let complexity = RecipeComplexityCalculator::calculate(&ingredients, &instructions, None);
        assert_eq!(complexity, Complexity::Moderate);
    }

    #[test]
    fn test_complexity_complex_with_advance_prep() {
        // With advance prep >= 4 hours, multiplier = 100
        // Score = (12 * 0.3) + (6 * 0.4) + (100 * 0.3) = 3.6 + 2.4 + 30 = 36 → Moderate
        // Need score > 60. Let's increase to get above threshold
        // Score = (50 * 0.3) + (80 * 0.4) + (100 * 0.3) = 15 + 32 + 30 = 77 → Complex
        let mut ingredients = Vec::new();
        for i in 0..50 {
            ingredients.push(make_ingredient(&format!("ingredient{}", i)));
        }
        let mut instructions = Vec::new();
        for i in 0..80 {
            instructions.push(make_instruction(i + 1));
        }

        // With 4+ hours advance prep, this should be Complex
        let complexity =
            RecipeComplexityCalculator::calculate(&ingredients, &instructions, Some(4));
        assert_eq!(complexity, Complexity::Complex);
    }

    #[test]
    fn test_cuisine_italian() {
        let ingredients = vec![
            make_ingredient("tomato"),
            make_ingredient("oregano"),
            make_ingredient("pasta"),
        ];
        let cuisine = CuisineInferenceService::infer(&ingredients);
        assert_eq!(cuisine, Some("Italian".to_string()));
    }

    #[test]
    fn test_cuisine_asian() {
        let ingredients = vec![
            make_ingredient("soy sauce"),
            make_ingredient("ginger"),
            make_ingredient("rice"),
        ];
        let cuisine = CuisineInferenceService::infer(&ingredients);
        assert_eq!(cuisine, Some("Asian".to_string()));
    }

    #[test]
    fn test_cuisine_no_match() {
        let ingredients = vec![
            make_ingredient("flour"),
            make_ingredient("sugar"),
            make_ingredient("eggs"),
        ];
        let cuisine = CuisineInferenceService::infer(&ingredients);
        assert_eq!(cuisine, None);
    }

    #[test]
    fn test_dietary_vegetarian_and_vegan() {
        let ingredients = vec![
            make_ingredient("tomato"),
            make_ingredient("lettuce"),
            make_ingredient("olive oil"),
        ];
        let tags = DietaryTagDetector::detect(&ingredients);
        assert!(tags.contains(&"vegetarian".to_string()));
        assert!(tags.contains(&"vegan".to_string()));
    }

    #[test]
    fn test_dietary_vegetarian_not_vegan() {
        let ingredients = vec![
            make_ingredient("eggs"),
            make_ingredient("flour"),
            make_ingredient("sugar"),
        ];
        let tags = DietaryTagDetector::detect(&ingredients);
        assert!(tags.contains(&"vegetarian".to_string()));
        assert!(!tags.contains(&"vegan".to_string()));
    }

    #[test]
    fn test_dietary_not_vegetarian() {
        let ingredients = vec![
            make_ingredient("chicken"),
            make_ingredient("rice"),
            make_ingredient("vegetables"),
        ];
        let tags = DietaryTagDetector::detect(&ingredients);
        assert!(!tags.contains(&"vegetarian".to_string()));
        assert!(!tags.contains(&"vegan".to_string()));
    }

    #[test]
    fn test_dietary_gluten_free() {
        let ingredients = vec![
            make_ingredient("rice"),
            make_ingredient("chicken"),
            make_ingredient("vegetables"),
        ];
        let tags = DietaryTagDetector::detect(&ingredients);
        assert!(tags.contains(&"gluten-free".to_string()));
    }

    #[test]
    fn test_dietary_not_gluten_free() {
        let ingredients = vec![
            make_ingredient("flour"),
            make_ingredient("sugar"),
            make_ingredient("eggs"),
        ];
        let tags = DietaryTagDetector::detect(&ingredients);
        assert!(!tags.contains(&"gluten-free".to_string()));
    }
}
