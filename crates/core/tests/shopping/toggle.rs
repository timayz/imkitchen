use crate::helpers;
use imkitchen_core::{recipe::UpdateInput, shopping::ToggleInput};
use imkitchen_types::recipe::{Ingredient, IngredientCategory, IngredientUnit, RecipeType};
use temp_dir::TempDir;

/// The ingredient set the aggregate stored when the recipe was added is a
/// snapshot: once the recipe is edited, the groceries view shows the new
/// ingredients, so toggling them must work too (and the dropped ones must be
/// rejected).
#[tokio::test]
async fn test_toggle_follows_recipe_edits() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let recipe_id = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;
    shopping.add_recipe(&recipe_id, 4, "john").await?;
    let flour = shopping.state("john", 4).await?.ingredients[0].key();

    let yeast = Ingredient {
        name: "yeast".to_owned(),
        quantity: 10,
        unit: Some(IngredientUnit::G),
        category: Some(IngredientCategory::Grocery),
    };
    recipe_cmd
        .update(
            UpdateInput {
                id: recipe_id.clone(),
                recipe_type: RecipeType::MainCourse,
                name: "Bread".to_owned(),
                origin: None,
                description: "desc".to_owned(),
                household_size: 4,
                prep_time: 10,
                cook_time: 25,
                ingredients: vec![yeast.clone()],
                instructions: vec![],
                dietary_restrictions: vec![],
                accepts_accompaniment: false,
                advance_prep: String::new(),
            },
            "john",
        )
        .await?;
    helpers::run_shopping_subscription(&state).await?;

    shopping
        .toggle(ToggleInput { name: yeast.key() }, "john")
        .await?;
    let view = shopping.state("john", 4).await?;
    assert_eq!(view.ingredients.len(), 1);
    assert!(view.checked.contains(&yeast.key()));

    assert!(
        shopping
            .toggle(ToggleInput { name: flour }, "john")
            .await
            .is_err(),
        "an ingredient no longer on the list cannot be toggled"
    );

    Ok(())
}
