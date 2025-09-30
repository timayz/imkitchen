use chrono::Utc;
use imkitchen_recipe::{
    commands::{CreateRecipeCommand, CreateRecipeParams, UpdateRecipeCommand},
    domain::{Difficulty, Ingredient, Instruction, Recipe, RecipeCategory, RecipeParams},
    projections::{RecipeDetailView, RecipeListView, RecipeSearchIndex},
    queries::RecipeSearchQuery,
};
use uuid::Uuid;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_recipe_workflow() {
        // Test the complete workflow from domain creation to projections

        // 1. Create a recipe using domain model
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let ingredients = vec![
            Ingredient::new(
                "All-purpose flour".to_string(),
                2.0,
                "cups".to_string(),
                None,
            )
            .unwrap(),
            Ingredient::new("Sugar".to_string(), 1.0, "cup".to_string(), None).unwrap(),
            Ingredient::new(
                "Eggs".to_string(),
                3.0,
                "large".to_string(),
                Some("room temperature".to_string()),
            )
            .unwrap(),
            Ingredient::new(
                "Butter".to_string(),
                0.5,
                "cup".to_string(),
                Some("melted".to_string()),
            )
            .unwrap(),
        ];

        let instructions = vec![
            Instruction::new(
                1,
                "Preheat oven to 350°F (175°C). Grease a 9x13 inch baking pan.".to_string(),
                Some(5),
            )
            .unwrap(),
            Instruction::new(
                2,
                "In a large bowl, combine flour and sugar.".to_string(),
                Some(2),
            )
            .unwrap(),
            Instruction::new(
                3,
                "Beat eggs in a separate bowl, then add to flour mixture.".to_string(),
                Some(3),
            )
            .unwrap(),
            Instruction::new(
                4,
                "Add melted butter and mix until just combined.".to_string(),
                Some(2),
            )
            .unwrap(),
            Instruction::new(
                5,
                "Pour batter into prepared pan and bake for 25-30 minutes.".to_string(),
                Some(30),
            )
            .unwrap(),
        ];

        let recipe = Recipe::new(RecipeParams {
            title: "Classic Vanilla Cake".to_string(),
            ingredients: ingredients.clone(),
            instructions: instructions.clone(),
            prep_time_minutes: 15,
            cook_time_minutes: 30,
            difficulty: Difficulty::Medium,
            category: RecipeCategory::Dessert,
            created_by: user_id,
            is_public: true,
            tags: vec![
                "cake".to_string(),
                "dessert".to_string(),
                "vanilla".to_string(),
                "baking".to_string(),
            ],
        })
        .unwrap();

        // Verify recipe properties
        assert_eq!(recipe.title, "Classic Vanilla Cake");
        assert_eq!(recipe.ingredients.len(), 4);
        assert_eq!(recipe.instructions.len(), 5);
        assert_eq!(recipe.total_time_minutes(), 45);
        assert_eq!(recipe.difficulty, Difficulty::Medium);
        assert_eq!(recipe.category, RecipeCategory::Dessert);
        assert!(recipe.is_public);

        // 2. Test command creation
        let create_command = CreateRecipeCommand::new(CreateRecipeParams {
            title: recipe.title.clone(),
            ingredients: recipe.ingredients.clone(),
            instructions: recipe.instructions.clone(),
            prep_time_minutes: recipe.prep_time_minutes,
            cook_time_minutes: recipe.cook_time_minutes,
            difficulty: recipe.difficulty,
            category: recipe.category,
            created_by: recipe.created_by,
            is_public: recipe.is_public,
            tags: recipe.tags.clone(),
        })
        .unwrap();

        assert_eq!(create_command.title, recipe.title);
        assert_eq!(create_command.ingredients.len(), 4);
        assert_eq!(create_command.instructions.len(), 5);

        // 3. Test update command
        let update_command = UpdateRecipeCommand::new(recipe_id, user_id)
            .with_title("Enhanced Vanilla Cake".to_string())
            .with_difficulty(Difficulty::Easy)
            .with_tags(vec![
                "cake".to_string(),
                "easy".to_string(),
                "vanilla".to_string(),
            ]);

        assert_eq!(update_command.recipe_id, recipe_id);
        assert_eq!(
            update_command.title,
            Some("Enhanced Vanilla Cake".to_string())
        );
        assert_eq!(update_command.difficulty, Some(Difficulty::Easy));

        // 4. Test search query building
        let search_query = RecipeSearchQuery::new()
            .with_search_text("vanilla cake".to_string())
            .with_category(RecipeCategory::Dessert)
            .with_difficulty(Difficulty::Medium)
            .with_max_prep_time(20)
            .with_max_cook_time(45)
            .with_tags(vec!["cake".to_string()])
            .with_pagination(10, 0);

        assert_eq!(search_query.search_text, Some("vanilla cake".to_string()));
        assert_eq!(search_query.category, Some(RecipeCategory::Dessert));
        assert_eq!(search_query.difficulty, Some(Difficulty::Medium));
        assert_eq!(search_query.max_prep_time, Some(20));
        assert_eq!(search_query.max_cook_time, Some(45));
        assert_eq!(search_query.tags, vec!["cake".to_string()]);
        assert_eq!(search_query.limit, Some(10));
        assert_eq!(search_query.offset, Some(0));

        // 5. Test projections
        let now = Utc::now();

        // Recipe List View projection
        let list_view = RecipeListView {
            recipe_id,
            title: recipe.title.clone(),
            prep_time_minutes: recipe.prep_time_minutes,
            cook_time_minutes: recipe.cook_time_minutes,
            difficulty: recipe.difficulty,
            category: recipe.category,
            rating: 4.5,
            review_count: 10,
            created_by: recipe.created_by,
            is_public: recipe.is_public,
            tags: recipe.tags.clone(),
            created_at: now,
            image_url: None,
        };

        assert_eq!(list_view.total_time_minutes(), 45);
        assert!(list_view.has_rating());

        // Recipe Detail View projection
        let detail_view = RecipeDetailView {
            recipe_id,
            title: recipe.title.clone(),
            ingredients: recipe.ingredients.clone(),
            instructions: recipe.instructions.clone(),
            prep_time_minutes: recipe.prep_time_minutes,
            cook_time_minutes: recipe.cook_time_minutes,
            difficulty: recipe.difficulty,
            category: recipe.category,
            rating: 4.5,
            review_count: 10,
            created_by: recipe.created_by,
            is_public: recipe.is_public,
            tags: recipe.tags.clone(),
            created_at: now,
            updated_at: None,
            image_url: None,
            nutritional_info: None,
        };

        assert_eq!(detail_view.total_time_minutes(), 45);
        assert_eq!(detail_view.ingredient_count(), 4);
        assert_eq!(detail_view.instruction_count(), 5);
        assert_eq!(detail_view.estimated_servings(), 2); // 4 ingredients = 2 servings

        // Recipe Search Index projection
        let search_text = RecipeSearchIndex::build_search_text(
            &recipe.title,
            &recipe.ingredients,
            &recipe.instructions,
            &recipe.tags,
        );

        assert!(search_text.contains("vanilla"));
        assert!(search_text.contains("cake"));
        assert!(search_text.contains("flour"));
        assert!(search_text.contains("sugar"));
        assert!(search_text.contains("eggs"));
        assert!(search_text.contains("butter"));
        assert!(search_text.contains("preheat"));
        assert!(search_text.contains("baking"));

        let search_index = RecipeSearchIndex {
            recipe_id,
            title: recipe.title.clone(),
            search_text: search_text.clone(),
            category: recipe.category,
            difficulty: recipe.difficulty,
            prep_time_minutes: recipe.prep_time_minutes,
            cook_time_minutes: recipe.cook_time_minutes,
            tags: recipe.tags.clone(),
            ingredients_text: "flour sugar eggs butter".to_string(),
            instructions_text: "preheat oven combine flour beat eggs add melted pour batter"
                .to_string(),
            is_public: recipe.is_public,
            created_by: recipe.created_by,
            rating: 4.5,
            review_count: 10,
            created_at: now,
        };

        // Test search functionality
        assert!(search_index.matches_search("vanilla"));
        assert!(search_index.matches_search("CAKE")); // Case insensitive
        assert!(search_index.matches_search("dessert"));
        assert!(search_index.matches_search("baking"));
        assert!(search_index.matches_search("flour"));
        assert!(!search_index.matches_search("chocolate")); // Not in recipe
    }

    #[test]
    fn test_recipe_validation_edge_cases() {
        let user_id = Uuid::new_v4();

        // Test empty title
        let result = Recipe::new(RecipeParams {
            title: String::new(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 1,
            cook_time_minutes: 1,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: false,
            tags: vec![],
        });
        assert!(result.is_err());

        // Test title too long (over 200 characters)
        let long_title = "a".repeat(201);
        let result = Recipe::new(RecipeParams {
            title: long_title,
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 1,
            cook_time_minutes: 1,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: false,
            tags: vec![],
        });
        assert!(result.is_err());

        // Test zero prep time
        let result = Recipe::new(RecipeParams {
            title: "Valid Title".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 0, // Invalid prep time
            cook_time_minutes: 1,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: false,
            tags: vec![],
        });
        assert!(result.is_err());

        // Test zero cook time
        let result = Recipe::new(RecipeParams {
            title: "Valid Title".to_string(),
            ingredients: vec![],
            instructions: vec![],
            prep_time_minutes: 1,
            cook_time_minutes: 0, // Invalid cook time
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: false,
            tags: vec![],
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_ingredient_validation_edge_cases() {
        // Test empty ingredient name
        let result = Ingredient::new(String::new(), 1.0, "cup".to_string(), None);
        assert!(result.is_err());

        // Test zero quantity
        let result = Ingredient::new("Flour".to_string(), 0.0, "cup".to_string(), None);
        assert!(result.is_err());

        // Test negative quantity
        let result = Ingredient::new("Flour".to_string(), -1.0, "cup".to_string(), None);
        assert!(result.is_err());

        // Test empty unit
        let result = Ingredient::new("Flour".to_string(), 1.0, String::new(), None);
        assert!(result.is_err());

        // Test valid ingredient with all fields
        let result = Ingredient::new(
            "All-purpose flour".to_string(),
            2.5,
            "cups".to_string(),
            Some("sifted".to_string()),
        );
        assert!(result.is_ok());
        let ingredient = result.unwrap();
        assert_eq!(ingredient.name, "All-purpose flour");
        assert_eq!(ingredient.quantity, 2.5);
        assert_eq!(ingredient.unit, "cups");
        assert_eq!(ingredient.notes, Some("sifted".to_string()));
    }

    #[test]
    fn test_instruction_validation_edge_cases() {
        // Test empty instruction text
        let result = Instruction::new(1, String::new(), Some(5));
        assert!(result.is_err());

        // Test whitespace-only instruction text
        let result = Instruction::new(1, "   \n\t   ".to_string(), Some(5));
        assert!(result.is_err());

        // Test valid instruction with minimal data
        let result = Instruction::new(1, "Mix ingredients.".to_string(), None);
        assert!(result.is_ok());
        let instruction = result.unwrap();
        assert_eq!(instruction.step_number, 1);
        assert_eq!(instruction.text, "Mix ingredients.");
        assert_eq!(instruction.estimated_minutes, None);

        // Test valid instruction with all fields
        let result = Instruction::new(
            2,
            "Bake in preheated oven until golden brown.".to_string(),
            Some(25),
        );
        assert!(result.is_ok());
        let instruction = result.unwrap();
        assert_eq!(instruction.step_number, 2);
        assert_eq!(
            instruction.text,
            "Bake in preheated oven until golden brown."
        );
        assert_eq!(instruction.estimated_minutes, Some(25));
    }
}
