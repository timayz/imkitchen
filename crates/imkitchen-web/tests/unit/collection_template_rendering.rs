use askama::Template;
use chrono::Utc;
use imkitchen_recipe::domain::collection::CollectionPrivacy;
use imkitchen_shared::Difficulty;
use imkitchen_recipe::domain::RecipeCategory;
use imkitchen_web::handlers::collections::{
    CollectionListTemplate, CollectionFormTemplate, CollectionDetailTemplate,
    FavoritesListTemplate, RecipeCollectionSelectorTemplate,
    CollectionListItem, CollectionFormData, CollectionFormErrors,
    CollectionDetailData, RecipeDetailItem, UserFavoritesData,
    RecipeSelectorItem
};
use uuid::Uuid;
use std::collections::HashMap;

#[test]
fn test_collection_form_template_rendering() {
    let collection = CollectionFormData {
        name: "Test Collection".to_string(),
        description: Some("A test collection".to_string()),
        privacy: Some("Private".to_string()),
    };

    let template = CollectionFormTemplate {
        form_title: "Create New Collection".to_string(),
        submit_button_text: "Create Collection".to_string(),
        collection,
        form_errors: CollectionFormErrors::default(),
    };

    let rendered = template.render().expect("Template should render successfully");
    
    // Verify form structure
    assert!(rendered.contains("Create New Collection"));
    assert!(rendered.contains("Create Collection"));
    assert!(rendered.contains("Test Collection"));
    assert!(rendered.contains("A test collection"));
    assert!(rendered.contains("ts-req=\"/collections\""));
    assert!(rendered.contains("ts-target=\"#collection-list\""));
}

#[test]
fn test_collection_form_template_with_validation_errors() {
    let collection = CollectionFormData {
        name: "".to_string(), // Empty name should show error
        description: None,
        privacy: Some("Private".to_string()),
    };

    let form_errors = CollectionFormErrors {
        name: Some("Collection name is required".to_string()),
        description: None,
    };

    let template = CollectionFormTemplate {
        form_title: "Create New Collection".to_string(),
        submit_button_text: "Create Collection".to_string(),
        collection,
        form_errors,
    };

    let rendered = template.render().expect("Template should render successfully");
    
    // Verify error display
    assert!(rendered.contains("Collection name is required"));
    assert!(rendered.contains("text-red-600")); // Error styling
}

#[test]
fn test_collection_list_template_rendering() {
    let collection_id = Uuid::new_v4();
    let created_at = Utc::now();

    let collections = vec![
        CollectionListItem {
            collection_id,
            name: "Quick Weeknight Dinners".to_string(),
            description: Some("Fast and easy recipes".to_string()),
            privacy: CollectionPrivacy::Private,
            recipe_count: 5,
            created_at,
            updated_at: created_at,
            is_archived: false,
        }
    ];

    let template = CollectionListTemplate { collections };
    let rendered = template.render().expect("Template should render successfully");

    // Verify collection display
    assert!(rendered.contains("Quick Weeknight Dinners"));
    assert!(rendered.contains("Fast and easy recipes"));
    assert!(rendered.contains("5 recipes"));
    assert!(rendered.contains("🔒 Private"));
    assert!(rendered.contains("bg-gray-100 text-gray-800")); // Private styling
}

#[test]
fn test_collection_list_template_empty_state() {
    let template = CollectionListTemplate { 
        collections: vec![] 
    };
    
    let rendered = template.render().expect("Template should render successfully");
    
    // Verify empty state
    assert!(rendered.contains("No collections yet"));
    assert!(rendered.contains("Create your first collection"));
    assert!(rendered.contains("Create Collection"));
}

#[test]
fn test_collection_detail_template_rendering() {
    let collection_id = Uuid::new_v4();
    let created_at = Utc::now();

    let recipe = RecipeDetailItem {
        recipe_id: Uuid::new_v4(),
        title: "Quick Pasta Salad".to_string(),
        difficulty: Difficulty::Easy,
        total_time_minutes: 20,
        rating: 4.5,
        review_count: 10,
        tags: vec!["pasta".to_string(), "quick".to_string()],
        image_url: None,
    };

    let collection = CollectionDetailData {
        collection_id,
        name: "Lunch Favorites".to_string(),
        description: Some("Quick lunch recipes".to_string()),
        privacy: CollectionPrivacy::Public,
        recipe_count: 1,
        recipes: vec![recipe],
        created_at,
        updated_at: created_at,
        categories: HashMap::new(),
    };

    let template = CollectionDetailTemplate {
        collection,
        is_owner: true,
        average_difficulty: Some(Difficulty::Easy),
        average_cook_time: 20,
    };

    let rendered = template.render().expect("Template should render successfully");

    // Verify collection header
    assert!(rendered.contains("Lunch Favorites"));
    assert!(rendered.contains("Quick lunch recipes"));
    assert!(rendered.contains("🌍 Public"));
    assert!(rendered.contains("1 recipes"));

    // Verify recipe display
    assert!(rendered.contains("Quick Pasta Salad"));
    assert!(rendered.contains("20m"));
    assert!(rendered.contains("4.5"));
    
    // Verify owner controls
    assert!(rendered.contains("Add Recipes"));
    assert!(rendered.contains("Edit Collection"));
}

