use crate::helpers;
use temp_dir::TempDir;

#[tokio::test]
async fn test_remove_recipe() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    let b = helpers::import_recipe(&recipe_cmd, "Cake", "sugar", 200, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&a, 4, "john").await?;
    shopping.add_recipe(&b, 4, "john").await?;

    // Remove one — only the other recipe's ingredient remains.
    shopping.remove_recipe(&a, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(
        loaded.recipes.iter().cloned().collect::<Vec<_>>(),
        vec![b.clone()]
    );
    assert_eq!(loaded.ingredients.len(), 1);

    helpers::run_shopping_list_subscription(&state).await?;
    let row = shopping.find("john").await?.expect("shopping list row");
    assert_eq!(row.ingredients.0.len(), 1);
    assert_eq!(row.ingredients.0[0].name, "sugar");

    // Remove the last — empty list.
    shopping.remove_recipe(&b, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.recipes.is_empty());
    assert!(loaded.ingredients.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_remove_recipe_not_in_list_is_not_found() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let a = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    let err = shopping.remove_recipe(&a, 4, "john").await.unwrap_err();
    assert_eq!(err.to_string(), "recipe not found".to_owned());

    Ok(())
}
