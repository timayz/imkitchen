use imkitchen_identity::meal_preferences::UpdateInput;
use imkitchen_types::recipe::{DietaryRestriction, RecipeType};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_update_meal_preferences() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.3,
                household_size: 2,
                dietary_restrictions: vec![DietaryRestriction::Vegetarian],
                // Out of order, and with the mandatory MainCourse included, to
                // pin the canonicalisation done by `update`.
                recipe_types: vec![
                    RecipeType::Beverage,
                    RecipeType::MainCourse,
                    RecipeType::Appetizer,
                ],
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
    assert_eq!(
        preferences.recipe_types,
        vec![RecipeType::Appetizer, RecipeType::Beverage]
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
                recipe_types: vec![],
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
    // An empty selection is a real choice ("no optional courses"), not a reason
    // to fall back to the default.
    assert!(preferences.recipe_types.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_meal_preferences_defaults() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.household_size, 4);
    assert_eq!(preferences.cuisine_variety_weight, 1.0);
    assert!(preferences.dietary_restrictions.is_empty());
    assert_eq!(
        preferences.recipe_types,
        RecipeType::default_meal_plan_types()
    );

    Ok(())
}

/// Streams written before `RecipeTypesChanged` existed carry only `Changed`.
/// They must still load, and must keep generating the courses they always did.
#[tokio::test]
async fn test_meal_preferences_legacy_changed_event() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state.clone());
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    evento::append(john)
        .event(&imkitchen_types::meal_preferences::Changed {
            household_size: 3,
            dietary_restrictions: vec![DietaryRestriction::Vegan],
            cuisine_variety_weight: 0.8,
        })
        .requested_by(john)
        .commit(&state.executor)
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.household_size, 3);
    assert_eq!(preferences.cuisine_variety_weight, 0.8);
    assert_eq!(
        preferences.recipe_types,
        RecipeType::default_meal_plan_types()
    );

    // Writing on top of a legacy stream must line up with `original_version`.
    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.5,
                household_size: 6,
                dietary_restrictions: vec![],
                recipe_types: vec![RecipeType::Condiment],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.household_size, 6);
    assert_eq!(preferences.recipe_types, vec![RecipeType::Condiment]);

    Ok(())
}
