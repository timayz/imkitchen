use imkitchen_recipe::commands::{
    AddRecipeToCollectionCommand, ArchiveCollectionCommand, BulkAddRecipesToCollectionCommand,
    BulkRemoveRecipesFromCollectionCommand, CreateCollectionCommand, CreateCollectionParams,
    DeleteCollectionCommand, RemoveRecipeFromCollectionCommand, RestoreCollectionCommand,
    UpdateCollectionCommand,
};
use imkitchen_recipe::domain::collection::CollectionPrivacy;
use uuid::Uuid;

#[test]
fn test_create_collection_command_validation() {
    let user_id = Uuid::new_v4();

    // Valid command
    let params = CreateCollectionParams {
        name: "Quick Weeknight Dinners".to_string(),
        description: Some("Fast and easy recipes for busy weeknights".to_string()),
        privacy: CollectionPrivacy::Private,
        created_by: user_id,
    };

    let command = CreateCollectionCommand::new(params).unwrap();
    assert_eq!(command.name, "Quick Weeknight Dinners");
    assert_eq!(
        command.description,
        Some("Fast and easy recipes for busy weeknights".to_string())
    );
    assert_eq!(command.privacy, CollectionPrivacy::Private);
    assert_eq!(command.created_by, user_id);
}

#[test]
fn test_create_collection_command_validation_failures() {
    let user_id = Uuid::new_v4();

    // Empty name should fail
    let params = CreateCollectionParams {
        name: "".to_string(),
        description: None,
        privacy: CollectionPrivacy::Private,
        created_by: user_id,
    };
    assert!(CreateCollectionCommand::new(params).is_err());

    // Name too long should fail
    let params = CreateCollectionParams {
        name: "a".repeat(101), // 101 chars, max is 100
        description: None,
        privacy: CollectionPrivacy::Private,
        created_by: user_id,
    };
    assert!(CreateCollectionCommand::new(params).is_err());

    // Description too long should fail
    let params = CreateCollectionParams {
        name: "Valid Name".to_string(),
        description: Some("a".repeat(501)), // 501 chars, max is 500
        privacy: CollectionPrivacy::Private,
        created_by: user_id,
    };
    assert!(CreateCollectionCommand::new(params).is_err());
}

#[test]
fn test_update_collection_command_builder() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = UpdateCollectionCommand::new(collection_id, user_id)
        .with_name("Updated Collection Name".to_string())
        .with_description(Some("Updated description".to_string()))
        .with_privacy(CollectionPrivacy::Public)
        .validate_and_build()
        .unwrap();

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.updated_by, user_id);
    assert_eq!(command.name, Some("Updated Collection Name".to_string()));
    assert_eq!(
        command.description,
        Some(Some("Updated description".to_string()))
    );
    assert_eq!(command.privacy, Some(CollectionPrivacy::Public));
}

#[test]
fn test_update_collection_command_validation_failures() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Empty name should fail
    let result = UpdateCollectionCommand::new(collection_id, user_id)
        .with_name("".to_string())
        .validate_and_build();
    assert!(result.is_err());

    // Name too long should fail
    let result = UpdateCollectionCommand::new(collection_id, user_id)
        .with_name("a".repeat(101))
        .validate_and_build();
    assert!(result.is_err());

    // Description too long should fail
    let result = UpdateCollectionCommand::new(collection_id, user_id)
        .with_description(Some("a".repeat(501)))
        .validate_and_build();
    assert!(result.is_err());
}

#[test]
fn test_delete_collection_command() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = DeleteCollectionCommand::new(collection_id, user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.deleted_by, user_id);
}

#[test]
fn test_add_recipe_to_collection_command() {
    let collection_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = AddRecipeToCollectionCommand::new(collection_id, recipe_id, user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.recipe_id, recipe_id);
    assert_eq!(command.added_by, user_id);
}

#[test]
fn test_remove_recipe_from_collection_command() {
    let collection_id = Uuid::new_v4();
    let recipe_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = RemoveRecipeFromCollectionCommand::new(collection_id, recipe_id, user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.recipe_id, recipe_id);
    assert_eq!(command.removed_by, user_id);
}

#[test]
fn test_bulk_add_recipes_to_collection_command() {
    let collection_id = Uuid::new_v4();
    let recipe_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id = Uuid::new_v4();

    let command =
        BulkAddRecipesToCollectionCommand::new(collection_id, recipe_ids.clone(), user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.recipe_ids, recipe_ids);
    assert_eq!(command.added_by, user_id);
}

#[test]
fn test_bulk_remove_recipes_from_collection_command() {
    let collection_id = Uuid::new_v4();
    let recipe_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let user_id = Uuid::new_v4();

    let command =
        BulkRemoveRecipesFromCollectionCommand::new(collection_id, recipe_ids.clone(), user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.recipe_ids, recipe_ids);
    assert_eq!(command.removed_by, user_id);
}

#[test]
fn test_archive_collection_command() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = ArchiveCollectionCommand::new(collection_id, user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.archived_by, user_id);
}

#[test]
fn test_restore_collection_command() {
    let collection_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let command = RestoreCollectionCommand::new(collection_id, user_id);

    assert_eq!(command.collection_id, collection_id);
    assert_eq!(command.restored_by, user_id);
}
