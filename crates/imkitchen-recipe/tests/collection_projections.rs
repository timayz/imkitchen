use chrono::Utc;
use imkitchen_recipe::domain::collection::CollectionPrivacy;
use imkitchen_recipe::domain::RecipeCategory;
use imkitchen_recipe::projections::{
    CollectionDetailView, CollectionListView, CollectionReference, CollectionSearchIndex,
    RecipeInCollectionsView, RecipeSummary, UserFavoritesView,
};
use imkitchen_shared::Difficulty;
use uuid::Uuid;

#[test]
fn test_collection_list_view_properties() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    let view = CollectionListView {
        collection_id,
        user_id,
        name: "Quick Weeknight Dinners".to_string(),
        description: Some("Fast and easy recipes".to_string()),
        privacy: CollectionPrivacy::Private,
        recipe_count: 5,
        created_at,
        updated_at: created_at,
        is_archived: false,
    };

    assert!(!view.is_empty());
    assert!(view.is_private());
    assert!(!view.is_public());
    assert!(!view.is_shared());

    // Test empty collection
    let empty_view = CollectionListView {
        collection_id,
        user_id,
        name: "Empty Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Public,
        recipe_count: 0,
        created_at,
        updated_at: created_at,
        is_archived: false,
    };

    assert!(empty_view.is_empty());
    assert!(!empty_view.is_private());
    assert!(empty_view.is_public());
    assert!(!empty_view.is_shared());

    // Test shared collection
    let shared_view = CollectionListView {
        collection_id,
        user_id,
        name: "Shared Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Shared,
        recipe_count: 3,
        created_at,
        updated_at: created_at,
        is_archived: false,
    };

    assert!(!shared_view.is_private());
    assert!(!shared_view.is_public());
    assert!(shared_view.is_shared());
}

