use crate::helpers;
use imkitchen_core::shopping::Generate;
use temp_dir::TempDir;

/// Regenerating from the meal plan replaces the whole recipe set: manually-added
/// recipes are dropped and only the meal-plan's recipes remain. (Product
/// decision: "regenerate clears everything".)
#[tokio::test]
async fn test_regenerate_replaces_manual_recipes() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());
    let shopping = imkitchen_core::shopping::Module::new(state.clone());

    let manual = helpers::import_recipe(&recipe_cmd, "Bread", "flour", 500, 4, "john").await?;
    let planned = helpers::import_recipe(&recipe_cmd, "Cake", "sugar", 200, 4, "john").await?;
    helpers::run_shopping_subscription(&state).await?;

    // Seed a meal-plan slot for the range (what `generate` reads from).
    let recipe_ids = bitcode::encode(&vec![planned.clone()]);
    sqlx::query("INSERT INTO shopping_slot (user_id, date, recipe_ids) VALUES (?, ?, ?)")
        .bind("john")
        .bind(20260101_i64)
        .bind(recipe_ids)
        .execute(&state.write_db)
        .await?;

    // Manually add a recipe that is NOT in the plan.
    shopping.add_recipe(&manual, 4, "john").await?;
    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert!(loaded.recipes.contains(&manual));

    // Regenerate from the plan.
    shopping
        .generate(
            Generate {
                date: 20260101,
                days: 7,
                household_size: 4,
            },
            "john",
        )
        .await?;

    let loaded = shopping.load("john").await?.expect("shopping aggregate");
    assert_eq!(
        loaded.recipes.iter().cloned().collect::<Vec<_>>(),
        vec![planned],
        "manual recipe should be dropped, only planned recipe remains"
    );
    assert!(!loaded.recipes.contains(&manual));
    assert_eq!(loaded.ingredients.len(), 1);

    Ok(())
}
