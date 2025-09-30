use imkitchen_recipe::events::discovery::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_recipe_viewed_event() {
    let event = RecipeViewedEvent {
        event_id: Uuid::new_v4(),
        recipe_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        viewed_at: Utc::now(),
        referrer: Some("search".to_string()),
    };

    // Test Evento-style serialization/deserialization
    let encoded = event.encode().unwrap();
    let decoded: RecipeViewedEvent = RecipeViewedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.recipe_id, decoded.recipe_id);
    assert_eq!(event.user_id, decoded.user_id);
    assert_eq!(event.session_id, decoded.session_id);
}

#[test]
fn test_recipe_searched_event() {
    let event = RecipeSearchedEvent {
        event_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        query_text: "pasta".to_string(),
        search_type: "FullText".to_string(),
        results_count: 25,
        searched_at: Utc::now(),
    };

    // Test Evento serialization
    let encoded = event.encode().unwrap();
    let decoded: RecipeSearchedEvent = RecipeSearchedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.query_text, decoded.query_text);
    assert_eq!(event.search_type, decoded.search_type);
    assert_eq!(event.results_count, decoded.results_count);
}

#[test]
fn test_filter_applied_event() {
    let event = FilterAppliedEvent {
        event_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        rating_threshold: Some(4.0),
        difficulty_levels: vec!["Easy".to_string(), "Medium".to_string()],
        max_prep_time: Some(30),
        dietary_restrictions: vec!["Vegetarian".to_string()],
        meal_types: vec!["Dinner".to_string()],
        results_count: 15,
        applied_at: Utc::now(),
    };

    let encoded = event.encode().unwrap();
    let decoded: FilterAppliedEvent = FilterAppliedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.rating_threshold, decoded.rating_threshold);
    assert_eq!(event.difficulty_levels, decoded.difficulty_levels);
    assert_eq!(event.results_count, decoded.results_count);
}

#[test]
fn test_random_recipe_requested_event() {
    let event = RandomRecipeRequestedEvent {
        event_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        filter_criteria: "vegetarian,easy".to_string(),
        selected_recipe_id: Some(Uuid::new_v4()),
        requested_at: Utc::now(),
    };

    let encoded = event.encode().unwrap();
    let decoded: RandomRecipeRequestedEvent = RandomRecipeRequestedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.filter_criteria, decoded.filter_criteria);
    assert_eq!(event.selected_recipe_id, decoded.selected_recipe_id);
}

#[test]
fn test_discovery_session_started_event() {
    let event = DiscoverySessionStartedEvent {
        event_id: Uuid::new_v4(),
        session_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        discovery_type: "browse".to_string(),
        started_at: Utc::now(),
    };

    let encoded = event.encode().unwrap();
    let decoded: DiscoverySessionStartedEvent = DiscoverySessionStartedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.session_id, decoded.session_id);
    assert_eq!(event.discovery_type, decoded.discovery_type);
}

#[test]
fn test_trending_calculated_event() {
    let event = TrendingCalculatedEvent {
        event_id: Uuid::new_v4(),
        recipe_id: Uuid::new_v4(),
        popularity_score: 85.5,
        trending_rank: 5,
        time_weighted_score: 92.3,
        calculated_at: Utc::now(),
    };

    let encoded = event.encode().unwrap();
    let decoded: TrendingCalculatedEvent = TrendingCalculatedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.recipe_id, decoded.recipe_id);
    assert_eq!(event.popularity_score, decoded.popularity_score);
    assert_eq!(event.trending_rank, decoded.trending_rank);
    assert_eq!(event.time_weighted_score, decoded.time_weighted_score);
}

#[test]
fn test_recipe_popularity_updated_event() {
    let event = RecipePopularityUpdatedEvent {
        event_id: Uuid::new_v4(),
        recipe_id: Uuid::new_v4(),
        view_count: 150,
        rating_average: 4.5,
        rating_count: 25,
        popularity_score: 78.2,
        updated_at: Utc::now(),
    };

    let encoded = event.encode().unwrap();
    let decoded: RecipePopularityUpdatedEvent = RecipePopularityUpdatedEvent::decode(&encoded).unwrap();
    
    assert_eq!(event.recipe_id, decoded.recipe_id);
    assert_eq!(event.view_count, decoded.view_count);
    assert_eq!(event.rating_average, decoded.rating_average);
    assert_eq!(event.popularity_score, decoded.popularity_score);
}

#[test]
fn test_aggregator_names() {
    assert_eq!(RecipeViewedEvent::aggregator_name(), "Discovery");
    assert_eq!(RecipeSearchedEvent::aggregator_name(), "Discovery");
    assert_eq!(FilterAppliedEvent::aggregator_name(), "Discovery");
    assert_eq!(RandomRecipeRequestedEvent::aggregator_name(), "Discovery");
    assert_eq!(DiscoverySessionStartedEvent::aggregator_name(), "DiscoverySession");
    assert_eq!(TrendingCalculatedEvent::aggregator_name(), "Trending");
    assert_eq!(RecipePopularityUpdatedEvent::aggregator_name(), "Trending");
}