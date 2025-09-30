use imkitchen_recipe::domain::discovery::*;
use imkitchen_shared::types::{Difficulty, DietaryRestriction, MealType};
use chrono::Utc;
use uuid::Uuid;
use validator::Validate;

#[test]
fn test_discovery_filters_creation() {
    let filters = DiscoveryFilters {
        rating_threshold: Some(4.0),
        difficulty_levels: vec![Difficulty::Easy, Difficulty::Medium],
        max_prep_time: Some(30),
        dietary_restrictions: vec![DietaryRestriction::Vegetarian],
        meal_types: vec![MealType::Dinner],
    };

    assert_eq!(filters.rating_threshold, Some(4.0));
    assert_eq!(filters.difficulty_levels.len(), 2);
    assert_eq!(filters.max_prep_time, Some(30));
}

#[test]
fn test_discovery_filters_validation() {
    let invalid_filters = DiscoveryFilters {
        rating_threshold: Some(6.0), // Invalid - should be 1-5
        difficulty_levels: vec![],
        max_prep_time: Some(0), // Invalid - should be positive
        dietary_restrictions: vec![],
        meal_types: vec![],
    };

    assert!(invalid_filters.validate().is_err());
}

#[test]
fn test_search_criteria_creation() {
    let criteria = SearchCriteria {
        query_text: "pasta".to_string(),
        search_type: SearchType::FullText,
        include_suggestions: true,
    };

    assert_eq!(criteria.query_text, "pasta");
    assert_eq!(criteria.search_type, SearchType::FullText);
    assert!(criteria.include_suggestions);
}

#[test]
fn test_search_criteria_validation() {
    let invalid_criteria = SearchCriteria {
        query_text: "".to_string(), // Invalid - empty query
        search_type: SearchType::FullText,
        include_suggestions: false,
    };

    assert!(invalid_criteria.validate().is_err());

    let too_long_criteria = SearchCriteria {
        query_text: "a".repeat(501), // Invalid - too long
        search_type: SearchType::FullText,
        include_suggestions: false,
    };

    assert!(too_long_criteria.validate().is_err());
}

#[test]
fn test_sorting_criteria_display() {
    assert_eq!(SortingCriteria::Newest.to_string(), "Newest");
    assert_eq!(SortingCriteria::Popular.to_string(), "Most Popular");
    assert_eq!(SortingCriteria::HighestRated.to_string(), "Highest Rated");
    assert_eq!(SortingCriteria::QuickestPrep.to_string(), "Quickest Prep");
}

#[test]
fn test_recipe_discovery_creation() {
    let discovery_id = Uuid::new_v4();
    let user_id = Some(Uuid::new_v4());
    
    let search_criteria = SearchCriteria {
        query_text: "chicken".to_string(),
        search_type: SearchType::Ingredient,
        include_suggestions: true,
    };

    let filters = DiscoveryFilters {
        rating_threshold: Some(3.5),
        difficulty_levels: vec![Difficulty::Easy],
        max_prep_time: Some(45),
        dietary_restrictions: vec![],
        meal_types: vec![MealType::Lunch, MealType::Dinner],
    };

    let discovery = RecipeDiscovery {
        discovery_id,
        user_id,
        search_criteria,
        filters,
        sort_order: SortingCriteria::HighestRated,
        page: 1,
        page_size: 20,
        created_at: Utc::now(),
    };

    assert_eq!(discovery.discovery_id, discovery_id);
    assert_eq!(discovery.user_id, user_id);
    assert_eq!(discovery.sort_order, SortingCriteria::HighestRated);
    assert_eq!(discovery.page, 1);
    assert_eq!(discovery.page_size, 20);
}

#[test]
fn test_trending_score_creation() {
    let recipe_id = Uuid::new_v4();
    let now = Utc::now();
    
    let trending_score = TrendingScore {
        recipe_id,
        popularity_score: 85.5,
        trending_rank: 5,
        calculated_at: now,
        time_weighted_score: 92.3,
    };

    assert_eq!(trending_score.recipe_id, recipe_id);
    assert_eq!(trending_score.popularity_score, 85.5);
    assert_eq!(trending_score.trending_rank, 5);
    assert_eq!(trending_score.time_weighted_score, 92.3);
}

#[test]
fn test_trending_score_validation() {
    let invalid_score = TrendingScore {
        recipe_id: Uuid::new_v4(),
        popularity_score: -10.0, // Invalid - negative score
        trending_rank: 0, // Invalid - should be positive
        calculated_at: Utc::now(),
        time_weighted_score: -5.0, // Invalid - negative score
    };

    assert!(invalid_score.validate().is_err());
}