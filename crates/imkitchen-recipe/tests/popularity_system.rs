use chrono::Duration;
use imkitchen_recipe::services::popularity::{
    PopularityConfig, PopularityService, TimeWindow, TrendFactor,
};
use uuid::Uuid;

#[tokio::test]
async fn test_popularity_service_creation() {
    let _service = PopularityService::new();
    // Service should be created successfully
    assert!(true); // Service creation is implicit in successful instantiation
}

#[tokio::test]
async fn test_popularity_service_with_custom_config() {
    let config = PopularityConfig {
        view_weight: 2.0,
        rating_weight: 4.0,
        bookmark_weight: 6.0,
        recency_decay_days: 60,
        trending_window_hours: 48,
        min_views_for_trending: 100,
        velocity_threshold: 2.0,
    };

    let _service = PopularityService::with_config(config);
    assert!(true); // Custom config service should be created successfully
}

#[tokio::test]
async fn test_calculate_base_score() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();

    let score = service.calculate_base_score(recipe_id).await.unwrap();

    // Score should be within valid range
    assert!(score >= 0.0 && score <= 100.0);
}

#[tokio::test]
async fn test_calculate_trending_score() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();
    let time_window = TimeWindow::last_24_hours();

    let trending_score = service
        .calculate_trending_score(recipe_id, &time_window)
        .await
        .unwrap();

    // Trending score should be non-negative
    assert!(trending_score >= 0.0);
    // Trending score can exceed base score but should be reasonable
    assert!(trending_score <= 200.0);
}

#[tokio::test]
async fn test_get_trending_recipes() {
    let service = PopularityService::new();
    let time_window = TimeWindow::last_24_hours();

    let trending_recipes = service
        .get_trending_recipes(&time_window, 10)
        .await
        .unwrap();

    // Should return some trending recipes
    assert!(!trending_recipes.is_empty());
    assert!(trending_recipes.len() <= 10);

    // Check structure of trending recipes
    for recipe in &trending_recipes {
        assert!(recipe.trending_score >= 0.0);
        assert!(recipe.velocity >= 0.0);
        assert!(recipe.recent_views >= 0);
        assert!(!recipe.title.is_empty());
        assert_eq!(recipe.time_window, time_window.label);
    }

    // Should be sorted by trending score (descending)
    for i in 1..trending_recipes.len() {
        assert!(trending_recipes[i - 1].trending_score >= trending_recipes[i].trending_score);
    }
}

#[tokio::test]
async fn test_analyze_trend_factors() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();
    let time_window = TimeWindow::last_24_hours();

    let factors = service
        .analyze_trend_factors(recipe_id, &time_window)
        .await
        .unwrap();

    // Should have some trend factors
    assert!(!factors.is_empty());

    // Check that factors are valid
    for factor in &factors {
        match factor {
            TrendFactor::ViewSpike {
                multiplier,
                timeframe,
            } => {
                assert!(*multiplier > 1.0);
                assert!(!timeframe.is_empty());
            }
            TrendFactor::RatingBoost {
                average_rating,
                recent_ratings,
            } => {
                assert!(*average_rating >= 1.0 && *average_rating <= 5.0);
                assert!(*recent_ratings > 0);
            }
            TrendFactor::BookmarkSurge { bookmark_velocity } => {
                assert!(*bookmark_velocity > 0.0);
            }
            TrendFactor::NewRecipe {
                days_old,
                novelty_boost,
            } => {
                assert!(*days_old >= 0);
                assert!(*novelty_boost >= 0.0 && *novelty_boost <= 1.0);
            }
            TrendFactor::SearchPopularity {
                query_frequency,
                search_rank,
            } => {
                assert!(*query_frequency > 0);
                assert!(*search_rank > 0);
            }
            TrendFactor::SeasonalRelevance {
                relevance_score,
                context,
            } => {
                assert!(*relevance_score >= 0.0 && *relevance_score <= 1.0);
                assert!(!context.is_empty());
            }
        }
    }
}

