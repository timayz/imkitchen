use chrono::Utc;
use imkitchen_recipe::{
    commands::{CreateRecipeCommand, DeleteRecipeCommand, UpdateRecipeCommand},
    domain::{Difficulty, Ingredient, Instruction, RecipeCategory},
    events::{RecipeCreated, RecipeDeleted, RecipeUpdated},
};
use uuid::Uuid;
use validator::Validate;

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn test_create_recipe_command_validation() {
        let user_id = Uuid::new_v4();

        let ingredient =
            Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None).unwrap();

        let instruction = Instruction::new(1, "Mix ingredients".to_string(), Some(5)).unwrap();

        // Valid command
        let valid_command = CreateRecipeCommand::new(
            "Test Recipe".to_string(),
            vec![ingredient.clone()],
            vec![instruction.clone()],
            15,
            30,
            Difficulty::Easy,
            RecipeCategory::Main,
            user_id,
            true,
            vec!["test".to_string()],
        );

        assert!(valid_command.is_ok());

        // Invalid command - empty title
        let invalid_command = CreateRecipeCommand::new(
            "".to_string(),
            vec![ingredient],
            vec![instruction],
            15,
            30,
            Difficulty::Easy,
            RecipeCategory::Main,
            user_id,
            true,
            vec![],
        );

        assert!(invalid_command.is_err());
    }

    #[test]
    fn test_update_recipe_command_builder() {
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let command = UpdateRecipeCommand::new(recipe_id, user_id)
            .with_title("Updated Recipe".to_string())
            .with_difficulty(Difficulty::Hard)
            .validate_and_build();

        assert!(command.is_ok());
        let command = command.unwrap();
        assert_eq!(command.recipe_id, recipe_id);
        assert_eq!(command.title, Some("Updated Recipe".to_string()));
        assert_eq!(command.difficulty, Some(Difficulty::Hard));
        assert_eq!(command.updated_by, user_id);
    }

    #[test]
    fn test_delete_recipe_command() {
        let recipe_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let command = DeleteRecipeCommand::new(recipe_id, user_id);

        assert_eq!(command.recipe_id, recipe_id);
        assert_eq!(command.deleted_by, user_id);
    }
}

#[cfg(test)]
mod event_tests {
    use super::*;
    use imkitchen_shared::DomainEvent;

    #[test]
    fn test_recipe_created_event() {
        let recipe_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ingredient =
            Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None).unwrap();

        let instruction = Instruction::new(1, "Mix ingredients".to_string(), Some(5)).unwrap();

        let event = RecipeCreated {
            event_id,
            recipe_id,
            title: "Test Recipe".to_string(),
            ingredients: vec![ingredient],
            instructions: vec![instruction],
            prep_time_minutes: 15,
            cook_time_minutes: 30,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: true,
            tags: vec!["test".to_string()],
            occurred_at: now,
        };

        assert_eq!(event.event_id(), event_id);
        assert_eq!(event.aggregate_id(), recipe_id);
        assert_eq!(event.event_type(), "RecipeCreated");
        assert_eq!(event.occurred_at(), now);
    }

    #[test]
    fn test_recipe_updated_event() {
        let recipe_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let now = Utc::now();

        let event = RecipeUpdated {
            event_id,
            recipe_id,
            title: Some("Updated Recipe".to_string()),
            ingredients: None,
            instructions: None,
            prep_time_minutes: Some(20),
            cook_time_minutes: None,
            difficulty: Some(Difficulty::Medium),
            category: None,
            is_public: None,
            tags: None,
            occurred_at: now,
        };

        assert_eq!(event.event_id(), event_id);
        assert_eq!(event.aggregate_id(), recipe_id);
        assert_eq!(event.event_type(), "RecipeUpdated");
        assert_eq!(event.occurred_at(), now);
    }

    #[test]
    fn test_recipe_deleted_event() {
        let recipe_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let now = Utc::now();

        let event = RecipeDeleted {
            event_id,
            recipe_id,
            occurred_at: now,
        };

        assert_eq!(event.event_id(), event_id);
        assert_eq!(event.aggregate_id(), recipe_id);
        assert_eq!(event.event_type(), "RecipeDeleted");
        assert_eq!(event.occurred_at(), now);
    }

    #[test]
    fn test_event_serialization() {
        let recipe_id = Uuid::new_v4();
        let event_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let now = Utc::now();

        let ingredient =
            Ingredient::new("Flour".to_string(), 2.0, "cups".to_string(), None).unwrap();

        let instruction = Instruction::new(1, "Mix ingredients".to_string(), Some(5)).unwrap();

        let event = RecipeCreated {
            event_id,
            recipe_id,
            title: "Test Recipe".to_string(),
            ingredients: vec![ingredient],
            instructions: vec![instruction],
            prep_time_minutes: 15,
            cook_time_minutes: 30,
            difficulty: Difficulty::Easy,
            category: RecipeCategory::Main,
            created_by: user_id,
            is_public: true,
            tags: vec!["test".to_string()],
            occurred_at: now,
        };

        // Test serialization
        let serialized = serde_json::to_string(&event).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: RecipeCreated = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.event_id, event.event_id);
        assert_eq!(deserialized.recipe_id, event.recipe_id);
        assert_eq!(deserialized.title, event.title);
        assert_eq!(deserialized.difficulty, event.difficulty);
    }
}
