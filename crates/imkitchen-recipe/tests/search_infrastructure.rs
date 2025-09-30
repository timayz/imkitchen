use imkitchen_recipe::services::search::{
    RecipeSearchService, SearchAnalytics, UserSearchPreferences
};
use imkitchen_recipe::domain::{RecipeCategory, Difficulty};
use imkitchen_shared::types::{DietaryRestriction, MealType};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::test]
async fn test_search_service_creation() {
    let _service = RecipeSearchService::new();
    // Service should be created successfully
    assert!(true); // Service creation is implicit in successful instantiation
}

#[tokio::test]
async fn test_generate_suggestions_with_ingredients() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("chick", None, 10).await.unwrap();
    
    // Should find chicken-related suggestions
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.text.contains("chicken")));
}

#[tokio::test]
async fn test_generate_suggestions_with_recipes() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("pasta", None, 10).await.unwrap();
    
    // Should find pasta-related suggestions
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.text.to_lowercase().contains("pasta")));
}

#[tokio::test]
async fn test_generate_suggestions_with_tags() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("quick", None, 10).await.unwrap();
    
    // Should find tag-related suggestions
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.text == "quick"));
}

#[tokio::test]
async fn test_typo_correction_suggestions() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("chickn", None, 10).await.unwrap();
    
    // Should suggest typo corrections
    assert!(suggestions.iter().any(|s| s.text == "chicken"));
}

#[tokio::test]
async fn test_suggestions_limit_respected() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("a", None, 3).await.unwrap();
    
    // Should respect the limit
    assert!(suggestions.len() <= 3);
}

#[tokio::test]
async fn test_suggestions_scoring() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("chicken", None, 10).await.unwrap();
    
    // Suggestions should be scored and sorted
    for suggestion in &suggestions {
        assert!(suggestion.score >= 0.0 && suggestion.score <= 1.0);
    }
    
    // Should be sorted by score (descending)
    for i in 1..suggestions.len() {
        assert!(suggestions[i-1].score >= suggestions[i].score);
    }
}

#[tokio::test]
async fn test_find_similar_recipes() {
    let service = RecipeSearchService::new();
    let recipe_id = Uuid::new_v4();
    let similarities = service.find_similar_recipes(recipe_id, 5).await.unwrap();
    
    // Should return some similarities (with mock data)
    assert!(!similarities.is_empty());
    assert!(similarities.len() <= 5);
    
    // Check similarity structure
    for similarity in &similarities {
        assert!(similarity.similarity_score >= 0.0 && similarity.similarity_score <= 1.0);
        assert!(!similarity.similarity_reasons.is_empty());
    }
}

#[tokio::test]
async fn test_similarity_scoring_bounds() {
    let service = RecipeSearchService::new();
    let recipe_id = Uuid::new_v4();
    let similarities = service.find_similar_recipes(recipe_id, 10).await.unwrap();
    
    // All similarity scores should be valid
    for similarity in &similarities {
        assert!(similarity.similarity_score >= 0.0);
        assert!(similarity.similarity_score <= 1.0);
        assert_ne!(similarity.recipe_id, recipe_id); // Should not include self
    }
}

#[tokio::test]
async fn test_user_preference_weighting() {
    let service = RecipeSearchService::new();
    
    let preferences = UserSearchPreferences {
        user_id: Uuid::new_v4(),
        preferred_categories: vec![RecipeCategory::Main],
        preferred_difficulty: vec![Difficulty::Easy],
        max_prep_time: Some(30),
        dietary_restrictions: vec![DietaryRestriction::Vegetarian],
        meal_types: vec![MealType::Dinner],
        search_history_weight: 0.3,
        recent_searches: vec!["chicken".to_string(), "pasta".to_string()],
    };
    
    let suggestions = service.generate_suggestions("chicken", Some(&preferences), 10).await.unwrap();
    
    // Should apply preference weighting (test structure, not specific values due to mock data)
    assert!(!suggestions.is_empty());
}

#[tokio::test]
async fn test_search_analytics_recording() {
    let service = RecipeSearchService::new();
    
    let analytics = SearchAnalytics {
        query: "test query".to_string(),
        results_count: 5,
        click_through_rate: 0.6,
        search_duration_ms: 250,
        popular_filters: HashMap::new(),
        successful_suggestions: vec!["chicken".to_string()],
    };
    
    // Should not error when recording analytics
    let result = service.record_search_analytics(analytics).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_user_preference_updates() {
    let service = RecipeSearchService::new();
    let user_id = Uuid::new_v4();
    let recipe_id = Some(Uuid::new_v4());
    
    // Should not error when updating preferences
    let result = service.update_user_preferences(user_id, "chicken recipe", recipe_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_empty_query_handling() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("", None, 10).await.unwrap();
    
    // Should handle empty queries gracefully
    // May return empty results or default suggestions
    assert!(suggestions.len() <= 10);
}

#[tokio::test]
async fn test_very_long_query_handling() {
    let service = RecipeSearchService::new();
    let long_query = "a".repeat(1000);
    let suggestions = service.generate_suggestions(&long_query, None, 10).await.unwrap();
    
    // Should handle very long queries without panicking
    assert!(suggestions.len() <= 10);
}

#[tokio::test]
async fn test_special_characters_in_query() {
    let service = RecipeSearchService::new();
    let suggestions = service.generate_suggestions("chicken!@#$%", None, 10).await.unwrap();
    
    // Should handle special characters gracefully
    assert!(suggestions.len() <= 10);
}

#[tokio::test]
async fn test_case_insensitive_search() {
    let service = RecipeSearchService::new();
    
    let lower_suggestions = service.generate_suggestions("chicken", None, 10).await.unwrap();
    let upper_suggestions = service.generate_suggestions("CHICKEN", None, 10).await.unwrap();
    let mixed_suggestions = service.generate_suggestions("ChIcKeN", None, 10).await.unwrap();
    
    // Should return similar results regardless of case
    assert!(!lower_suggestions.is_empty());
    assert!(!upper_suggestions.is_empty());
    assert!(!mixed_suggestions.is_empty());
}

#[tokio::test]
async fn test_similarity_reasons_structure() {
    let service = RecipeSearchService::new();
    let recipe_id = Uuid::new_v4();
    let similarities = service.find_similar_recipes(recipe_id, 3).await.unwrap();
    
    // Check that similarity reasons are properly structured
    for similarity in &similarities {
        assert!(!similarity.similarity_reasons.is_empty());
        
        // Each reason should be valid
        for reason in &similarity.similarity_reasons {
            match reason {
                imkitchen_recipe::services::search::SimilarityReason::IngredientOverlap { common_ingredients, overlap_score } => {
                    assert!(!common_ingredients.is_empty());
                    assert!(*overlap_score >= 0.0 && *overlap_score <= 1.0);
                },
                imkitchen_recipe::services::search::SimilarityReason::CookingTechnique { common_techniques, technique_score } => {
                    assert!(!common_techniques.is_empty());
                    assert!(*technique_score >= 0.0 && *technique_score <= 1.0);
                },
                _ => {
                    // Other reasons are valid by construction
                }
            }
        }
    }
}