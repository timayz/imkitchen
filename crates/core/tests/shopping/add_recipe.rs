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
    assert!(loaded.recipes.contains(&recipe_id));
    assert_eq!(loaded.ingredients.len(), 1);

    // Read model reflects it after the list subscription runs.
    helpers::run_shopping_list_subscription(&state).await?;
    let row = shopping.find("john").await?.expect("shopping list row");
    let recipes = row.recipes.expect("recipes column").0;
    assert_eq!(recipes, vec![recipe_id]);
    assert_eq!(row.ingredients.0.len(), 1);
    assert_eq!(row.ingredients.0[0].name, "carrot");

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
