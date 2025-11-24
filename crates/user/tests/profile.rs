use imkitchen_recipe::DietaryRestriction;
use imkitchen_shared::Metadata;
use imkitchen_user::{meal_preferences::UpdateInput, subscribe_command};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_update_meal_preferences() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command =
        imkitchen_user::meal_preferences::Command(state.evento.clone(), state.pool.clone());
    let users = helpers::create_users(&state, vec!["john", "albert"]).await?;
    let john = users.first().unwrap();
    let albert = users.get(1).unwrap();

    command
        .update(
            UpdateInput {
                cuisine_variety_weight: 0.3,
                household_size: 2,
                dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            },
            &Metadata::by(john.to_owned()),
        )
        .await?;

    let preferences = command.load(john).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 0.3);
    assert_eq!(preferences.item.household_size, 2);
    assert_eq!(
        preferences.item.dietary_restrictions,
        vec![DietaryRestriction::Vegetarian]
    );

    command
        .update(
            UpdateInput {
                cuisine_variety_weight: 0.5,
                household_size: 4,
                dietary_restrictions: vec![
                    DietaryRestriction::GlutenFree,
                    DietaryRestriction::Vegan,
                ],
            },
            &Metadata::by(john.to_owned()),
        )
        .await?;

    let preferences = command.load(john).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 0.5);
    assert_eq!(preferences.item.household_size, 4);
    assert_eq!(
        preferences.item.dietary_restrictions,
        vec![DietaryRestriction::GlutenFree, DietaryRestriction::Vegan,]
    );

    command
        .update(
            UpdateInput {
                cuisine_variety_weight: 1.0,
                household_size: 1,
                dietary_restrictions: vec![DietaryRestriction::GlutenFree],
            },
            &Metadata::by_as(john.to_owned(), albert.to_owned()),
        )
        .await?;

    let preferences = command.load(albert).await?;

    assert_eq!(preferences.item.cuisine_variety_weight, 1.0);
    assert_eq!(preferences.item.household_size, 1);
    assert_eq!(
        preferences.item.dietary_restrictions,
        vec![DietaryRestriction::GlutenFree]
    );

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
