use imkitchen_identity::meal_preferences::{UpdateInput, meal_sources};
use imkitchen_types::meal_preferences::MealSources;
use imkitchen_types::recipe::{DietaryRestriction, IngredientCategory, RecipeType};
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
                sources: MealSources::default(),
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
                sources: MealSources::default(),
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
    assert_eq!(
        preferences.aisle_order,
        IngredientCategory::DEFAULT_AISLE_ORDER.to_vec()
    );
    assert_eq!(preferences.sources, MealSources::DishesAndMainCourses);

    Ok(())
}

#[tokio::test]
async fn test_meal_sources_round_trip_and_never_both_off() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    assert_eq!(meal_sources(true, true)?, MealSources::DishesAndMainCourses);
    assert_eq!(meal_sources(true, false)?, MealSources::Dishes);
    assert_eq!(meal_sources(false, true)?, MealSources::MainCourses);
    let err = meal_sources(false, false).unwrap_err();
    assert!(
        matches!(err, imkitchen_core::Error::User(_)),
        "both off is a user error, got {err:?}"
    );

    for sources in [MealSources::Dishes, MealSources::MainCourses] {
        cmd.meal_preferences
            .update(
                john,
                UpdateInput {
                    cuisine_variety_weight: 1.0,
                    sources,
                    household_size: 4,
                    dietary_restrictions: vec![],
                    recipe_types: vec![],
                },
            )
            .await?;
        assert_eq!(cmd.meal_preferences.load(john).await?.sources, sources);
    }

    // The aisle order is its own write and leaves the sources alone.
    cmd.meal_preferences
        .set_aisle_order(john, vec![IngredientCategory::Frozen])
        .await?;
    assert_eq!(
        cmd.meal_preferences.load(john).await?.sources,
        MealSources::MainCourses
    );

    Ok(())
}

#[tokio::test]
async fn test_set_aisle_order_canonicalises() -> anyhow::Result<()> {
    use IngredientCategory::*;

    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    // Partial and repeated: first occurrences keep their place, the rest
    // follow in default order.
    cmd.meal_preferences
        .set_aisle_order(john, vec![Bakery, Frozen, Bakery])
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;
    assert_eq!(
        preferences.aisle_order,
        vec![
            Bakery,
            Frozen,
            FruitsAndVegetables,
            Butcher,
            Seafood,
            DairyAndEggs,
            Grocery,
            Refrigerated,
            SnacksAndConfectionery,
        ]
    );

    // Nothing at all means the default.
    cmd.meal_preferences.set_aisle_order(john, vec![]).await?;
    let preferences = cmd.meal_preferences.load(john).await?;
    assert_eq!(
        preferences.aisle_order,
        IngredientCategory::DEFAULT_AISLE_ORDER.to_vec()
    );

    Ok(())
}

/// The aisle order may be the first thing a user ever saves. The other
/// preferences must then still read as their defaults, not as a zeroed row.
#[tokio::test]
async fn test_set_aisle_order_on_fresh_stream_keeps_default_preferences() -> anyhow::Result<()> {
    use IngredientCategory::*;

    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let users = helpers::create_users(&cmd, vec!["john"]).await?;
    let john = users.first().unwrap();

    cmd.meal_preferences
        .set_aisle_order(john, vec![Seafood])
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;
    assert_eq!(preferences.household_size, 4);
    assert_eq!(preferences.cuisine_variety_weight, 1.0);
    assert!(preferences.dietary_restrictions.is_empty());
    assert_eq!(
        preferences.recipe_types,
        RecipeType::default_meal_plan_types()
    );
    assert_eq!(preferences.aisle_order[0], Seafood);

    // A later preferences save keeps the custom aisle order.
    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.5,
                sources: MealSources::default(),
                household_size: 2,
                dietary_restrictions: vec![],
                recipe_types: vec![],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;
    assert_eq!(preferences.household_size, 2);
    assert_eq!(preferences.aisle_order[0], Seafood);

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
    assert_eq!(preferences.sources, MealSources::DishesAndMainCourses);

    // Writing on top of a legacy stream must line up with `original_version`.
    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.5,
                sources: MealSources::default(),
                household_size: 6,
                dietary_restrictions: vec![],
                recipe_types: vec![RecipeType::Condiment],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;

    assert_eq!(preferences.household_size, 6);
    assert_eq!(preferences.recipe_types, vec![RecipeType::Condiment]);
    // No `AisleOrderChanged` on the stream: the default order.
    assert_eq!(
        preferences.aisle_order,
        IngredientCategory::DEFAULT_AISLE_ORDER.to_vec()
    );

    // A custom order survives a later `Changed` on replay.
    cmd.meal_preferences
        .set_aisle_order(john, vec![IngredientCategory::Frozen])
        .await?;
    cmd.meal_preferences
        .update(
            john,
            UpdateInput {
                cuisine_variety_weight: 0.5,
                sources: MealSources::default(),
                household_size: 6,
                dietary_restrictions: vec![],
                recipe_types: vec![RecipeType::Condiment],
            },
        )
        .await?;

    let preferences = cmd.meal_preferences.load(john).await?;
    assert_eq!(preferences.aisle_order[0], IngredientCategory::Frozen);
    assert_eq!(preferences.aisle_order.len(), 9);

    Ok(())
}
