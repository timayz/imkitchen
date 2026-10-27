use crate::helpers;
use imkitchen_core::dish::ComposeInput;
use imkitchen_types::recipe::RecipeType;
use temp_dir::TempDir;

#[tokio::test]
async fn adds_the_whole_dish_in_meal_order_once() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
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
    let sauce = helpers::import_typed_recipe(
        &recipe,
        "sauce",
        "beef",
        50,
        2,
        RecipeType::Condiment,
        false,
        "john",
    )
    .await?;
    let other = helpers::import_recipe(&recipe, "other", "rice", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    let dish_id = dish
        .create(
            ComposeInput {
                name: "Beef & potatoes".to_owned(),
                recipe_ids: vec![sauce.clone(), main.clone(), side.clone()],
            },
            "john",
        )
        .await?;

    // The side is already in the list: it keeps its place.
    shopping.add_recipe(&other, 4, "john").await?;
    shopping.add_recipe(&side, 4, "john").await?;

    shopping.add_dish(&dish_id, 4, "john").await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(
        loaded.recipes,
        vec![other.clone(), side.clone(), main.clone(), sauce.clone()]
    );
    // beef (main + sauce), potato and rice: one key per distinct ingredient.
    assert_eq!(loaded.ingredients.len(), 3);
    assert!(loaded.ingredients.iter().any(|k| k.starts_with("beef-")));
    let version = loaded.aggregate_version;

    // Adding it again changes nothing.
    shopping.add_dish(&dish_id, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.aggregate_version, version);
    assert_eq!(loaded.recipes.len(), 4);

    Ok(())
}

#[tokio::test]
async fn skips_deleted_recipes_and_refuses_other_users_dishes() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
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

    recipe.delete(&side, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    assert!(matches!(
        shopping.add_dish(&dish_id, 4, "albert").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));
    assert!(matches!(
        shopping.add_dish("nope", 4, "john").await,
        Err(imkitchen_core::Error::NotFound(_))
    ));

    shopping.add_dish(&dish_id, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![main]);

    Ok(())
}
