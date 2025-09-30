use chrono::{DateTime, Utc};
use imkitchen_recipe::domain::{
    Difficulty, Ingredient, IngredientParser, Instruction, NutritionalCalculator, Recipe,
    RecipeCategory, RecipeDifficultyCalculator,
};
use uuid::Uuid;
use validator::Validate;

#[cfg(test)]
mod recipe_tests {
    use super::*;

    #[test]
    fn test_recipe_creation_with_valid_data() {
        let recipe_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let now = Utc::now();

        let ingredient =
            Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None).unwrap();

        let instruction = Instruction::new(1, "Mix flour with water".to_string(), Some(5)).unwrap();

        let recipe = Recipe {
            recipe_id,
            title: "Simple Bread".to_string(),
            ingredients: vec![ingredient],
            instructions: vec![instruction],
            prep_time_minutes: 15,
            cook_time_minutes: 45,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Bread,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec!["bread".to_string(), "simple".to_string()],
            created_at: now,
        };

        assert!(recipe.validate().is_ok());
        assert_eq!(recipe.title, "Simple Bread");
        assert_eq!(recipe.ingredients.len(), 1);
        assert_eq!(recipe.instructions.len(), 1);
        assert_eq!(recipe.prep_time_minutes, 15);
        assert_eq!(recipe.cook_time_minutes, 45);
    }

    #[test]
    fn test_recipe_title_validation() {
        let recipe_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let now = Utc::now();

        // Test empty title (should fail)
        let recipe_empty_title = Recipe {
            recipe_id,
            title: "".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 15,
            cook_time_minutes: 45,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec![],
            created_at: now,
        };

        assert!(recipe_empty_title.validate().is_err());

        // Test title too long (should fail)
        let long_title = "a".repeat(201);
        let recipe_long_title = Recipe {
            recipe_id,
            title: long_title,
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 15,
            cook_time_minutes: 45,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec![],
            created_at: now,
        };

        assert!(recipe_long_title.validate().is_err());

        // Test valid title (should pass)
        let valid_ingredient =
            Ingredient::new("Test Ingredient".to_string(), 1.0, "cup".to_string(), None).unwrap();

        let valid_instruction =
            Instruction::new(1, "Test instruction".to_string(), Some(5)).unwrap();

        let recipe_valid_title = Recipe {
            recipe_id,
            title: "Valid Recipe Title".to_string(),
            ingredients: vec![valid_ingredient],
            instructions: vec![valid_instruction],
            prep_time_minutes: 15,
            cook_time_minutes: 45,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec![],
            created_at: now,
        };

        assert!(recipe_valid_title.validate().is_ok());
    }

    #[test]
    fn test_recipe_time_validation() {
        let recipe_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let now = Utc::now();

        // Test zero prep time (should fail)
        let recipe_zero_prep = Recipe {
            recipe_id,
            title: "Test Recipe".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 0,
            cook_time_minutes: 45,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec![],
            created_at: now,
        };

        assert!(recipe_zero_prep.validate().is_err());

        // Test zero cook time (should fail)
        let recipe_zero_cook = Recipe {
            recipe_id,
            title: "Test Recipe".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 15,
            cook_time_minutes: 0,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            rating: 0.0,
            review_count: 0,
            created_by,
            is_public: true,
            tags: vec![],
            created_at: now,
        };

        assert!(recipe_zero_cook.validate().is_err());
    }

    #[test]
    fn test_ingredient_validation() {
        // Test valid ingredient
        let ingredient = Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None);
        assert!(ingredient.is_ok());

        // Test empty ingredient name (should fail)
        let empty_name = Ingredient::new("".to_string(), 2.0, "cups".to_string(), None);
        assert!(empty_name.is_err());

        // Test negative quantity (should fail)
        let negative_quantity =
            Ingredient::new("Sugar".to_string(), -1.0, "cups".to_string(), None);
        assert!(negative_quantity.is_err());

        // Test zero quantity (should fail)
        let zero_quantity = Ingredient::new("Salt".to_string(), 0.0, "teaspoons".to_string(), None);
        assert!(zero_quantity.is_err());
    }

    #[test]
    fn test_instruction_validation() {
        // Test valid instruction
        let instruction = Instruction::new(1, "Mix ingredients well".to_string(), Some(5));
        assert!(instruction.is_ok());

        // Test empty instruction text (should fail)
        let empty_text = Instruction::new(1, "".to_string(), Some(5));
        assert!(empty_text.is_err());

        // Test whitespace-only instruction text (should fail)
        let whitespace_text = Instruction::new(1, "   ".to_string(), Some(5));
        assert!(whitespace_text.is_err());
    }

    #[test]
    fn test_difficulty_levels() {
        let difficulties = vec![Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        assert_eq!(difficulties.len(), 3);
    }

    #[test]
    fn test_recipe_category_types() {
        let categories = vec![
            RecipeCategory::Appetizer,
            RecipeCategory::Main,
            RecipeCategory::Dessert,
            RecipeCategory::Beverage,
            RecipeCategory::Bread,
            RecipeCategory::Soup,
            RecipeCategory::Salad,
        ];
        assert_eq!(categories.len(), 7);
    }
}

