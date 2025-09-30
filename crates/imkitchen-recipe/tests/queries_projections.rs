use chrono::Utc;
use imkitchen_recipe::{
    domain::{Difficulty, Ingredient, Instruction, RecipeCategory},
    projections::{
        RecipeDetailView, RecipeListView, RecipeSearchIndex, RecipeSummary, UserRecipeCollection,
    },
    queries::{
        PopularRecipesQuery, RecipeByIdQuery, RecipeSearchQuery, RecipesByTagQuery,
        RecipesByUserQuery,
    },
};
use uuid::Uuid;

#[cfg(test)]
mod query_tests {
    use super::*;

    #[test]
    fn test_recipe_by_id_query() {
        let recipe_id = Uuid::new_v4();
        let query = RecipeByIdQuery::new(recipe_id);

        assert_eq!(query.recipe_id, recipe_id);
    }

    #[test]
    fn test_recipes_by_user_query() {
        let user_id = Uuid::new_v4();
        let query = RecipesByUserQuery::new(user_id, true);

        assert_eq!(query.user_id, user_id);
        assert!(query.include_private);

        let public_only_query = RecipesByUserQuery::new(user_id, false);
        assert!(!public_only_query.include_private);
    }

    #[test]
    fn test_recipe_search_query_builder() {
        let user_id = Uuid::new_v4();

        let query = RecipeSearchQuery::new()
            .with_search_text("pasta".to_string())
            .with_category(RecipeCategory::Main)
            .with_difficulty(Difficulty::Easy)
            .with_max_prep_time(30)
            .with_max_cook_time(45)
            .with_tags(vec!["italian".to_string(), "quick".to_string()])
            .with_user_context(user_id)
            .with_pagination(10, 20);

        assert_eq!(query.search_text, Some("pasta".to_string()));
        assert_eq!(query.category, Some(RecipeCategory::Main));
        assert_eq!(query.difficulty, Some(Difficulty::Easy));
        assert_eq!(query.max_prep_time, Some(30));
        assert_eq!(query.max_cook_time, Some(45));
        assert_eq!(query.tags, vec!["italian".to_string(), "quick".to_string()]);
        assert_eq!(query.user_id, Some(user_id));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_recipe_search_query_defaults() {
        let query = RecipeSearchQuery::default();

        assert_eq!(query.search_text, None);
        assert_eq!(query.category, None);
        assert_eq!(query.difficulty, None);
        assert_eq!(query.max_prep_time, None);
        assert_eq!(query.max_cook_time, None);
        assert_eq!(query.tags, Vec::<String>::new());
        assert_eq!(query.user_id, None);
        assert_eq!(query.limit, Some(20));
        assert_eq!(query.offset, Some(0));
    }

    #[test]
    fn test_recipes_by_tag_query() {
        let user_id = Uuid::new_v4();

        let query = RecipesByTagQuery::new("vegetarian".to_string())
            .with_user_context(user_id)
            .with_pagination(15, 30);

        assert_eq!(query.tag, "vegetarian");
        assert_eq!(query.user_id, Some(user_id));
        assert_eq!(query.limit, Some(15));
        assert_eq!(query.offset, Some(30));
    }

    #[test]
    fn test_popular_recipes_query() {
        let query = PopularRecipesQuery::new()
            .with_category(RecipeCategory::Dessert)
            .with_time_range(7) // Last 7 days
            .with_pagination(5, 0);

        assert_eq!(query.category, Some(RecipeCategory::Dessert));
        assert_eq!(query.time_range_days, Some(7));
        assert_eq!(query.limit, Some(5));
        assert_eq!(query.offset, Some(0));
    }
}

#[cfg(test)]
mod projection_tests {
    use super::*;

    #[test]
    fn test_recipe_list_view() {
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let list_view = RecipeListView {
            recipe_id,
            title: "Test Recipe".to_string(),
            prep_time_minutes: 15,
            cook_time_minutes: 30,
            difficulty: Difficulty::Medium,
            category: RecipeCategory::Main,
            rating: 4.5,
            review_count: 12,
            created_by: user_id,
            is_public: true,
            tags: vec!["quick".to_string(), "healthy".to_string()],
            created_at: now,
            image_url: None,
        };

        assert_eq!(list_view.total_time_minutes(), 45);
        assert!(list_view.has_rating());
        assert_eq!(list_view.recipe_id, recipe_id);
        assert_eq!(list_view.title, "Test Recipe");
    }

    #[test]
    fn test_recipe_detail_view() {
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ingredient =
            Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None).unwrap();

        let instruction = Instruction::new(1, "Mix ingredients".to_string(), Some(5)).unwrap();

        let detail_view = RecipeDetailView {
            recipe_id,
            title: "Detailed Recipe".to_string(),
            ingredients: vec![ingredient.clone(), ingredient.clone(), ingredient.clone()],
            instructions: vec![instruction.clone(), instruction.clone()],
            prep_time_minutes: 20,
            cook_time_minutes: 40,
            difficulty: Difficulty::Hard,
            category: RecipeCategory::Bread,
            rating: 3.8,
            review_count: 25,
            created_by: user_id,
            is_public: false,
            tags: vec!["bread".to_string(), "homemade".to_string()],
            created_at: now,
            updated_at: None,
            image_url: None,
            nutritional_info: None,
        };