#[test]
fn test_collection_detail_template_empty_collection() {
    let collection_id = Uuid::new_v4();
    let created_at = Utc::now();

    let collection = CollectionDetailData {
        collection_id,
        name: "Empty Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipe_count: 0,
        recipes: vec![],
        created_at,
        updated_at: created_at,
        categories: HashMap::new(),
    };

    let template = CollectionDetailTemplate {
        collection,
        is_owner: true,
        average_difficulty: None,
        average_cook_time: 0,
    };

    let rendered = template.render().expect("Template should render successfully");

    // Verify empty state
    assert!(rendered.contains("No recipes in this collection yet"));
    assert!(rendered.contains("Start building your collection"));
    assert!(rendered.contains("Add Recipes"));
}

#[test]
fn test_favorites_list_template_rendering() {
    let quick_recipe = RecipeDetailItem {
        recipe_id: Uuid::new_v4(),
        title: "5-Minute Sandwich".to_string(),
        difficulty: Difficulty::Easy,
        total_time_minutes: 5, // Quick recipe
        rating: 4.0,
        review_count: 15,
        tags: vec!["quick".to_string()],
        image_url: None,
    };

    let highly_rated_recipe = RecipeDetailItem {
        recipe_id: Uuid::new_v4(),
        title: "Perfect Pasta".to_string(),
        difficulty: Difficulty::Medium,
        total_time_minutes: 30,
        rating: 4.8, // Highly rated
        review_count: 50,
        tags: vec!["pasta".to_string()],
        image_url: None,
    };

    let favorites = UserFavoritesData {
        total_count: 2,
        recipes: vec![quick_recipe.clone(), highly_rated_recipe.clone()],
        quick_recipes: vec![quick_recipe.clone()],
        highly_rated_favorites: vec![quick_recipe.clone(), highly_rated_recipe.clone()],
        categories: HashMap::new(),
    };

    let template = FavoritesListTemplate { favorites };
    let rendered = template.render().expect("Template should render successfully");

    // Verify favorites header
    assert!(rendered.contains("⭐ My Favorites"));
    assert!(rendered.contains("2")); // Total count
    assert!(rendered.contains("Quick access to your favorite recipes"));

    // Verify tab structure
    assert!(rendered.contains("All Favorites (2)"));
    assert!(rendered.contains("Quick Recipes (1)"));
    assert!(rendered.contains("Highly Rated (2)"));

    // Verify recipe display
    assert!(rendered.contains("5-Minute Sandwich"));
    assert!(rendered.contains("Perfect Pasta"));
}

#[test]
fn test_favorites_list_template_empty_state() {
    let favorites = UserFavoritesData {
        total_count: 0,
        recipes: vec![],
        quick_recipes: vec![],
        highly_rated_favorites: vec![],
        categories: HashMap::new(),
    };

    let template = FavoritesListTemplate { favorites };
    let rendered = template.render().expect("Template should render successfully");

    // Verify empty state
    assert!(rendered.contains("No favorites yet"));
    assert!(rendered.contains("Start favoriting recipes"));
    assert!(rendered.contains("Discover Recipes"));
}

