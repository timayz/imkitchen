use crate::helpers;
use imkitchen_core::shopping::ToggleInput;
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
    assert_eq!(loaded.recipes, vec![b.clone()]);
    assert_eq!(loaded.ingredients.len(), 1);

    let view = shopping.state("john", 4).await?;
    assert_eq!(view.ingredients.len(), 1);
    assert_eq!(view.ingredients[0].name, "sugar");

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

/// Checks belong to the list as it is: once the recipe needing an ingredient
/// is removed, a recipe added later with the same ingredient starts unchecked.
#[tokio::test]
async fn test_remove_recipe_forgets_checks_of_dropped_ingredients() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let bread = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    let cake = helpers::import_recipe(&recipe_cmd, "Cake", "flour", 200, 4, "john").await?;
    let soup = helpers::import_recipe(&recipe_cmd, "Soup", "leek", 300, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping.add_recipe(&bread, 4, "john").await?;
    shopping.add_recipe(&soup, 4, "john").await?;
    let view = shopping.state("john", 4).await?;
    let flour = view
        .ingredients
        .iter()
        .find(|i| i.name == "flour")
        .unwrap()
        .key();
    let leek = view
        .ingredients
        .iter()
        .find(|i| i.name == "leek")
        .unwrap()
        .key();
    shopping
        .toggle(
            ToggleInput {
                name: flour.clone(),
            },
            "john",
        )
        .await?;
    shopping
        .toggle(ToggleInput { name: leek.clone() }, "john")
        .await?;

    // Removing one recipe keeps the checks of the ingredients still needed.
    shopping.remove_recipe(&bread, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(!loaded.checked.contains(&flour));
    assert!(loaded.checked.contains(&leek));

    // Emptying the list leaves nothing checked.
    shopping.remove_recipe(&soup, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.checked.is_empty());

    // The same ingredient needed by a new recipe starts unchecked.
    shopping.add_recipe(&cake, 4, "john").await?;
    let view = shopping.state("john", 4).await?;
    assert_eq!(view.ingredients.len(), 1);
    assert!(view.checked.is_empty());

    Ok(())
}
