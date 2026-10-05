use crate::helpers;
use temp_dir::TempDir;

#[tokio::test]
async fn test_add_recipe() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let recipe_id = helpers::import_recipe(&recipe_cmd, "Soup", "carrot", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&recipe_id, 4, "john").await?;

    // Aggregate reflects the new recipe + its ingredient.
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![recipe_id.clone()]);
    assert_eq!(loaded.ingredients.len(), 1);

    // The synchronous state view has the merged ingredient.
    let view = shopping.state("john", 4).await?;
    assert_eq!(view.recipe_ids, vec![recipe_id]);
    assert_eq!(view.ingredients.len(), 1);
    assert_eq!(view.ingredients[0].name, "carrot");

    Ok(())
}

#[tokio::test]
async fn test_add_recipe_keeps_list_order() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    let b = helpers::import_recipe(&recipe_cmd, "Cake", "sugar", 200, 4, "john").await?;
    let c = helpers::import_recipe(&recipe_cmd, "Soup", "carrot", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&b, 4, "john").await?;
    shopping.add_recipe(&c, 4, "john").await?;
    shopping.add_recipe(&a, 4, "john").await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![b, c, a]);

    Ok(())
}

#[tokio::test]
async fn test_add_recipe_is_idempotent() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let recipe_id = helpers::import_recipe(&recipe_cmd, "Soup", "carrot", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&recipe_id, 4, "john").await?;
    shopping.add_recipe(&recipe_id, 4, "john").await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes.len(), 1);
    assert_eq!(loaded.ingredients.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_add_unknown_recipe_is_not_found() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let err = shopping.add_recipe("missing", 4, "john").await.unwrap_err();
    assert_eq!(err.to_string(), "recipe not found".to_owned());

    Ok(())
}