#[test]
fn test_recipe_collection_selector_template_rendering() {
    let collection_id = Uuid::new_v4();
    
    let recipes = vec![
        RecipeSelectorItem {
            recipe_id: Uuid::new_v4(),
            title: "Available Recipe".to_string(),
            difficulty: Difficulty::Easy,
            total_time_minutes: 15,
            rating: 4.2,
            image_url: None,
            in_collection: false,
        },
        RecipeSelectorItem {
            recipe_id: Uuid::new_v4(),
            title: "Already Added Recipe".to_string(),
            difficulty: Difficulty::Medium,
            total_time_minutes: 30,
            rating: 4.5,
            image_url: None,
            in_collection: true,
        }
    ];

    let template = RecipeCollectionSelectorTemplate {
        collection_id,
        available_recipes: recipes,
        collection_recipe_count: 1,
    };

    let rendered = template.render().expect("Template should render successfully");

    // Verify modal structure
    assert!(rendered.contains("Add Recipes to Collection"));
    assert!(rendered.contains("Search your recipes"));
    assert!(rendered.contains("1 recipes in collection"));

    // Verify recipe display and buttons
    assert!(rendered.contains("Available Recipe"));
    assert!(rendered.contains("Add to Collection"));
    assert!(rendered.contains("Already Added Recipe"));
    assert!(rendered.contains("Remove"));

    // Verify TwinSpark attributes
    assert!(rendered.contains("ts-req=\"/collections"));
    assert!(rendered.contains("ts-method=\"POST\""));
    assert!(rendered.contains("ts-method=\"DELETE\""));
}

#[test]
fn test_collection_privacy_styling() {
    let created_at = Utc::now();
    
    let collections = vec![
        CollectionListItem {
            collection_id: Uuid::new_v4(),
            name: "Private Collection".to_string(),
            description: None,
            privacy: CollectionPrivacy::Private,
            recipe_count: 1,
            created_at,
            updated_at: created_at,
            is_archived: false,
        },
        CollectionListItem {
            collection_id: Uuid::new_v4(),
            name: "Shared Collection".to_string(),
            description: None,
            privacy: CollectionPrivacy::Shared,
            recipe_count: 2,
            created_at,
            updated_at: created_at,
            is_archived: false,
        },
        CollectionListItem {
            collection_id: Uuid::new_v4(),
            name: "Public Collection".to_string(),
            description: None,
            privacy: CollectionPrivacy::Public,
            recipe_count: 3,
            created_at,
            updated_at: created_at,
            is_archived: false,
        }
    ];

    let template = CollectionListTemplate { collections };
    let rendered = template.render().expect("Template should render successfully");

    // Verify privacy-specific styling
    assert!(rendered.contains("🔒 Private"));
    assert!(rendered.contains("bg-gray-100 text-gray-800"));
    
    assert!(rendered.contains("👥 Shared"));
    assert!(rendered.contains("bg-blue-100 text-blue-800"));
    
    assert!(rendered.contains("🌍 Public"));
    assert!(rendered.contains("bg-green-100 text-green-800"));
}

#[test]
fn test_archived_collection_display() {
    let created_at = Utc::now();
    
    let collections = vec![
        CollectionListItem {
            collection_id: Uuid::new_v4(),
            name: "Archived Collection".to_string(),
            description: Some("This collection is archived".to_string()),
            privacy: CollectionPrivacy::Private,
            recipe_count: 5,
            created_at,
            updated_at: created_at,
            is_archived: true,
        }
    ];

    let template = CollectionListTemplate { collections };
    let rendered = template.render().expect("Template should render successfully");

    // Verify archived indicator
    assert!(rendered.contains("📦 Archived"));
    assert!(rendered.contains("bg-yellow-100 text-yellow-800"));
}

#[test]
fn test_twinspark_attributes_in_templates() {
    let template = CollectionFormTemplate {
        form_title: "Test Form".to_string(),
        submit_button_text: "Submit".to_string(),
        collection: CollectionFormData::default(),
        form_errors: CollectionFormErrors::default(),
    };

    let rendered = template.render().expect("Template should render successfully");

    // Verify TwinSpark attributes for JavaScript-free interactions
    assert!(rendered.contains("ts-req=\"/collections\""));
    assert!(rendered.contains("ts-target=\"#collection-list\""));
    assert!(rendered.contains("ts-indicator=\"#form-loading\""));
}

#[test]
fn test_collection_search_functionality() {
    // This test verifies the search input has proper TwinSpark configuration
    let template = CollectionListTemplate { 
        collections: vec![] 
    };
    
    let rendered = template.render().expect("Template should render successfully");
    
    // Verify search functionality
    assert!(rendered.contains("ts-req=\"/collections/search\""));
    assert!(rendered.contains("ts-target=\"#collection-results\""));
    assert!(rendered.contains("ts-trigger=\"keyup changed delay:500ms\""));
    
    // Verify filter functionality
    assert!(rendered.contains("ts-req=\"/collections/filter\""));
    assert!(rendered.contains("ts-req=\"/collections/sort\""));
}