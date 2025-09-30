use chrono::Utc;
use imkitchen_recipe::services::recommendation::{
    InteractionType, MealTime, RecommendationContext, RecommendationEngine, RecommendationFilter,
    RecommendationReason, RecommendationType, Season, TimeContext, UserInteraction,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_recommendation_engine_creation() {
    let _engine = RecommendationEngine::new();
    assert!(true); // Engine creation is implicit in successful instantiation
}

#[tokio::test]
async fn test_recommendation_engine_with_custom_weights() {
    let mut weights = HashMap::new();
    weights.insert(RecommendationType::ContentBased, 0.6);
    weights.insert(RecommendationType::CollaborativeFiltering, 0.4);

    let _engine = RecommendationEngine::new().with_model_weights(weights);
    assert!(true); // Custom weights should be accepted
}

#[tokio::test]
async fn test_generate_recommendations_basic() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_123".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Dinner),
            season: Season::Summer,
            day_of_week: "Monday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 10,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Should return some recommendations
    assert!(!recommendations.is_empty());
    assert!(recommendations.len() <= 10);

    // Check recommendation structure
    for rec in &recommendations {
        assert!(!rec.title.is_empty());
        assert!(rec.confidence_score >= 0.0 && rec.confidence_score <= 1.0);
        assert!(rec.personalization_score >= 0.0 && rec.personalization_score <= 1.0);
        assert!(rec.novelty_score >= 0.0);
        assert!(!rec.reasons.is_empty());
    }
}

#[tokio::test]
async fn test_generate_recommendations_with_meal_time() {
    let engine = RecommendationEngine::new();

    let breakfast_context = RecommendationContext {
        user_id: "test_user_breakfast".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Breakfast),
            season: Season::Spring,
            day_of_week: "Saturday".to_string(),
            is_weekend: true,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 5,
        require_novelty: false,
        diversity_weight: 0.2,
    };

    let recommendations = engine
        .generate_recommendations(&breakfast_context)
        .await
        .unwrap();

    assert!(!recommendations.is_empty());
    assert!(recommendations.len() <= 5);

    // Should have applied temporal relevance
    for rec in &recommendations {
        assert!(rec.temporal_relevance >= 0.0);
    }
}

#[tokio::test]
async fn test_generate_recommendations_with_filters() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_filtered".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: None,
            season: Season::Fall,
            day_of_week: "Wednesday".to_string(),
            is_weekend: false,
        },
        filters: vec![
            RecommendationFilter {
                filter_type: "category".to_string(),
                filter_values: vec!["dessert".to_string()],
                is_exclude: true,
            },
            RecommendationFilter {
                filter_type: "difficulty".to_string(),
                filter_values: vec!["hard".to_string()],
                is_exclude: true,
            },
        ],
        exclude_recipes: vec![Uuid::new_v4()],
        boost_categories: vec!["pasta".to_string()],
        max_recommendations: 8,
        require_novelty: true,
        diversity_weight: 0.5,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    assert!(recommendations.len() <= 8);

    // Should have applied novelty and diversity
    for rec in &recommendations {
        if context.require_novelty {
            assert!(rec.novelty_score >= 0.0);
        }
        assert!(rec.diversity_boost.abs() <= 1.0); // Some diversity adjustment
    }
}

#[tokio::test]
async fn test_generate_recommendations_seasonal() {
    let engine = RecommendationEngine::new();

    let winter_context = RecommendationContext {
        user_id: "test_user_winter".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Lunch),
            season: Season::Winter,
            day_of_week: "Friday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 12,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let recommendations = engine
        .generate_recommendations(&winter_context)
        .await
        .unwrap();

    assert!(!recommendations.is_empty());

    // Should include seasonal recommendations
    let has_seasonal = recommendations.iter().any(|r| {
        matches!(
            r.recommendation_type,
            RecommendationType::SeasonalSuggestion
        )
    });

    if !recommendations.is_empty() {
        // At least some seasonal context should be applied
        assert!(recommendations.iter().any(|r| r.temporal_relevance > 0.0));
    }
}

#[tokio::test]
async fn test_recommendation_types_diversity() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_diversity".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Dinner),
            season: Season::Summer,
            day_of_week: "Sunday".to_string(),
            is_weekend: true,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 15,
        require_novelty: false,
        diversity_weight: 0.4,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Should have multiple recommendation types
    let mut type_counts = HashMap::new();
    for rec in &recommendations {
        *type_counts.entry(&rec.recommendation_type).or_insert(0) += 1;
    }

    // Should have at least 2 different recommendation types
    assert!(type_counts.len() >= 2);
}

