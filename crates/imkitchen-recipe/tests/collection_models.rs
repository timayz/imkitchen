use chrono::Utc;
use imkitchen_recipe::domain::collection::{
    CollectionPrivacy, RecipeCollection, RecipeCollectionMembership,
};
use imkitchen_recipe::domain::services::{
    CollectionSearchService, CollectionValidationService, RecipeCollectionMapper,
};
use uuid::Uuid;
use validator::Validate;

#[test]
fn test_recipe_collection_creation_with_valid_data() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let name = "Quick Weeknight Dinners".to_string();
    let description = Some("Fast and easy recipes for busy weeknights".to_string());
    let privacy = CollectionPrivacy::Private;
    let created_at = Utc::now();

    let collection = RecipeCollection {
        collection_id,
        user_id,
        name: name.clone(),
        description: description.clone(),
        privacy,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };

    assert_eq!(collection.collection_id, collection_id);
    assert_eq!(collection.user_id, user_id);
    assert_eq!(collection.name, name);
    assert_eq!(collection.description, description);
    assert_eq!(collection.privacy, privacy);
    assert!(collection.recipes.is_empty());
    assert!(collection.validate().is_ok());
}

#[test]
fn test_collection_name_validation_constraints() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    // Test empty name (should fail)
    let collection_empty_name = RecipeCollection {
        collection_id,
        user_id,
        name: "".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_empty_name.validate().is_err());

    // Test name too long (should fail)
    let long_name = "a".repeat(101); // 101 chars, max is 100
    let collection_long_name = RecipeCollection {
        collection_id,
        user_id,
        name: long_name,
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_long_name.validate().is_err());

    // Test valid name length (should pass)
    let collection_valid_name = RecipeCollection {
        collection_id,
        user_id,
        name: "Valid Collection Name".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_valid_name.validate().is_ok());
}

#[test]
fn test_collection_description_validation_constraints() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    // Test description too long (should fail)
    let long_description = Some("a".repeat(501)); // 501 chars, max is 500
    let collection_long_desc = RecipeCollection {
        collection_id,
        user_id,
        name: "Test Collection".to_string(),
        description: long_description,
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_long_desc.validate().is_err());

    // Test valid description length (should pass)
    let collection_valid_desc = RecipeCollection {
        collection_id,
        user_id,
        name: "Test Collection".to_string(),
        description: Some("Valid description".to_string()),
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_valid_desc.validate().is_ok());

    // Test None description (should pass)
    let collection_no_desc = RecipeCollection {
        collection_id,
        user_id,
        name: "Test Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };
    assert!(collection_no_desc.validate().is_ok());
}

#[test]
fn test_collection_privacy_enum_values() {
    let privacy_private = CollectionPrivacy::Private;
    let privacy_shared = CollectionPrivacy::Shared;
    let privacy_public = CollectionPrivacy::Public;

    assert_eq!(privacy_private, CollectionPrivacy::Private);
    assert_eq!(privacy_shared, CollectionPrivacy::Shared);
    assert_eq!(privacy_public, CollectionPrivacy::Public);
}

#[test]
fn test_recipe_collection_membership() {
    let recipe_id = Uuid::new_v4();
    let added_at = Utc::now();
    let sort_order = 1;

    let membership = RecipeCollectionMembership {
        recipe_id,
        added_at,
        sort_order,
    };

    assert_eq!(membership.recipe_id, recipe_id);
    assert_eq!(membership.added_at, added_at);
    assert_eq!(membership.sort_order, sort_order);
}

#[test]
fn test_collection_recipe_management() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let created_at = Utc::now();

    let membership = RecipeCollectionMembership {
        recipe_id,
        added_at: created_at,
        sort_order: 1,
    };

    let mut collection = RecipeCollection {
        collection_id,
        user_id,
        name: "Test Collection".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        recipes: vec![membership],
        created_at,
        updated_at: created_at,
    };

    assert_eq!(collection.recipes.len(), 1);
    assert_eq!(collection.recipes[0].recipe_id, recipe_id);
    assert_eq!(collection.recipes[0].sort_order, 1);

    // Test adding another recipe
    let recipe_id_2 = Uuid::new_v4();
    let membership_2 = RecipeCollectionMembership {
        recipe_id: recipe_id_2,
        added_at: created_at,
        sort_order: 2,
    };
    collection.recipes.push(membership_2);

    assert_eq!(collection.recipes.len(), 2);
    assert_eq!(collection.recipes[1].recipe_id, recipe_id_2);
    assert_eq!(collection.recipes[1].sort_order, 2);
}

#[test]
fn test_collection_validation_service() {
    let service = CollectionValidationService::new();

    // Test max collections per user validation (mock implementation)
    let user_id = Uuid::new_v4();
    let current_collection_count = 49; // Just under limit

    assert!(service.validate_user_collection_limit(user_id, current_collection_count));

    let current_collection_count = 50; // At limit
    assert!(!service.validate_user_collection_limit(user_id, current_collection_count));
}

#[test]
fn test_recipe_collection_mapper() {
    let mapper = RecipeCollectionMapper::new();

    // Test mapping collection to different views
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    let collection = RecipeCollection {
        collection_id,
        user_id,
        name: "Test Collection".to_string(),
        description: Some("Test description".to_string()),
        privacy: CollectionPrivacy::Public,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };

    // Test mapping to list item
    let list_item = mapper.to_list_item(&collection);
    assert_eq!(list_item.collection_id, collection_id);
    assert_eq!(list_item.name, "Test Collection");
    assert_eq!(list_item.privacy, CollectionPrivacy::Public);
    assert_eq!(list_item.recipe_count, 0);
}

#[test]
fn test_collection_search_service() {
    let service = CollectionSearchService::new();

    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();

    let collection = RecipeCollection {
        collection_id,
        user_id,
        name: "Quick Weeknight Dinners".to_string(),
        description: Some("Fast and easy recipes for busy weeknights".to_string()),
        privacy: CollectionPrivacy::Public,
        recipes: Vec::new(),
        created_at,
        updated_at: created_at,
    };

    // Test search functionality
    assert!(service.matches_search_query(&collection, "quick"));
    assert!(service.matches_search_query(&collection, "weeknight"));
    assert!(service.matches_search_query(&collection, "easy"));
    assert!(!service.matches_search_query(&collection, "complex"));
}
