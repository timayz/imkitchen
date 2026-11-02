use crate::helpers;
use imkitchen_core::dish::ComposeInput;
use imkitchen_core::shopping::{ChangeRecipeStatus, GenerateList, ToggleInput};
use imkitchen_types::meal_preferences::MealSources;
use imkitchen_types::recipe::RecipeType;
use imkitchen_types::shopping::RecipeStatus;
use temp_dir::TempDir;

struct Seeded {
    state: imkitchen_core::State<evento::Sqlite>,
    shopping: imkitchen_core::shopping::Module<evento::Sqlite>,
    dish_id: String,
    main: String,
    side: String,
    other: String,
}

/// A dish (main + side, both using potato) and a loose main on the list.
async fn seed(dir: &TempDir) -> anyhow::Result<Seeded> {
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());
    let dish = imkitchen_core::dish::Module::new(state.clone());

    let main = helpers::import_recipe(&recipe, "main", "beef", 500, 4, "john").await?;
    let side = helpers::import_typed_recipe(
        &recipe,
        "side",
        "potato",
        800,
        4,
        RecipeType::Accompaniment,
        false,
        "john",
    )
    .await?;
    let other = helpers::import_recipe(&recipe, "other", "potato", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    let dish_id = dish
        .create(
            ComposeInput {
                name: "Beef & potatoes".to_owned(),
                recipe_ids: vec![main.clone(), side.clone()],
            },
            "john",
        )
        .await?;
    helpers::run_dish_subscription(&state).await?;

    shopping.add_recipe(&other, 4, "john").await?;
    shopping.add_dish(&dish_id, 4, "john").await?;

    Ok(Seeded {
        state,
        shopping,
        dish_id,
        main,
        side,
        other,
    })
}

#[tokio::test]
async fn removes_every_recipe_of_the_dish_and_keeps_the_rest() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let Seeded {
        shopping,
        dish_id,
        main,
        side,
        other,
        ..
    } = seed(&dir).await?;

    // Cooking progress and a check on the shared ingredient, to be cleaned up.
    shopping
        .change_recipe_status(
            ChangeRecipeStatus {
                recipe_id: main.clone(),
                status: RecipeStatus::Cooking(0),
            },
            "john",
        )
        .await?;
    let before = shopping.state("john", 4).await?;
    assert_eq!(before.ingredients.len(), 2);
    let potato = before
        .ingredients
        .iter()
        .find(|i| i.name == "potato")
        .map(|i| i.key())
        .expect("potato on the list");
    let beef = before
        .ingredients
        .iter()
        .find(|i| i.name == "beef")
        .map(|i| i.key())
        .expect("beef on the list");
    for key in [&potato, &beef] {
        shopping
            .toggle(ToggleInput { name: key.clone() }, "john")
            .await?;
    }

    shopping.remove_dish(&dish_id, 4, "john").await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![other.clone()]);
    assert!(loaded.dishes.is_empty());
    assert!(!loaded.statuses.contains_key(&main));
    assert!(!loaded.servings.contains_key(&main));
    assert!(!loaded.servings.contains_key(&side));
    assert_eq!(loaded.servings.get(&other), Some(&4));
    // Only `other` is left: beef is gone, potato stays checked at its new
    // quantity key or is forgotten when the key changed.
    assert_eq!(loaded.ingredients.len(), 1);
    assert!(
        loaded
            .checked
            .iter()
            .all(|k| loaded.ingredients.contains(k))
    );
    assert!(!loaded.checked.contains(&beef));

    let view = shopping.state("john", 4).await?;
    assert_eq!(view.ingredients.len(), 1);
    assert_eq!(view.ingredients[0].name, "potato");
    assert_eq!(view.ingredients[0].quantity, 300);

    // Gone from the list: removing it again is a not-found.
    assert!(matches!(
        shopping.remove_dish(&dish_id, 4, "john").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));

    Ok(())
}

#[tokio::test]
async fn unknown_or_foreign_dish_is_not_found() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let Seeded {
        shopping, dish_id, ..
    } = seed(&dir).await?;

    assert!(matches!(
        shopping.remove_dish("nope", 4, "john").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));
    // Another user has no such group on their own list.
    assert!(matches!(
        shopping.remove_dish(&dish_id, 4, "albert").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes.len(), 3);
    assert_eq!(loaded.dishes.len(), 1);

    Ok(())
}

/// Removing one recipe shrinks its group; removing the last member drops it.
#[tokio::test]
async fn removing_recipes_one_by_one_shrinks_then_drops_the_group() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let Seeded {
        shopping,
        dish_id,
        main,
        side,
        other,
        ..
    } = seed(&dir).await?;

    shopping.remove_recipe(&side, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(
        loaded.dish(&dish_id).map(|d| d.recipe_ids.clone()),
        Some(vec![main.clone()])
    );

    shopping.remove_recipe(&main, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.dishes.is_empty());
    assert_eq!(loaded.recipes, vec![other]);

    Ok(())
}

/// Generating replaces the list: groups placed by hand go with it.
#[tokio::test]
async fn generating_clears_the_groups_placed_by_hand() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let Seeded {
        state,
        shopping,
        dish_id,
        ..
    } = seed(&dir).await?;
    let dish = imkitchen_core::dish::Module::new(state.clone());
    dish.delete(&dish_id, "john").await?;
    helpers::run_dish_subscription(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 2,
                household_size: 4,
                sources: MealSources::MainCourses,
                randomize: None,
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.dishes.is_empty());
    assert_eq!(loaded.recipes.len(), 2);

    Ok(())
}
