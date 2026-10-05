use crate::helpers;
use imkitchen_core::shopping::ChangeRecipeStatus;
use imkitchen_types::shopping::RecipeStatus;
use temp_dir::TempDir;

#[tokio::test]
async fn test_change_recipe_status() -> anyhow::Result<()> {
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

    let view = shopping.state("john", 4).await?;
    assert_eq!(view.status(&a), RecipeStatus::Idle);

    for status in [RecipeStatus::Cooking(1), RecipeStatus::Completed] {
        shopping
            .change_recipe_status(
                ChangeRecipeStatus {
                    recipe_id: a.clone(),
                    status: status.clone(),
                },
                "john",
            )
            .await?;
        let view = shopping.state("john", 4).await?;
        assert_eq!(view.status(&a), status);
        assert_eq!(view.status(&b), RecipeStatus::Idle);
    }

    // The cooked recipe stays in the list until removed.
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![a.clone(), b.clone()]);

    // Removing it drops its status.
    shopping.remove_recipe(&a, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.statuses.is_empty());
    assert_eq!(loaded.recipes, vec![b]);

    Ok(())
}

#[tokio::test]
async fn test_change_status_of_recipe_not_in_list_is_not_found() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let err = shopping
        .change_recipe_status(
            ChangeRecipeStatus {
                recipe_id: "missing".to_owned(),
                status: RecipeStatus::Completed,
            },
            "john",
        )
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "recipe not found".to_owned());

    Ok(())
}
