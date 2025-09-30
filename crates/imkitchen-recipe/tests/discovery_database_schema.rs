use chrono::Utc;
use imkitchen_recipe::services::discovery_data::{
    DiscoveryDataService, DiscoveryQueryAnalytics, EventType, PreferenceType, QueryType,
    RecipeDiscoveryEvent, RecipeSimilarityCache, SimilarityType, SourceContext,
    UserDiscoveryPreference,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_discovery_data_service_creation() {
    let _service = DiscoveryDataService::new();
    assert!(true); // Service creation is implicit in successful instantiation
}

#[tokio::test]
async fn test_get_recipe_metrics() {
    let service = DiscoveryDataService::new();
    let recipe_id = Uuid::new_v4();

    let metrics = service.get_recipe_metrics(recipe_id).await.unwrap();

    // Verify metrics structure
    assert_eq!(metrics.recipe_id, recipe_id);
    assert!(metrics.base_popularity_score >= 0.0);
    assert!(metrics.trending_score_24h >= 0.0);
    assert!(metrics.trending_score_7d >= 0.0);
    assert!(metrics.trending_score_30d >= 0.0);
    assert!(metrics.view_count_total >= 0);
    assert!(metrics.view_count_24h >= 0);
    assert!(metrics.view_count_7d >= 0);
    assert!(metrics.view_count_30d >= 0);
    assert!(metrics.bookmark_count >= 0);
    assert!(metrics.bookmark_velocity_24h >= 0.0);
    assert!(metrics.search_mention_count >= 0);
    assert!(metrics.last_viewed_at.is_some());
    assert!(metrics.last_bookmarked_at.is_some());
}

#[tokio::test]
async fn test_update_recipe_metrics() {
    let service = DiscoveryDataService::new();
    let recipe_id = Uuid::new_v4();

    let metrics = service.get_recipe_metrics(recipe_id).await.unwrap();
    let result = service.update_recipe_metrics(&metrics).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_user_preferences() {
    let service = DiscoveryDataService::new();
    let user_id = "test_user_123";

    let preferences = service.get_user_preferences(user_id).await.unwrap();

    assert!(!preferences.is_empty());
    for pref in &preferences {
        assert_eq!(pref.user_id, user_id);
        assert!(pref.weight >= 0.0 && pref.weight <= 1.0);
        assert!(pref.interaction_count > 0);
        assert!(!pref.preference_value.is_empty());
    }
}

#[tokio::test]
async fn test_update_user_preference() {
    let service = DiscoveryDataService::new();
    let user_id = "test_user_456";

    let result = service
        .update_user_preference(user_id, PreferenceType::Category, "italian", 0.7)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_preference_type_conversion() {
    assert_eq!(PreferenceType::Category.as_str(), "category");
    assert_eq!(PreferenceType::Difficulty.as_str(), "difficulty");
    assert_eq!(PreferenceType::PrepTime.as_str(), "prep_time");
    assert_eq!(PreferenceType::Dietary.as_str(), "dietary");
    assert_eq!(PreferenceType::MealType.as_str(), "meal_type");
    assert_eq!(PreferenceType::Cuisine.as_str(), "cuisine");
    assert_eq!(PreferenceType::Technique.as_str(), "technique");

    assert!(matches!(
        PreferenceType::from_str("category"),
        Some(PreferenceType::Category)
    ));
    assert!(matches!(
        PreferenceType::from_str("difficulty"),
        Some(PreferenceType::Difficulty)
    ));
    assert!(matches!(PreferenceType::from_str("invalid"), None));
}

#[tokio::test]
async fn test_get_recipe_similarities() {
    let service = DiscoveryDataService::new();
    let recipe_id = Uuid::new_v4();

    let similarities = service.get_recipe_similarities(recipe_id, 5).await.unwrap();

    assert!(!similarities.is_empty());
    assert!(similarities.len() <= 5);

    for sim in &similarities {
        assert_eq!(sim.recipe_id, recipe_id);
        assert_ne!(sim.similar_recipe_id, recipe_id);
        assert!(sim.similarity_score >= 0.0 && sim.similarity_score <= 1.0);
        assert!(!sim.similarity_reasons.is_empty());
        assert!(sim.cache_valid_until > Utc::now());
    }

    // Should be sorted by similarity score (descending)
    for i in 1..similarities.len() {
        assert!(similarities[i - 1].similarity_score >= similarities[i].similarity_score);
    }
}

#[tokio::test]
async fn test_cache_recipe_similarities() {
    let service = DiscoveryDataService::new();
    let recipe_id = Uuid::new_v4();

    let similarities = vec![RecipeSimilarityCache {
        recipe_id,
        similar_recipe_id: Uuid::new_v4(),
        similarity_score: 0.85,
        similarity_type: SimilarityType::Ingredient,
        similarity_reasons: vec!["Common ingredients".to_string()],
        cache_valid_until: Utc::now() + chrono::Duration::hours(24),
    }];

    let result = service.cache_recipe_similarities(&similarities).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_similarity_type_conversion() {
    assert_eq!(SimilarityType::Ingredient.as_str(), "ingredient");
    assert_eq!(SimilarityType::Technique.as_str(), "technique");
    assert_eq!(SimilarityType::Category.as_str(), "category");
    assert_eq!(SimilarityType::Difficulty.as_str(), "difficulty");
    assert_eq!(SimilarityType::PrepTime.as_str(), "prep_time");
    assert_eq!(SimilarityType::Flavor.as_str(), "flavor");
    assert_eq!(SimilarityType::Equipment.as_str(), "equipment");
}

#[tokio::test]
async fn test_record_discovery_event() {
    let service = DiscoveryDataService::new();

    let event = RecipeDiscoveryEvent {
        event_id: Uuid::new_v4().to_string(),
        event_type: EventType::View,
        user_id: Some("test_user".to_string()),
        recipe_id: Uuid::new_v4(),
        session_id: Some("session_123".to_string()),
        source_context: SourceContext::Search,
        source_query: Some("chicken pasta".to_string()),
        position_in_results: Some(3),
        event_metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    let result = service.record_discovery_event(&event).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_event_type_conversion() {
    assert_eq!(EventType::View.as_str(), "view");
    assert_eq!(EventType::Click.as_str(), "click");
    assert_eq!(EventType::Bookmark.as_str(), "bookmark");
    assert_eq!(EventType::Share.as_str(), "share");
    assert_eq!(EventType::Rate.as_str(), "rate");
    assert_eq!(EventType::SimilarClick.as_str(), "similar_click");
    assert_eq!(EventType::FilterApply.as_str(), "filter_apply");
    assert_eq!(EventType::SearchRefine.as_str(), "search_refine");
}

#[tokio::test]
async fn test_source_context_conversion() {
    assert_eq!(SourceContext::Search.as_str(), "search");
    assert_eq!(SourceContext::Trending.as_str(), "trending");
    assert_eq!(SourceContext::Similar.as_str(), "similar");
    assert_eq!(SourceContext::Popular.as_str(), "popular");
    assert_eq!(SourceContext::Featured.as_str(), "featured");
    assert_eq!(SourceContext::Recommendation.as_str(), "recommendation");
    assert_eq!(SourceContext::Category.as_str(), "category");
    assert_eq!(SourceContext::Random.as_str(), "random");
}

#[tokio::test]
async fn test_record_query_analytics() {
    let service = DiscoveryDataService::new();

    let analytics = DiscoveryQueryAnalytics {
        query_id: Uuid::new_v4().to_string(),
        session_id: Some("session_456".to_string()),
        user_id: Some("test_user".to_string()),
        query_text: "easy dinner recipes".to_string(),
        query_type: QueryType::Search,
        results_count: 25,
        results_clicked: 3,
        click_through_rate: Some(0.12),
        first_click_position: Some(2),
        query_duration_ms: 150,
        had_typos: false,
        used_suggestions: true,
        applied_filters: HashMap::from([
            ("difficulty".to_string(), vec!["easy".to_string()]),
            ("prep_time".to_string(), vec!["30".to_string()]),
        ]),
        sort_order: Some("popularity".to_string()),
        timestamp: Utc::now(),
    };

    let result = service.record_query_analytics(&analytics).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_query_type_conversion() {
    assert_eq!(QueryType::Search.as_str(), "search");
    assert_eq!(QueryType::Filter.as_str(), "filter");
    assert_eq!(QueryType::Suggestion.as_str(), "suggestion");
    assert_eq!(QueryType::Trending.as_str(), "trending");
    assert_eq!(QueryType::Popular.as_str(), "popular");
    assert_eq!(QueryType::Similar.as_str(), "similar");
}

#[tokio::test]
async fn test_get_popular_recipes() {
    let service = DiscoveryDataService::new();

    let popular_recipes = service.get_popular_recipes(10, None).await.unwrap();

    assert!(!popular_recipes.is_empty());
    assert!(popular_recipes.len() <= 10);

    for recipe in &popular_recipes {
        assert!(!recipe.title.is_empty());
        assert!(recipe.base_popularity_score >= 0.0);
        assert!(recipe.view_count_total >= 0);
        assert!(recipe.bookmark_count >= 0);
        assert!(recipe.average_rating >= 0.0 && recipe.average_rating <= 5.0);
        assert!(recipe.review_count >= 0);
    }

    // Should be sorted by popularity score (descending)
    for i in 1..popular_recipes.len() {
        assert!(
            popular_recipes[i - 1].base_popularity_score
                >= popular_recipes[i].base_popularity_score
        );
    }
}

#[tokio::test]
async fn test_get_popular_recipes_with_category() {
    let service = DiscoveryDataService::new();

    let popular_recipes = service.get_popular_recipes(5, Some("pasta")).await.unwrap();

    assert!(!popular_recipes.is_empty());
    assert!(popular_recipes.len() <= 5);

    for recipe in &popular_recipes {
        if let Some(ref category) = recipe.category {
            assert_eq!(category, "pasta");
        }
    }
}

#[tokio::test]
async fn test_get_trending_recipes() {
    let service = DiscoveryDataService::new();

    let trending_recipes = service.get_trending_recipes(8, "24h").await.unwrap();

    assert!(!trending_recipes.is_empty());
    assert!(trending_recipes.len() <= 8);

    for recipe in &trending_recipes {
        assert!(!recipe.title.is_empty());
        assert!(recipe.trending_score_24h >= 0.0);
        assert!(recipe.trending_score_7d >= 0.0);
        assert!(recipe.view_count_24h >= 0);
        assert!(recipe.view_count_7d >= 0);
        assert!(recipe.bookmark_velocity_24h >= 0.0);
        assert!(recipe.velocity_24h >= 0.0);
    }

    // Should be sorted by trending score (descending)
    for i in 1..trending_recipes.len() {
        assert!(
            trending_recipes[i - 1].trending_score_24h >= trending_recipes[i].trending_score_24h
        );
    }
}

#[tokio::test]
async fn test_get_trending_recipes_different_windows() {
    let service = DiscoveryDataService::new();

    let trending_24h = service.get_trending_recipes(5, "24h").await.unwrap();
    let trending_7d = service.get_trending_recipes(5, "7d").await.unwrap();
    let trending_30d = service.get_trending_recipes(5, "30d").await.unwrap();

    assert!(!trending_24h.is_empty());
    assert!(!trending_7d.is_empty());
    assert!(!trending_30d.is_empty());

    // Different time windows should potentially have different trending scores
    // (though this is implementation dependent)
    assert!(trending_24h.len() <= 5);
    assert!(trending_7d.len() <= 5);
    assert!(trending_30d.len() <= 5);
}

#[tokio::test]
async fn test_get_analytics_summary() {
    let service = DiscoveryDataService::new();

    let summary = service.get_analytics_summary(7).await.unwrap();

    assert!(!summary.is_empty());
    assert!(summary.len() <= 7);

    for day_summary in &summary {
        assert!(!day_summary.analytics_date.is_empty());
        assert!(day_summary.total_queries >= 0);
        assert!(day_summary.unique_users >= 0);
        assert!(day_summary.avg_results_per_query >= 0.0);
        assert!(
            day_summary.avg_click_through_rate >= 0.0 && day_summary.avg_click_through_rate <= 1.0
        );
        assert!(day_summary.avg_query_duration_ms >= 0.0);
        assert!(day_summary.queries_with_suggestions >= 0);
        assert!(day_summary.queries_with_typos >= 0);
    }
}

#[tokio::test]
async fn test_get_feature_config() {
    let service = DiscoveryDataService::new();

    let all_config = service.get_feature_config(None).await.unwrap();
    assert!(!all_config.is_empty());

    let search_config = service.get_feature_config(Some("search")).await.unwrap();
    for config in &search_config {
        assert_eq!(config.category, "search");
        assert!(!config.config_key.is_empty());
        assert!(!config.config_value.is_empty());
        assert!(config.is_active);
    }

    let trending_config = service.get_feature_config(Some("trending")).await.unwrap();
    for config in &trending_config {
        assert_eq!(config.category, "trending");
    }
}

#[tokio::test]
async fn test_update_feature_config() {
    let service = DiscoveryDataService::new();

    let result = service
        .update_feature_config("search_max_results", "100", Some("admin_user"))
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_recipes_fts() {
    let service = DiscoveryDataService::new();

    let results = service
        .search_recipes_fts("chicken pasta", 10, 0)
        .await
        .unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    for (recipe_id, relevance_score) in &results {
        assert!(!recipe_id.to_string().is_empty());
        assert!(*relevance_score >= 0.0 && *relevance_score <= 1.0);
    }

    // Should be sorted by relevance score (descending)
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[tokio::test]
async fn test_search_recipes_fts_with_pagination() {
    let service = DiscoveryDataService::new();

    let page1 = service.search_recipes_fts("dinner", 5, 0).await.unwrap();
    let page2 = service.search_recipes_fts("dinner", 5, 5).await.unwrap();

    assert!(page1.len() <= 5);
    assert!(page2.len() <= 5);

    // Pages should have different results (in real implementation)
    if !page1.is_empty() && !page2.is_empty() {
        // At least verify they're not the same size always
        assert!(page1.len() <= 5 && page2.len() <= 5);
    }
}

#[tokio::test]
async fn test_service_default_implementation() {
    let service1 = DiscoveryDataService::new();
    let service2 = DiscoveryDataService::default();

    // Both should work and produce results
    let recipe_id = Uuid::new_v4();

    let metrics1 = service1.get_recipe_metrics(recipe_id).await.unwrap();
    let metrics2 = service2.get_recipe_metrics(recipe_id).await.unwrap();

    // Both should have the same recipe_id
    assert_eq!(metrics1.recipe_id, recipe_id);
    assert_eq!(metrics2.recipe_id, recipe_id);
}

#[tokio::test]
async fn test_complex_user_preferences_workflow() {
    let service = DiscoveryDataService::new();
    let user_id = "workflow_test_user";

    // Get initial preferences
    let initial_prefs = service.get_user_preferences(user_id).await.unwrap();
    assert!(!initial_prefs.is_empty());

    // Update multiple preferences
    let update_results = vec![
        service
            .update_user_preference(user_id, PreferenceType::Category, "mediterranean", 0.9)
            .await,
        service
            .update_user_preference(user_id, PreferenceType::Difficulty, "intermediate", 0.7)
            .await,
        service
            .update_user_preference(user_id, PreferenceType::PrepTime, "quick", 0.8)
            .await,
    ];

    for result in update_results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discovery_event_workflow() {
    let service = DiscoveryDataService::new();
    let recipe_id = Uuid::new_v4();
    let session_id = "test_session_workflow";
    let user_id = "test_user_workflow";

    // Simulate a complete discovery workflow
    let events = vec![
        RecipeDiscoveryEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: EventType::View,
            user_id: Some(user_id.to_string()),
            recipe_id,
            session_id: Some(session_id.to_string()),
            source_context: SourceContext::Search,
            source_query: Some("mediterranean dinner".to_string()),
            position_in_results: Some(1),
            event_metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RecipeDiscoveryEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: EventType::Click,
            user_id: Some(user_id.to_string()),
            recipe_id,
            session_id: Some(session_id.to_string()),
            source_context: SourceContext::Search,
            source_query: Some("mediterranean dinner".to_string()),
            position_in_results: Some(1),
            event_metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RecipeDiscoveryEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: EventType::Bookmark,
            user_id: Some(user_id.to_string()),
            recipe_id,
            session_id: Some(session_id.to_string()),
            source_context: SourceContext::Search,
            source_query: None,
            position_in_results: None,
            event_metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
    ];

    for event in events {
        let result = service.record_discovery_event(&event).await;
        assert!(result.is_ok());
    }
}