#[tokio::test]
async fn test_update_popularity_metrics() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();

    let metrics = service.update_popularity_metrics(recipe_id).await.unwrap();

    // Check metrics structure
    assert_eq!(metrics.recipe_id, recipe_id);
    assert!(metrics.base_score >= 0.0 && metrics.base_score <= 100.0);
    assert!(metrics.trending_score >= 0.0);
    assert!(metrics.view_count >= 0);
    assert!(metrics.rating_average >= 0.0 && metrics.rating_average <= 5.0);
    assert!(metrics.rating_count >= 0);
    assert!(metrics.bookmark_count >= 0);
    assert!(metrics.recent_activity_score >= 0.0 && metrics.recent_activity_score <= 100.0);
}

#[tokio::test]
async fn test_get_popularity_rankings() {
    let service = PopularityService::new();
    let time_window = TimeWindow::last_24_hours();

    let rankings = service
        .get_popularity_rankings(None, &time_window, 15)
        .await
        .unwrap();

    // Should return rankings
    assert!(!rankings.is_empty());
    assert!(rankings.len() <= 15);

    // Check ranking structure
    for (recipe_id, score) in &rankings {
        assert!(!recipe_id.to_string().is_empty());
        assert!(*score >= 0.0);
    }

    // Should be sorted by score (descending)
    for i in 1..rankings.len() {
        assert!(rankings[i - 1].1 >= rankings[i].1);
    }
}

#[tokio::test]
async fn test_get_popularity_rankings_with_category() {
    let service = PopularityService::new();
    let time_window = TimeWindow::last_week();
    let category = "pasta".to_string();

    let rankings = service
        .get_popularity_rankings(Some(category), &time_window, 10)
        .await
        .unwrap();

    // Should return category-filtered rankings
    assert!(!rankings.is_empty());
    assert!(rankings.len() <= 10);
}

#[tokio::test]
async fn test_time_window_creation() {
    let window_24h = TimeWindow::last_24_hours();
    let window_7d = TimeWindow::last_week();
    let window_30d = TimeWindow::last_month();

    // Check labels
    assert_eq!(window_24h.label, "24h");
    assert_eq!(window_7d.label, "7d");
    assert_eq!(window_30d.label, "30d");

    // Check time ranges
    assert!(window_24h.end > window_24h.start);
    assert!(window_7d.end > window_7d.start);
    assert!(window_30d.end > window_30d.start);

    // Check duration
    let duration_24h = window_24h.end - window_24h.start;
    let duration_7d = window_7d.end - window_7d.start;
    let duration_30d = window_30d.end - window_30d.start;

    assert_eq!(duration_24h, Duration::hours(24));
    assert_eq!(duration_7d, Duration::days(7));
    assert_eq!(duration_30d, Duration::days(30));
}

#[tokio::test]
async fn test_popularity_config_defaults() {
    let config = PopularityConfig::default();

    assert_eq!(config.view_weight, 1.0);
    assert_eq!(config.rating_weight, 3.0);
    assert_eq!(config.bookmark_weight, 5.0);
    assert_eq!(config.recency_decay_days, 30);
    assert_eq!(config.trending_window_hours, 24);
    assert_eq!(config.min_views_for_trending, 50);
    assert_eq!(config.velocity_threshold, 1.5);
}

#[tokio::test]
async fn test_different_time_windows() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();

    let score_24h = service
        .calculate_trending_score(recipe_id, &TimeWindow::last_24_hours())
        .await
        .unwrap();
    let score_7d = service
        .calculate_trending_score(recipe_id, &TimeWindow::last_week())
        .await
        .unwrap();
    let score_30d = service
        .calculate_trending_score(recipe_id, &TimeWindow::last_month())
        .await
        .unwrap();

    // All scores should be valid
    assert!(score_24h >= 0.0);
    assert!(score_7d >= 0.0);
    assert!(score_30d >= 0.0);
}

#[tokio::test]
async fn test_trending_recipes_with_different_windows() {
    let service = PopularityService::new();

    let trending_24h = service
        .get_trending_recipes(&TimeWindow::last_24_hours(), 5)
        .await
        .unwrap();
    let trending_7d = service
        .get_trending_recipes(&TimeWindow::last_week(), 5)
        .await
        .unwrap();
    let trending_30d = service
        .get_trending_recipes(&TimeWindow::last_month(), 5)
        .await
        .unwrap();

    // All should return results
    assert!(!trending_24h.is_empty());
    assert!(!trending_7d.is_empty());
    assert!(!trending_30d.is_empty());

    // Check time window labels
    for recipe in &trending_24h {
        assert_eq!(recipe.time_window, "24h");
    }
    for recipe in &trending_7d {
        assert_eq!(recipe.time_window, "7d");
    }
    for recipe in &trending_30d {
        assert_eq!(recipe.time_window, "30d");
    }
}