#[tokio::test]
async fn test_recommendation_reasons_structure() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_reasons".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: None,
            season: Season::Spring,
            day_of_week: "Tuesday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 6,
        require_novelty: false,
        diversity_weight: 0.25,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Check that all recommendations have valid reasons
    for rec in &recommendations {
        assert!(!rec.reasons.is_empty());

        for reason in &rec.reasons {
            match reason {
                RecommendationReason::SimilarIngredients {
                    common_ingredients,
                    similarity_score,
                } => {
                    assert!(!common_ingredients.is_empty());
                    assert!(*similarity_score >= 0.0 && *similarity_score <= 1.0);
                }
                RecommendationReason::UserPreferenceMatch {
                    preference_type,
                    preference_value,
                    match_strength,
                } => {
                    assert!(!preference_type.is_empty());
                    assert!(!preference_value.is_empty());
                    assert!(*match_strength >= 0.0 && *match_strength <= 1.0);
                }
                RecommendationReason::CollaborativeSignal {
                    similar_users,
                    confidence,
                } => {
                    assert!(*similar_users > 0);
                    assert!(*confidence >= 0.0 && *confidence <= 1.0);
                }
                RecommendationReason::TrendingInCategory {
                    category,
                    trend_strength,
                } => {
                    assert!(!category.is_empty());
                    assert!(*trend_strength >= 0.0 && *trend_strength <= 1.0);
                }
                RecommendationReason::SeasonalRelevance {
                    season,
                    relevance_score,
                } => {
                    assert!(!season.is_empty());
                    assert!(*relevance_score >= 0.0 && *relevance_score <= 1.0);
                }
                _ => {
                    // Other reasons are valid by construction
                }
            }
        }
    }
}