        assert_eq!(detail_view.total_time_minutes(), 60);
        assert_eq!(detail_view.ingredient_count(), 3);
        assert_eq!(detail_view.instruction_count(), 2);
        assert_eq!(detail_view.estimated_servings(), 1); // 3 ingredients = 1 serving
    }

    #[test]
    fn test_recipe_search_index() {
        let ingredient1 =
            Ingredient::new("Tomato".to_string(), 2.0, "pieces".to_string(), None).unwrap();

        let ingredient2 =
            Ingredient::new("Cheese".to_string(), 1.0, "cup".to_string(), None).unwrap();

        let instruction = Instruction::new(
            1,
            "Chop tomatoes and mix with cheese carefully".to_string(),
            Some(10),
        )
        .unwrap();

        let search_text = RecipeSearchIndex::build_search_text(
            "Tomato Cheese Salad",
            &[ingredient1, ingredient2],
            &[instruction],
            &["salad".to_string(), "quick".to_string()],
        );

        assert!(search_text.contains("tomato"));
        assert!(search_text.contains("cheese"));
        assert!(search_text.contains("salad"));
        assert!(search_text.contains("chop tomatoes and mix"));
        assert!(search_text.contains("quick"));

        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let search_index = RecipeSearchIndex {
            recipe_id,
            title: "Tomato Cheese Salad".to_string(),
            search_text: search_text.clone(),
            category: RecipeCategory::Salad,
            difficulty: Difficulty::Easy,
            prep_time_minutes: 10,
            cook_time_minutes: 0,
            tags: vec!["salad".to_string(), "quick".to_string()],
            ingredients_text: "tomato cheese".to_string(),
            instructions_text: "chop tomatoes and mix with cheese".to_string(),
            is_public: true,
            created_by: user_id,
            rating: 4.2,
            review_count: 8,
            created_at: now,
        };

        assert!(search_index.matches_search("tomato"));
        assert!(search_index.matches_search("CHEESE")); // Case insensitive
        assert!(search_index.matches_search("salad"));
        assert!(search_index.matches_search("quick"));
        assert!(!search_index.matches_search("pasta"));
    }

    #[test]
    fn test_recipe_summary() {
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let quick_summary = RecipeSummary {
            recipe_id,
            title: "Quick Snack".to_string(),
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Appetizer,
            total_time_minutes: 15,
            rating: 4.8,
            review_count: 10,
            ingredient_count: 3,
            tags: vec!["quick".to_string()],
            created_by: user_id,
            is_public: true,
            image_url: None,
        };

        assert!(quick_summary.is_quick_recipe());
        assert!(quick_summary.is_highly_rated());

        let slow_summary = RecipeSummary {
            recipe_id,
            title: "Slow Roast".to_string(),
            difficulty: Difficulty::Hard,
            category: RecipeCategory::Main,
            total_time_minutes: 180,
            rating: 3.5,
            review_count: 2,
            ingredient_count: 8,
            tags: vec!["roast".to_string()],
            created_by: user_id,
            is_public: true,
            image_url: None,
        };

        assert!(!slow_summary.is_quick_recipe());
        assert!(!slow_summary.is_highly_rated());
    }

    #[test]
    fn test_user_recipe_collection() {
        let user_id = Uuid::new_v4();
        let mut collection = UserRecipeCollection::new(user_id);

        assert_eq!(collection.user_id, user_id);
        assert_eq!(collection.total_count, 0);
        assert_eq!(collection.recipes.len(), 0);

        let recipe1 = RecipeSummary {
            recipe_id: Uuid::new_v4(),
            title: "Recipe 1".to_string(),
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            total_time_minutes: 30,
            rating: 4.0,
            review_count: 5,
            ingredient_count: 4,
            tags: vec!["main".to_string()],
            created_by: user_id,
            is_public: true,
            image_url: None,
        };

        let recipe2 = RecipeSummary {
            recipe_id: Uuid::new_v4(),
            title: "Recipe 2".to_string(),
            difficulty: Difficulty::Medium,
            category: RecipeCategory::Dessert,
            total_time_minutes: 60,
            rating: 4.5,
            review_count: 3,
            ingredient_count: 6,
            tags: vec!["dessert".to_string()],
            created_by: user_id,
            is_public: false,
            image_url: None,
        };

        collection.add_recipe(recipe1);
        collection.add_recipe(recipe2);

        assert_eq!(collection.total_count, 2);
        assert_eq!(collection.public_count, 1);
        assert_eq!(collection.private_count, 1);
        assert_eq!(collection.categories.len(), 2);
        assert_eq!(collection.categories[&RecipeCategory::Main], 1);
        assert_eq!(collection.categories[&RecipeCategory::Dessert], 1);
        assert_eq!(collection.average_rating, 4.25); // (4.0 + 4.5) / 2
    }
}
