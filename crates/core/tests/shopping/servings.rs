//! Per-recipe household size: generate stamps the chosen size on every
//! recipe, add/add_dish on theirs, remove forgets it, and a list written
//! before servings existed falls back to the caller's default.

use crate::helpers;
use evento::WriteBuilder;
use imkitchen_core::dish::ComposeInput;
use imkitchen_core::shopping::{GenerateList, Randomize};
use imkitchen_types::recipe::{Ingredient, IngredientCategory, IngredientUnit, RecipeType};
use imkitchen_types::shopping::RecipeAdded;
use temp_dir::TempDir;

fn quantity(view: &imkitchen_core::shopping::ShoppingState, name: &str) -> u32 {
    view.ingredients
        .iter()
        .find(|i| i.name == name)
        .map(|i| i.quantity)
        .unwrap_or_else(|| panic!("{name} not in the list"))
}

#[tokio::test]
async fn generate_records_the_household_size_on_every_recipe() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe, "Soup", "carrot", 300, 4, "john").await?;
    let b = helpers::import_recipe(&recipe, "Bread", "flour", 500, 2, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    shopping
        .generate(
            GenerateList {
                count: 2,
                household_size: 6,
                randomize: Some(Randomize {
                    cuisine_variety_weight: 1.0,
                    dietary_restrictions: vec![],
                    recipe_types: vec![],
                }),
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.servings.get(&a), Some(&6));
    assert_eq!(loaded.servings.get(&b), Some(&6));

    // The settings default (2) does not matter: every recipe is scaled to 6.
    let view = shopping.state("john", 2).await?;
    assert_eq!(view.servings(&a), 6);
    assert_eq!(view.servings(&b), 6);
    assert_eq!(quantity(&view, "carrot"), 300 * 6 / 4);
    assert_eq!(quantity(&view, "flour"), 500 * 6 / 2);

    Ok(())
}

#[tokio::test]
async fn add_recipe_scales_only_that_recipe_and_rescales_on_a_new_size() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe, "Soup", "carrot", 300, 4, "john").await?;
    let b = helpers::import_recipe(&recipe, "Bread", "flour", 500, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&a, 4, "john").await?;
    shopping.add_recipe(&b, 8, "john").await?;

    let view = shopping.state("john", 1).await?;
    assert_eq!(view.servings(&a), 4);
    assert_eq!(view.servings(&b), 8);
    assert_eq!(quantity(&view, "carrot"), 300);
    assert_eq!(quantity(&view, "flour"), 1000);

    // Same size again: nothing written.
    let before = shopping.load("john").await?.unwrap().aggregate_version;
    shopping.add_recipe(&b, 8, "john").await?;
    let loaded = shopping.load("john").await?.unwrap();
    assert_eq!(loaded.aggregate_version, before);

    // Another size: one `ServingsSet` only, the list keeps its order.
    shopping.add_recipe(&a, 8, "john").await?;
    let loaded = shopping.load("john").await?.unwrap();
    assert_eq!(loaded.aggregate_version, before + 1);
    assert_eq!(loaded.recipes, vec![a.clone(), b.clone()]);
    let view = shopping.state("john", 1).await?;
    assert_eq!(quantity(&view, "carrot"), 600);

    // Removing forgets the size.
    shopping.remove_recipe(&a, 1, "john").await?;
    let loaded = shopping.load("john").await?.unwrap();
    assert!(!loaded.servings.contains_key(&a));
    assert_eq!(loaded.servings.get(&b), Some(&8));

    Ok(())
}

#[tokio::test]
async fn add_recipe_rejects_an_impossible_household_size() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe, "Soup", "carrot", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    assert!(shopping.add_recipe(&a, 0, "john").await.is_err());
    assert!(shopping.add_recipe(&a, 1001, "john").await.is_err());
    assert!(shopping.load("john").await?.is_none());

    Ok(())
}

#[tokio::test]
async fn add_dish_rescales_recipes_already_in_the_list() -> anyhow::Result<()> {
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

    shopping.add_recipe(&side, 4, "john").await?;
    shopping.add_dish(&dish_id, 6, "john").await?;

    let loaded = shopping.load("john").await?.unwrap();
    assert_eq!(loaded.recipes, vec![side.clone(), main.clone()]);
    assert_eq!(loaded.servings.get(&side), Some(&6));
    assert_eq!(loaded.servings.get(&main), Some(&6));

    // Whole dish already there at that size: nothing written.
    let before = loaded.aggregate_version;
    shopping.add_dish(&dish_id, 6, "john").await?;
    assert_eq!(
        shopping.load("john").await?.unwrap().aggregate_version,
        before
    );

    // Same dish for more people: only the servings change.
    shopping.add_dish(&dish_id, 8, "john").await?;
    let loaded = shopping.load("john").await?.unwrap();
    assert_eq!(loaded.aggregate_version, before + 1);
    assert_eq!(loaded.recipes, vec![side.clone(), main.clone()]);
    assert_eq!(loaded.servings.get(&side), Some(&8));
    assert_eq!(loaded.servings.get(&main), Some(&8));

    Ok(())
}

#[tokio::test]
async fn a_list_without_recorded_servings_uses_the_default() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe, "Soup", "carrot", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    // A `RecipeAdded` as written before `ServingsSet` existed.
    WriteBuilder::new("john")
        .event(&RecipeAdded {
            recipe_id: a.clone(),
            recipe_ids: vec![a.clone()],
            ingredients: vec![Ingredient {
                name: "carrot".to_owned(),
                quantity: 300,
                unit: Some(IngredientUnit::G),
                category: Some(IngredientCategory::Grocery),
            }],
        })
        .requested_by("john")
        .commit(&state.executor)
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![a.clone()]);
    assert!(loaded.servings.is_empty());

    let view = shopping.state("john", 8).await?;
    assert_eq!(view.servings(&a), 8);
    assert_eq!(quantity(&view, "carrot"), 600);

    Ok(())
}