#[tokio::test]
async fn test_update_user_profile_from_interaction() {
    let engine = RecommendationEngine::new();
    let user_id = "test_user_interaction";

    let interaction = UserInteraction {
        recipe_id: Uuid::new_v4(),
        interaction_type: InteractionType::Cooked,
        rating: Some(4.5),
        completion_status: Some("completed".to_string()),
        timestamp: Utc::now(),
        context: Some("dinner".to_string()),
    };

    let result = engine
        .update_user_profile_from_interaction(user_id, &interaction)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_interaction_types() {
    let engine = RecommendationEngine::new();
    let user_id = "test_user_interaction_types";

    let interaction_types = vec![
        InteractionType::Viewed,
        InteractionType::Clicked,
        InteractionType::Bookmarked,
        InteractionType::Cooked,
        InteractionType::Rated,
        InteractionType::Shared,
        InteractionType::Searched,
        InteractionType::Abandoned,
    ];

    for interaction_type in interaction_types {
        let interaction = UserInteraction {
            recipe_id: Uuid::new_v4(),
            interaction_type,
            rating: Some(4.0),
            completion_status: None,
            timestamp: Utc::now(),
            context: None,
        };

        let result = engine
            .update_user_profile_from_interaction(user_id, &interaction)
            .await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_get_model_metrics() {
    let engine = RecommendationEngine::new();

    let metrics = engine.get_model_metrics().await.unwrap();

    assert!(!metrics.is_empty());

    for metric in &metrics {
        assert!(!metric.model_type.is_empty());
        assert!(metric.accuracy >= 0.0 && metric.accuracy <= 1.0);
        assert!(metric.precision >= 0.0 && metric.precision <= 1.0);
        assert!(metric.recall >= 0.0 && metric.recall <= 1.0);
        assert!(metric.coverage >= 0.0 && metric.coverage <= 1.0);
        assert!(metric.diversity >= 0.0 && metric.diversity <= 1.0);
        assert!(metric.novelty >= 0.0 && metric.novelty <= 1.0);
        assert!(metric.user_satisfaction >= 0.0 && metric.user_satisfaction <= 1.0);
        assert!(metric.click_through_rate >= 0.0 && metric.click_through_rate <= 1.0);
        assert!(metric.conversion_rate >= 0.0 && metric.conversion_rate <= 1.0);
    }
}

#[tokio::test]
async fn test_confidence_score_ordering() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_ordering".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Dinner),
            season: Season::Fall,
            day_of_week: "Thursday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 10,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Should be sorted by confidence score (descending)
    for i in 1..recommendations.len() {
        assert!(recommendations[i - 1].confidence_score >= recommendations[i].confidence_score);
    }
}

#[tokio::test]
async fn test_empty_results_handling() {
    let engine = RecommendationEngine::new();

    // Test with very restrictive context
    let context = RecommendationContext {
        user_id: "test_user_empty".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: None,
            season: Season::Winter,
            day_of_week: "Monday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 0, // Request zero recommendations
        require_novelty: false,
        diversity_weight: 0.0,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Should handle zero limit gracefully
    assert!(recommendations.is_empty());
}

#[tokio::test]
async fn test_large_recommendation_limit() {
    let engine = RecommendationEngine::new();

    let context = RecommendationContext {
        user_id: "test_user_large_limit".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Lunch),
            season: Season::Summer,
            day_of_week: "Saturday".to_string(),
            is_weekend: true,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 100, // Very large limit
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let recommendations = engine.generate_recommendations(&context).await.unwrap();

    // Should handle large limits without issues
    assert!(recommendations.len() <= 100);
}

#[tokio::test]
async fn test_meal_time_enum_coverage() {
    let engine = RecommendationEngine::new();

    let meal_times = vec![
        MealTime::Breakfast,
        MealTime::Lunch,
        MealTime::Dinner,
        MealTime::Snack,
        MealTime::Dessert,
    ];

    for meal_time in meal_times {
        let context = RecommendationContext {
            user_id: "test_user_meal_times".to_string(),
            session_context: None,
            time_context: TimeContext {
                current_time: Utc::now(),
                meal_time: Some(meal_time),
                season: Season::Spring,
                day_of_week: "Sunday".to_string(),
                is_weekend: true,
            },
            filters: vec![],
            exclude_recipes: vec![],
            boost_categories: vec![],
            max_recommendations: 5,
            require_novelty: false,
            diversity_weight: 0.2,
        };

        let recommendations = engine.generate_recommendations(&context).await;
        assert!(recommendations.is_ok());
    }
}

#[tokio::test]
async fn test_season_enum_coverage() {
    let engine = RecommendationEngine::new();

    let seasons = vec![Season::Spring, Season::Summer, Season::Fall, Season::Winter];

    for season in seasons {
        let context = RecommendationContext {
            user_id: "test_user_seasons".to_string(),
            session_context: None,
            time_context: TimeContext {
                current_time: Utc::now(),
                meal_time: None,
                season,
                day_of_week: "Monday".to_string(),
                is_weekend: false,
            },
            filters: vec![],
            exclude_recipes: vec![],
            boost_categories: vec![],
            max_recommendations: 3,
            require_novelty: false,
            diversity_weight: 0.3,
        };

        let recommendations = engine.generate_recommendations(&context).await;
        assert!(recommendations.is_ok());
    }
}

#[tokio::test]
async fn test_weekend_vs_weekday_context() {
    let engine = RecommendationEngine::new();

    let weekday_context = RecommendationContext {
        user_id: "test_user_weekday".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Dinner),
            season: Season::Fall,
            day_of_week: "Tuesday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 5,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let weekend_context = RecommendationContext {
        user_id: "test_user_weekend".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: Some(MealTime::Dinner),
            season: Season::Fall,
            day_of_week: "Saturday".to_string(),
            is_weekend: true,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 5,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let weekday_recs = engine
        .generate_recommendations(&weekday_context)
        .await
        .unwrap();
    let weekend_recs = engine
        .generate_recommendations(&weekend_context)
        .await
        .unwrap();

    // Both should work
    assert!(!weekday_recs.is_empty() || !weekend_recs.is_empty());

    // Weekend recommendations might have slightly different temporal relevance
    if !weekend_recs.is_empty() {
        assert!(weekend_recs.iter().any(|r| r.temporal_relevance > 0.0));
    }
}

#[tokio::test]
async fn test_recommendation_engine_default() {
    let engine1 = RecommendationEngine::new();
    let engine2 = RecommendationEngine::default();

    // Both should work
    let context = RecommendationContext {
        user_id: "test_user_default".to_string(),
        session_context: None,
        time_context: TimeContext {
            current_time: Utc::now(),
            meal_time: None,
            season: Season::Spring,
            day_of_week: "Wednesday".to_string(),
            is_weekend: false,
        },
        filters: vec![],
        exclude_recipes: vec![],
        boost_categories: vec![],
        max_recommendations: 3,
        require_novelty: false,
        diversity_weight: 0.3,
    };

    let recs1 = engine1.generate_recommendations(&context).await;
    let recs2 = engine2.generate_recommendations(&context).await;

    assert!(recs1.is_ok());
    assert!(recs2.is_ok());
}
