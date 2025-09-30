use chrono::Utc;
use imkitchen_recipe::command_handlers::discovery::*;
use imkitchen_recipe::commands::discovery::*;
use imkitchen_recipe::domain::discovery::*;
use imkitchen_shared::types::{DietaryRestriction, Difficulty, MealType};
use uuid::Uuid;
use validator::Validate;

#[tokio::test]
async fn test_search_recipes_command_handler() {
    let handler = SearchRecipesCommandHandler::new();

    let command = SearchRecipesCommand {
        command_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        search_criteria: SearchCriteria {
            query_text: "pasta".to_string(),
            search_type: SearchType::FullText,
            include_suggestions: true,
        },
        filters: DiscoveryFilters::default(),
        sort_order: SortingCriteria::HighestRated,
        page: 1,
        page_size: 20,
    };

    // Test validation passes for valid command
    assert!(command.validate().is_ok());

    // Test command processing
    let result = handler.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_recipes_command_validation() {
    let handler = SearchRecipesCommandHandler::new();

    let invalid_command = SearchRecipesCommand {
        command_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        search_criteria: SearchCriteria {
            query_text: "".to_string(), // Invalid - empty query
            search_type: SearchType::FullText,
            include_suggestions: true,
        },
        filters: DiscoveryFilters::default(),
        sort_order: SortingCriteria::HighestRated,
        page: 0,        // Invalid - should be >= 1
        page_size: 200, // Invalid - should be <= 100
    };

    // Test validation fails for invalid command
    assert!(invalid_command.validate().is_err());
}

#[tokio::test]
async fn test_apply_filters_command_handler() {
    let handler = ApplyFiltersCommandHandler::new();

    let command = ApplyFiltersCommand {
        command_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        filters: DiscoveryFilters {
            rating_threshold: Some(4.0),
            difficulty_levels: vec![Difficulty::Easy, Difficulty::Medium],
            max_prep_time: Some(30),
            dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            meal_types: vec![MealType::Dinner],
        },
        current_search_criteria: SearchCriteria {
            query_text: "chicken".to_string(),
            search_type: SearchType::Ingredient,
            include_suggestions: true,
        },
    };

    assert!(command.validate().is_ok());

    let result = handler.handle(command).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_request_random_recipe_command_handler() {
    let handler = RequestRandomRecipeCommandHandler::new();

    let command = RequestRandomRecipeCommand {
        command_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        session_id: Uuid::new_v4(),
        preference_filters: DiscoveryFilters {
            rating_threshold: Some(3.0),
            difficulty_levels: vec![Difficulty::Easy],
            max_prep_time: Some(45),
            dietary_restrictions: vec![],
            meal_types: vec![MealType::Lunch, MealType::Dinner],
        },
    };

    assert!(command.validate().is_ok());

    let result = handler.handle(command).await;
    assert!(result.is_ok());
}

#[test]
fn test_discovery_session_aggregate() {
    let session_id = Uuid::new_v4();
    let user_id = Some(Uuid::new_v4());

    let session = DiscoverySession::start(session_id, user_id, "browse");

    assert_eq!(session.session_id, session_id);
    assert_eq!(session.user_id, user_id);
    assert_eq!(session.discovery_type, "browse");
    assert_eq!(session.search_count, 0);
    assert_eq!(session.filter_applications, 0);
    assert_eq!(session.recipes_viewed, 0);
}

#[test]
fn test_discovery_session_interactions() {
    let session_id = Uuid::new_v4();
    let user_id = Some(Uuid::new_v4());

    let mut session = DiscoverySession::start(session_id, user_id, "search");

    // Track search
    session.record_search("pasta", "FullText", 25);
    assert_eq!(session.search_count, 1);

    // Track filter application
    session.record_filter_application(15);
    assert_eq!(session.filter_applications, 1);

    // Track recipe views
    session.record_recipe_view(Uuid::new_v4());
    assert_eq!(session.recipes_viewed, 1);
    assert_eq!(session.viewed_recipe_ids.len(), 1);
}

#[test]
fn test_discovery_session_preferences() {
    let session_id = Uuid::new_v4();
    let user_id = Some(Uuid::new_v4());

    let mut session = DiscoverySession::start(session_id, user_id, "browse");

    // Learn preferences from interactions
    session.learn_preference("difficulty", "Easy");
    session.learn_preference("meal_type", "Dinner");
    session.learn_preference("dietary", "Vegetarian");

    assert_eq!(session.learned_preferences.len(), 3);
    assert!(session.learned_preferences.contains_key("difficulty"));
    assert!(session.learned_preferences.contains_key("meal_type"));
    assert!(session.learned_preferences.contains_key("dietary"));
}
