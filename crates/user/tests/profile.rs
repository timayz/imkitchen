use imkitchen_shared::recipe::DietaryRestriction;
use imkitchen_user::meal_preferences::UpdateInput;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_update_meal_preferences() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.3,
                household_size: 2,
                dietary_restrictions: vec![DietaryRestriction::Vegetarian],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.cuisine_variety_weight, 0.3);
    assert_eq!(preferences.household_size, 2);
    assert_eq!(
        preferences.dietary_restrictions,
        vec![DietaryRestriction::Vegetarian]
    );

    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.5,
                household_size: 4,
                dietary_restrictions: vec![
                    DietaryRestriction::GlutenFree,
                    DietaryRestriction::Vegan,
                ],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.cuisine_variety_weight, 0.5);
    assert_eq!(preferences.household_size, 4);
    assert_eq!(
        preferences.dietary_restrictions,
        vec![DietaryRestriction::GlutenFree, DietaryRestriction::Vegan,]
    );

    Ok(())
}
