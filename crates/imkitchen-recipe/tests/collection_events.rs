use chrono::Utc;
use imkitchen_recipe::domain::collection::CollectionPrivacy;
use imkitchen_recipe::events::{
    CollectionArchived, CollectionCreated, CollectionDeleted, CollectionRestored,
    CollectionUpdated, RecipeAddedToCollection, RecipeRemovedFromCollection,
};
use imkitchen_shared::DomainEvent;
use uuid::Uuid;

#[test]
fn test_collection_created_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let name = "Quick Weeknight Dinners".to_string();
    let description = Some("Fast and easy recipes for busy weeknights".to_string());
    let privacy = CollectionPrivacy::Private;
    let occurred_at = Utc::now();

    let event = CollectionCreated {
        event_id,
        collection_id,
        user_id,
        name: name.clone(),
        description: description.clone(),
        privacy,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "CollectionCreated");
    assert_eq!(event.user_id, user_id);
    assert_eq!(event.name, name);
    assert_eq!(event.description, description);
    assert_eq!(event.privacy, privacy);
}

#[test]
fn test_collection_updated_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let occurred_at = Utc::now();

    let event = CollectionUpdated {
        event_id,
        collection_id,
        name: Some("Updated Collection Name".to_string()),
        description: Some(Some("Updated description".to_string())),
        privacy: Some(CollectionPrivacy::Public),
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "CollectionUpdated");
}

#[test]
fn test_collection_deleted_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let occurred_at = Utc::now();

    let event = CollectionDeleted {
        event_id,
        collection_id,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "CollectionDeleted");
}

#[test]
fn test_recipe_added_to_collection_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let sort_order = 1;
    let occurred_at = Utc::now();

    let event = RecipeAddedToCollection {
        event_id,
        collection_id,
        recipe_id,
        sort_order,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "RecipeAddedToCollection");
    assert_eq!(event.recipe_id, recipe_id);
    assert_eq!(event.sort_order, sort_order);
}

#[test]
fn test_recipe_removed_from_collection_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let occurred_at = Utc::now();

    let event = RecipeRemovedFromCollection {
        event_id,
        collection_id,
        recipe_id,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "RecipeRemovedFromCollection");
    assert_eq!(event.recipe_id, recipe_id);
}

#[test]
fn test_collection_archived_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let occurred_at = Utc::now();

    let event = CollectionArchived {
        event_id,
        collection_id,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "CollectionArchived");
}

#[test]
fn test_collection_restored_event() {
    let event_id = Uuid::new_v4();
    let collection_id = Uuid::new_v4();
    let occurred_at = Utc::now();

    let event = CollectionRestored {
        event_id,
        collection_id,
        occurred_at,
    };

    assert_eq!(event.event_id(), event_id);
    assert_eq!(event.aggregate_id(), collection_id);
    assert_eq!(event.occurred_at(), occurred_at);
    assert_eq!(event.event_type(), "CollectionRestored");
}
