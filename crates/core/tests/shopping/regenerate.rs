use crate::helpers;
use imkitchen_core::shopping::{ChangeRecipeStatus, GenerateList, ToggleInput};
use imkitchen_types::shopping::RecipeStatus;
use temp_dir::TempDir;

/// Regenerating replaces the whole list: manually-added recipes are dropped,
/// checked ingredients and cooking statuses are reset. (Product decision:
/// "regenerate clears everything".)
#[tokio::test]
async fn test_regenerate_replaces_manual_recipes_and_resets_progress() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    // The manual recipe belongs to another user, so it is never in john's pool
    // and can only reach his list by being added by hand.
    let manual = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "albert").await?;
    let pooled = helpers::import_recipe(&recipe_cmd, "Cake", "sugar", 200, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    helpers::run_pool_subscription(&state).await?;

    shopping.add_recipe(&manual, 4, "john").await?;
    let flour = shopping.state("john", 4).await?.ingredients[0].key();
    shopping.toggle(ToggleInput { name: flour }, "john").await?;
    shopping
        .change_recipe_status(
            ChangeRecipeStatus {
                recipe_id: manual.clone(),
                status: RecipeStatus::Completed,
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(loaded.recipes, vec![manual.clone()]);
    assert_eq!(loaded.checked.len(), 1);
    assert_eq!(loaded.status(&manual), RecipeStatus::Completed);

    shopping
        .generate(
            GenerateList {
                count: 7,
                household_size: 4,
                randomize: None,
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(
        loaded.recipes,
        vec![pooled],
        "manual recipe should be dropped, only pooled recipes remain"
    );
    assert!(loaded.checked.is_empty(), "checks reset on regenerate");
    assert!(
        loaded.statuses.is_empty(),
        "cooking statuses reset on regenerate"
    );
    assert_eq!(loaded.ingredients.len(), 1);

    Ok(())
}