#[test]
fn test_collection_detail_view_analytics() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    // Create test recipes
    let recipe1 = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Quick Pasta".to_string(),
        difficulty: Difficulty::Easy,
        category: RecipeCategory::Main,
        total_time_minutes: 20,
        rating: 4.5,
        review_count: 10,
        ingredient_count: 5,
        tags: vec!["pasta".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    let recipe2 = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Complex Dish".to_string(),
        difficulty: Difficulty::Hard,
        category: RecipeCategory::Main,
        total_time_minutes: 120,
        rating: 4.8,
        review_count: 5,
        ingredient_count: 15,
        tags: vec!["advanced".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    let recipe3 = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Medium Recipe".to_string(),
        difficulty: Difficulty::Medium,
        category: RecipeCategory::Appetizer,
        total_time_minutes: 45,
        rating: 4.0,
        review_count: 8,
        ingredient_count: 8,
        tags: vec!["appetizer".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    let view = CollectionDetailView {
        collection_id,
        user_id,
        name: "Mixed Collection".to_string(),
        description: Some("Various difficulty recipes".to_string()),
        privacy: CollectionPrivacy::Private,
        recipes: vec![recipe1, recipe2, recipe3],
        recipe_count: 3,
        created_at,
        updated_at: created_at,
        is_archived: false,
    };

    assert!(!view.is_empty());

    // Test average difficulty calculation (1+3+2)/3 = 2 = Medium
    assert_eq!(view.average_difficulty(), Some(Difficulty::Medium));

    // Test average cook time (20+120+45)/3 = 61.67 ≈ 61
    assert_eq!(view.average_cook_time(), 61);

    // Test categories
    let categories = view.categories();
    assert_eq!(categories.get(&RecipeCategory::Main), Some(&2));
    assert_eq!(categories.get(&RecipeCategory::Appetizer), Some(&1));

    // Test empty collection
    let empty_view = CollectionDetailView {
        collection_id,
        user_id,
        name: "Empty Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: vec![],
        recipe_count: 0,
        created_at,
        updated_at: created_at,
        is_archived: false,
    };

    assert!(empty_view.is_empty());
    assert_eq!(empty_view.average_difficulty(), None);
    assert_eq!(empty_view.average_cook_time(), 0);
    assert!(empty_view.categories().is_empty());
}

#[test]
fn test_collection_search_index() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    let recipe_titles = vec![
        "Quick Pasta Sauce".to_string(),
        "Easy Weeknight Dinner".to_string(),
    ];

    let search_text = CollectionSearchIndex::build_search_text(
        "Weeknight Collection",
        &Some("Fast and easy recipes for busy weeknights".to_string()),
        &recipe_titles,
    );

    let view = CollectionSearchIndex {
        collection_id,
        user_id,
        name: "Weeknight Collection".to_string(),
        description: Some("Fast and easy recipes for busy weeknights".to_string()),
        privacy: CollectionPrivacy::Public,
        search_text,
        recipe_count: 2,
        recipe_titles,
        created_at,
        is_archived: false,
    };

    // Test search matching
    assert!(view.matches_search("weeknight"));
    assert!(view.matches_search("easy"));
    assert!(view.matches_search("pasta"));
    assert!(view.matches_search("busy"));
    assert!(!view.matches_search("complex"));
    assert!(!view.matches_search("advanced"));

    // Test case insensitive search
    assert!(view.matches_search("WEEKNIGHT"));
    assert!(view.matches_search("Easy"));
}

#[test]
fn test_user_favorites_view() {
    let user_id = Uuid::new_v4();
    let mut favorites = UserFavoritesView::new(user_id);

    assert_eq!(favorites.user_id, user_id);
    assert_eq!(favorites.total_count, 0);
    assert!(favorites.recipes.is_empty());

    // Create test recipes
    let quick_recipe = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Quick Snack".to_string(),
        difficulty: Difficulty::Easy,
        category: RecipeCategory::Appetizer,
        total_time_minutes: 15, // Quick recipe
        rating: 4.2,
        review_count: 20,
        ingredient_count: 3,
        tags: vec!["quick".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    let highly_rated_recipe = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Amazing Dish".to_string(),
        difficulty: Difficulty::Medium,
        category: RecipeCategory::Main,
        total_time_minutes: 60,
        rating: 4.8,      // Highly rated
        review_count: 50, // Many reviews
        ingredient_count: 8,
        tags: vec!["amazing".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    let regular_recipe = RecipeSummary {
        recipe_id: Uuid::new_v4(),
        title: "Regular Recipe".to_string(),
        difficulty: Difficulty::Medium,
        category: RecipeCategory::Main,
        total_time_minutes: 45,
        rating: 3.5, // Not highly rated
        review_count: 10,
        ingredient_count: 6,
        tags: vec!["regular".to_string()],
        created_by: user_id,
        is_public: true,
        image_url: None,
    };

    // Add favorites
    favorites.add_favorite(quick_recipe.clone());
    favorites.add_favorite(highly_rated_recipe.clone());
    favorites.add_favorite(regular_recipe.clone());

    assert_eq!(favorites.total_count, 3);
    assert!(favorites.is_favorited(quick_recipe.recipe_id));
    assert!(favorites.is_favorited(highly_rated_recipe.recipe_id));
    assert!(favorites.is_favorited(regular_recipe.recipe_id));

    // Test adding duplicate (should not increase count)
    favorites.add_favorite(quick_recipe.clone());
    assert_eq!(favorites.total_count, 3);

    // Test quick recipes filter
    let quick_recipes = favorites.quick_recipes();
    assert_eq!(quick_recipes.len(), 1);
    assert_eq!(quick_recipes[0].recipe_id, quick_recipe.recipe_id);

    // Test highly rated favorites filter (both quick_recipe and highly_rated_recipe qualify)
    let highly_rated = favorites.highly_rated_favorites();
    assert_eq!(highly_rated.len(), 2);
    let highly_rated_ids: Vec<Uuid> = highly_rated.iter().map(|r| r.recipe_id).collect();
    assert!(highly_rated_ids.contains(&quick_recipe.recipe_id));
    assert!(highly_rated_ids.contains(&highly_rated_recipe.recipe_id));

    // Test removing favorite
    favorites.remove_favorite(regular_recipe.recipe_id);
    assert_eq!(favorites.total_count, 2);
    assert!(!favorites.is_favorited(regular_recipe.recipe_id));

    // Test removing non-existent favorite
    favorites.remove_favorite(Uuid::new_v4());
    assert_eq!(favorites.total_count, 2);
}

#[test]
fn test_recipe_in_collections_view() {
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let collection_id_1 = Uuid::new_v4();
    let collection_id_2 = Uuid::new_v4();
    let collection_id_3 = Uuid::new_v4();
    let added_at = Utc::now();

    let mut view = RecipeInCollectionsView::new(recipe_id);

    assert_eq!(view.recipe_id, recipe_id);
    assert_eq!(view.total_collection_count, 0);
    assert_eq!(view.public_collection_count, 0);
    assert_eq!(view.private_collection_count, 0);

    // Add collections
    let public_collection = CollectionReference {
        collection_id: collection_id_1,
        name: "Public Collection".to_string(),
        privacy: CollectionPrivacy::Public,
        user_id,
        added_at,
    };

    let private_collection = CollectionReference {
        collection_id: collection_id_2,
        name: "Private Collection".to_string(),
        privacy: CollectionPrivacy::Private,
        user_id,
        added_at,
    };

    let shared_collection = CollectionReference {
        collection_id: collection_id_3,
        name: "Shared Collection".to_string(),
        privacy: CollectionPrivacy::Shared,
        user_id,
        added_at,
    };

    view.add_collection(public_collection.clone());
    view.add_collection(private_collection.clone());
    view.add_collection(shared_collection.clone());

    assert_eq!(view.total_collection_count, 3);
    assert_eq!(view.public_collection_count, 1);
    assert_eq!(view.private_collection_count, 1);

    // Test adding duplicate (should not increase count)
    view.add_collection(public_collection.clone());
    assert_eq!(view.total_collection_count, 3);

    // Test filtering
    let user_collections = view.collections_for_user(user_id);
    assert_eq!(user_collections.len(), 3);

    let public_collections = view.public_collections();
    assert_eq!(public_collections.len(), 1);
    assert_eq!(public_collections[0].collection_id, collection_id_1);

    // Test removing collection
    view.remove_collection(collection_id_2);
    assert_eq!(view.total_collection_count, 2);
    assert_eq!(view.private_collection_count, 0);

    // Test removing non-existent collection
    view.remove_collection(Uuid::new_v4());
    assert_eq!(view.total_collection_count, 2);
}