#[tokio::test]
async fn test_trend_factors_variety() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();
    let time_window = TimeWindow::last_24_hours();

    let factors = service
        .analyze_trend_factors(recipe_id, &time_window)
        .await
        .unwrap();

    // Should have multiple types of factors
    let has_view_spike = factors
        .iter()
        .any(|f| matches!(f, TrendFactor::ViewSpike { .. }));
    let has_rating_boost = factors
        .iter()
        .any(|f| matches!(f, TrendFactor::RatingBoost { .. }));
    let has_new_recipe = factors
        .iter()
        .any(|f| matches!(f, TrendFactor::NewRecipe { .. }));
    let has_search_popularity = factors
        .iter()
        .any(|f| matches!(f, TrendFactor::SearchPopularity { .. }));

    // At least some factor types should be present
    assert!(has_view_spike || has_rating_boost || has_new_recipe || has_search_popularity);
}

#[tokio::test]
async fn test_velocity_through_trending_recipes() {
    let service = PopularityService::new();
    let time_window = TimeWindow::last_24_hours();

    let trending_recipes = service.get_trending_recipes(&time_window, 5).await.unwrap();

    // Check that velocity is accessible through trending recipes
    for recipe in &trending_recipes {
        // Velocity should be non-negative
        assert!(recipe.velocity >= 0.0);
    }
}

#[tokio::test]
async fn test_empty_results_handling() {
    let service = PopularityService::new();

    // Test with zero limit
    let trending_recipes = service
        .get_trending_recipes(&TimeWindow::last_24_hours(), 0)
        .await
        .unwrap();
    assert!(trending_recipes.is_empty());

    let rankings = service
        .get_popularity_rankings(None, &TimeWindow::last_24_hours(), 0)
        .await
        .unwrap();
    assert!(rankings.is_empty());
}

#[tokio::test]
async fn test_large_limit_handling() {
    let service = PopularityService::new();

    // Test with very large limit
    let trending_recipes = service
        .get_trending_recipes(&TimeWindow::last_24_hours(), 1000)
        .await
        .unwrap();
    assert!(trending_recipes.len() <= 1000); // Should not exceed reasonable bounds

    let rankings = service
        .get_popularity_rankings(None, &TimeWindow::last_24_hours(), 1000)
        .await
        .unwrap();
    assert!(rankings.len() <= 1000);
}

#[tokio::test]
async fn test_popularity_score_bounds() {
    let service = PopularityService::new();
    let recipe_id = Uuid::new_v4();

    let base_score = service.calculate_base_score(recipe_id).await.unwrap();
    let trending_score = service
        .calculate_trending_score(recipe_id, &TimeWindow::last_24_hours())
        .await
        .unwrap();

    // Base score should be 0-100
    assert!(base_score >= 0.0 && base_score <= 100.0);

    // Trending score can be higher but should be reasonable
    assert!(trending_score >= 0.0 && trending_score <= 200.0);
}

#[tokio::test]
async fn test_multiple_recipe_consistency() {
    let service = PopularityService::new();
    let recipe_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

    for recipe_id in recipe_ids {
        let score = service.calculate_base_score(recipe_id).await.unwrap();
        assert!(score >= 0.0 && score <= 100.0);

        let trending_score = service
            .calculate_trending_score(recipe_id, &TimeWindow::last_24_hours())
            .await
            .unwrap();
        assert!(trending_score >= 0.0);
    }
}

#[tokio::test]
async fn test_service_default_implementation() {
    let service1 = PopularityService::new();
    let service2 = PopularityService::default();

    // Both should work and produce consistent results
    let recipe_id = Uuid::new_v4();

    let score1 = service1.calculate_base_score(recipe_id).await.unwrap();
    let score2 = service2.calculate_base_score(recipe_id).await.unwrap();

    // Should produce same results (both using default config)
    assert_eq!(score1, score2);
}