#[cfg(test)]
mod domain_service_tests {
    use super::*;

    #[test]
    fn test_recipe_difficulty_calculator() {
        let calculator = RecipeDifficultyCalculator::new();

        // Simple recipe (few ingredients, short time)
        let simple_difficulty = calculator.calculate_difficulty(
            3,  // ingredient_count
            2,  // instruction_count
            15, // prep_time_minutes
            30, // cook_time_minutes
        );
        assert_eq!(simple_difficulty, Difficulty::Easy);

        // Complex recipe (many ingredients, long time)
        let complex_difficulty = calculator.calculate_difficulty(
            15,  // ingredient_count
            12,  // instruction_count
            60,  // prep_time_minutes
            180, // cook_time_minutes
        );
        assert_eq!(complex_difficulty, Difficulty::Hard);

        // Medium recipe
        let medium_difficulty = calculator.calculate_difficulty(
            8,  // ingredient_count
            6,  // instruction_count
            30, // prep_time_minutes
            60, // cook_time_minutes
        );
        assert_eq!(medium_difficulty, Difficulty::Medium);
    }

    #[test]
    fn test_ingredient_parser() {
        let parser = IngredientParser::new();

        // Test parsing "2 cups flour"
        let ingredient = parser.parse_ingredient_text("2 cups flour").unwrap();
        assert_eq!(ingredient.quantity, 2.0);
        assert_eq!(ingredient.unit, "cups");
        assert_eq!(ingredient.name, "flour");

        // Test parsing "1/2 teaspoon salt"
        let ingredient = parser.parse_ingredient_text("1/2 teaspoon salt").unwrap();
        assert_eq!(ingredient.quantity, 0.5);
        assert_eq!(ingredient.unit, "teaspoon");
        assert_eq!(ingredient.name, "salt");

        // Test parsing "3.5 lbs chicken breast"
        let ingredient = parser
            .parse_ingredient_text("3.5 lbs chicken breast")
            .unwrap();
        assert_eq!(ingredient.quantity, 3.5);
        assert_eq!(ingredient.unit, "lbs");
        assert_eq!(ingredient.name, "chicken breast");
    }

    #[test]
    fn test_nutritional_calculator() {
        let calculator = NutritionalCalculator::new();

        let ingredients = vec![
            Ingredient::new("flour".to_string(), 2.0, "cups".to_string(), None).unwrap(),
            Ingredient::new("sugar".to_string(), 1.0, "cup".to_string(), None).unwrap(),
            Ingredient::new("butter".to_string(), 0.5, "cup".to_string(), None).unwrap(),
        ];

        let nutrition = calculator.estimate_nutrition(&ingredients);

        // Basic nutritional calculation should return some values
        assert!(nutrition.calories > 0.0);
        assert!(nutrition.protein >= 0.0);
        assert!(nutrition.carbohydrates >= 0.0);
        assert!(nutrition.fat >= 0.0);
    }
}
