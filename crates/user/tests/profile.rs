use imkitchen_shared::Metadata;
use imkitchen_user::{UpdateMealPreferencesInput, subscribe_command};

mod helpers;

#[tokio::test]
async fn test_update_meal_preferences() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let users = helpers::create_users(&state, vec!["john", "albert"]).await?;
    let john = users.first().unwrap();
    let albert = users.get(1).unwrap();

    command
        .update_meal_preferences(
            UpdateMealPreferencesInput {
                cuisine_variety_weight: 0.3,
                household_size: 2,
                dietary_restrictions: vec!["vegetarian".to_owned()],
            },
            Metadata::by(john.to_owned()),
        )
        .await?;

    let preferences = command.load_meal_preferences(john).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 0.3);
    assert_eq!(preferences.item.household_size, 2);
    assert_eq!(preferences.item.dietary_restrictions, vec!["vegetarian"]);

    command
        .update_meal_preferences(
            UpdateMealPreferencesInput {
                cuisine_variety_weight: 0.5,
                household_size: 4,
                dietary_restrictions: vec!["gluten-free".to_owned(), "vegan".to_owned()],
            },
            Metadata::by(john.to_owned()),
        )
        .await?;

    let preferences = command.load_meal_preferences(john).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 0.5);
    assert_eq!(preferences.item.household_size, 4);
    assert_eq!(
        preferences.item.dietary_restrictions,
        vec!["gluten-free", "vegan"]
    );

    command
        .update_meal_preferences(
            UpdateMealPreferencesInput {
                cuisine_variety_weight: 1.0,
                household_size: 1,
                dietary_restrictions: vec!["gluten-free".to_owned()],
            },
            Metadata::by_as(john.to_owned(), albert.to_owned()),
        )
        .await?;

    let preferences = command.load_meal_preferences(albert).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 1.0);
    assert_eq!(preferences.item.household_size, 1);
    assert_eq!(preferences.item.dietary_restrictions, vec!["gluten-free"]);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
