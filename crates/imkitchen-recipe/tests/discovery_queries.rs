use imkitchen_recipe::queries::discovery::*;
use imkitchen_recipe::domain::discovery::*;
use imkitchen_recipe::projections::discovery::*;
use imkitchen_shared::types::{Difficulty, DietaryRestriction, MealType};
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_recipe_discovery_query() {
    let query_handler = RecipeDiscoveryQueryHandler::new();
    
    let query = RecipeDiscoveryQuery {
        query_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        filters: DiscoveryFilters {
            rating_threshold: Some(4.0),
            difficulty_levels: vec![Difficulty::Easy, Difficulty::Medium],
            max_prep_time: Some(30),
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            meal_types: vec![MealType::Dinner],
        },
        sort_order: SortingCriteria::HighestRated,
        page: 1,
        page_size: 20,
    };

    let result = query_handler.handle(query).await;
    assert!(result.is_ok());
    
    let browse_view = result.unwrap();
    assert_eq!(browse_view.page, 1);
    assert_eq!(browse_view.page_size, 20);
    assert!(browse_view.recipes.len() <= 20);
}

#[tokio::test]
async fn test_discovery_search_query() {
    let query_handler = DiscoverySearchQueryHandler::new();
    
    let query = DiscoverySearchQuery {
        query_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        search_criteria: SearchCriteria {
            query_text: "pasta".to_string(),
            search_type: SearchType::FullText,
            include_suggestions: true,
        },
        page: 1,
        page_size: 20,
    };

    let result = query_handler.handle(query).await;
    assert!(result.is_ok());
    
    let search_results = result.unwrap();
    assert_eq!(search_results.query_text, "pasta");
    assert_eq!(search_results.page, 1);
    assert!(search_results.total_results >= 0);
}

#[tokio::test]
async fn test_trending_recipes_query() {
    let query_handler = TrendingRecipesQueryHandler::new();
    
    let query = TrendingRecipesQuery {
        query_id: Uuid::new_v4(),
        time_window: "24h".to_string(),
        limit: 10,
        min_popularity_score: 50.0,
    };

    let result = query_handler.handle(query).await;
    assert!(result.is_ok());
    
    let trending_view = result.unwrap();
    assert_eq!(trending_view.time_window, "24h");
    assert!(trending_view.trending_recipes.len() <= 10);
    
    // Verify recipes are sorted by trending rank
    for window in trending_view.trending_recipes.windows(2) {
        assert!(window[0].trending_rank <= window[1].trending_rank);
    }
}

#[tokio::test]
async fn test_similar_recipes_query() {
    let query_handler = SimilarRecipesQueryHandler::new();
    
    let recipe_id = Uuid::new_v4();
    let query = SimilarRecipesQuery {
        query_id: Uuid::new_v4(),
        recipe_id,
        similarity_threshold: 0.7,
        limit: 5,
    };

    let result = query_handler.handle(query).await;
    assert!(result.is_ok());
    
    let similar_recipes = result.unwrap();
    assert_eq!(similar_recipes.original_recipe_id, recipe_id);
    assert!(similar_recipes.similar_recipes.len() <= 5);
    
    // Verify similarity scores are above threshold
    for recipe in &similar_recipes.similar_recipes {
        assert!(recipe.similarity_score >= 0.7);
    }
}

#[test]
fn test_recipe_browse_view_projection() {
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    let recipe_card = RecipeCard {
        recipe_id,
        title: "Vegetarian Pasta".to_string(),
        prep_time_minutes: 15,
        cook_time_minutes: 20,
        difficulty: Difficulty::Easy,
        rating_average: 4.5,
        rating_count: 25,
        image_url: Some("pasta.jpg".to_string()),
        created_by: user_id,
        tags: vec!["vegetarian".to_string(), "pasta".to_string()],
    };

    assert_eq!(recipe_card.recipe_id, recipe_id);
    assert_eq!(recipe_card.title, "Vegetarian Pasta");
    assert_eq!(recipe_card.total_time_minutes(), 35);
    assert!(recipe_card.is_highly_rated());
}

#[test]
fn test_search_suggestions_view() {
    let mut suggestions_view = SearchSuggestionsView {
        query_prefix: "pas".to_string(),
        suggestions: vec![
            SearchSuggestion {
                suggestion_text: "pasta".to_string(),
                suggestion_type: "ingredient".to_string(),
                frequency: 150,
            },
            SearchSuggestion {
                suggestion_text: "pastry".to_string(),
                suggestion_type: "category".to_string(),
                frequency: 80,
            },
        ],
        generated_at: Utc::now(),
    };

    // Test sorting by frequency
    suggestions_view.sort_by_frequency();
    assert_eq!(suggestions_view.suggestions[0].suggestion_text, "pasta");
    assert_eq!(suggestions_view.suggestions[1].suggestion_text, "pastry");
    
    // Test filtering by type
    let ingredient_suggestions = suggestions_view.filter_by_type("ingredient");
    assert_eq!(ingredient_suggestions.len(), 1);
    assert_eq!(ingredient_suggestions[0].suggestion_text, "pasta");
}

#[test]
fn test_trending_recipes_view() {
    let trending_view = TrendingRecipesView {
        time_window: "24h".to_string(),
        trending_recipes: vec![
            TrendingRecipe {
                recipe_id: Uuid::new_v4(),
                title: "Viral Pasta Recipe".to_string(),
                popularity_score: 95.5,
                trending_rank: 1,
                view_count_24h: 1500,
                rating_average: 4.8,
                time_weighted_score: 98.2,
            },
            TrendingRecipe {
                recipe_id: Uuid::new_v4(),
                title: "Popular Salad".to_string(),
                popularity_score: 87.3,
                trending_rank: 2,
                view_count_24h: 950,
                rating_average: 4.6,
                time_weighted_score: 89.1,
            },
        ],
        calculated_at: Utc::now(),
    };

    assert_eq!(trending_view.trending_recipes.len(), 2);
    assert_eq!(trending_view.trending_recipes[0].trending_rank, 1);
    assert_eq!(trending_view.trending_recipes[1].trending_rank, 2);
    
    // Test top trending recipe
    let top_recipe = trending_view.get_top_trending();
    assert!(top_recipe.is_some());
    assert_eq!(top_recipe.unwrap().title, "Viral Pasta Recipe");
}

#[test]
fn test_recipe_similarity_calculation() {
    let recipe1 = SimilarRecipe {
        recipe_id: Uuid::new_v4(),
        title: "Chicken Alfredo".to_string(),
        similarity_score: 0.85,
        similarity_reasons: vec![
            "Common ingredients: chicken, pasta".to_string(),
            "Similar cooking technique: sautéing".to_string(),
        ],
        ingredient_overlap: 0.75,
        technique_similarity: 0.90,
    };

    assert!(recipe1.is_highly_similar());
    assert_eq!(recipe1.similarity_reasons.len(), 2);
    assert!(recipe1.ingredient_overlap > 0.7);
    assert!(recipe1.technique_similarity > 0.8);
}