use imkitchen_recipe::domain::collection::CollectionPrivacy;
use imkitchen_recipe::queries::{
    CollectionByIdQuery, CollectionSearchQuery, CollectionsByUserQuery, RecipeInCollectionsQuery,
    RecipesByCollectionQuery, UserFavoritesQuery,
};
use uuid::Uuid;

#[test]
fn test_collection_by_id_query() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Query without user context
    let query = CollectionByIdQuery::new(collection_id);
    assert_eq!(query.collection_id, collection_id);
    assert_eq!(query.user_id, None);

    // Query with user context
    let query_with_user = CollectionByIdQuery::new(collection_id).with_user_context(user_id);
    assert_eq!(query_with_user.collection_id, collection_id);
    assert_eq!(query_with_user.user_id, Some(user_id));
}

#[test]
fn test_collections_by_user_query() {
    let user_id = Uuid::new_v4();

    // Default query
    let query = CollectionsByUserQuery::new(user_id);
    assert_eq!(query.user_id, user_id);
    assert!(!query.include_archived);
    assert_eq!(query.privacy_filter, None);
    assert_eq!(query.limit, Some(20));
    assert_eq!(query.offset, Some(0));

    // Query with options
    let query_with_options = CollectionsByUserQuery::new(user_id)
        .include_archived()
        .filter_by_privacy(CollectionPrivacy::Public)
        .with_pagination(10, 5);

    assert!(query_with_options.include_archived);
    assert_eq!(
        query_with_options.privacy_filter,
        Some(CollectionPrivacy::Public)
    );
    assert_eq!(query_with_options.limit, Some(10));
    assert_eq!(query_with_options.offset, Some(5));
}

#[test]
fn test_recipes_by_collection_query() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Default query
    let query = RecipesByCollectionQuery::new(collection_id);
    assert_eq!(query.collection_id, collection_id);
    assert_eq!(query.user_id, None);
    assert_eq!(query.limit, Some(20));
    assert_eq!(query.offset, Some(0));

    // Query with user context and pagination
    let query_with_options = RecipesByCollectionQuery::new(collection_id)
        .with_user_context(user_id)
        .with_pagination(50, 10);

    assert_eq!(query_with_options.user_id, Some(user_id));
    assert_eq!(query_with_options.limit, Some(50));
    assert_eq!(query_with_options.offset, Some(10));
}

#[test]
fn test_collection_search_query() {
    let user_id = Uuid::new_v4();

    // Default query
    let query = CollectionSearchQuery::new();
    assert_eq!(query.search_text, None);
    assert_eq!(query.privacy, None);
    assert_eq!(query.user_id, None);
    assert_eq!(query.limit, Some(20));
    assert_eq!(query.offset, Some(0));

    // Query with search options
    let query_with_options = CollectionSearchQuery::new()
        .with_search_text("weeknight dinners".to_string())
        .filter_by_privacy(CollectionPrivacy::Public)
        .with_user_context(user_id)
        .with_pagination(15, 30);

    assert_eq!(
        query_with_options.search_text,
        Some("weeknight dinners".to_string())
    );
    assert_eq!(query_with_options.privacy, Some(CollectionPrivacy::Public));
    assert_eq!(query_with_options.user_id, Some(user_id));
    assert_eq!(query_with_options.limit, Some(15));
    assert_eq!(query_with_options.offset, Some(30));
}

#[test]
fn test_collection_search_query_default() {
    let query = CollectionSearchQuery::default();
    assert_eq!(query.search_text, None);
    assert_eq!(query.privacy, None);
    assert_eq!(query.user_id, None);
    assert_eq!(query.limit, Some(20));
    assert_eq!(query.offset, Some(0));
}

#[test]
fn test_recipe_in_collections_query() {
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Query without user context
    let query = RecipeInCollectionsQuery::new(recipe_id);
    assert_eq!(query.recipe_id, recipe_id);
    assert_eq!(query.user_id, None);

    // Query with user context
    let query_with_user = RecipeInCollectionsQuery::new(recipe_id).with_user_context(user_id);
    assert_eq!(query_with_user.recipe_id, recipe_id);
    assert_eq!(query_with_user.user_id, Some(user_id));
}

#[test]
fn test_user_favorites_query() {
    let user_id = Uuid::new_v4();

    // Default query
    let query = UserFavoritesQuery::new(user_id);
    assert_eq!(query.user_id, user_id);
    assert_eq!(query.limit, Some(20));
    assert_eq!(query.offset, Some(0));

    // Query with pagination
    let query_with_pagination = UserFavoritesQuery::new(user_id).with_pagination(100, 25);
    assert_eq!(query_with_pagination.limit, Some(100));
    assert_eq!(query_with_pagination.offset, Some(25));
}
