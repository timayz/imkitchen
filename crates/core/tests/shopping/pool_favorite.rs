use crate::helpers;
use evento::WriteBuilder;
use imkitchen_core::recipe::favorite::legacy_id;
use imkitchen_types::favorite::Saved;
use temp_dir::TempDir;

/// A favorite saved twice for the same recipe and cook, once on the legacy
/// stream and once on the current one (what happened in production while
/// the detail page could not find legacy favorites), must not stall the pool
/// subscription on a duplicate row: the recipe is in the cook's pool once.
#[tokio::test]
async fn duplicate_saved_events_keep_one_pool_row() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let state = helpers::setup_test_state(dir.child("db.sqlite3")).await?;
    let recipe_cmd = imkitchen_core::recipe::Module::new(state.clone());

    let recipe_id = helpers::import_recipe(&recipe_cmd, "Paella", "rice", 300, 4, "chef").await?;
    helpers::run_pool_subscription(&state).await?;

    let saved = Saved {
        recipe_id: recipe_id.clone(),
        recipe_owner: "chef".to_owned(),
    };
    WriteBuilder::new(legacy_id(&recipe_id, "cook"))
        .event(&saved)
        .requested_by("cook")
        .commit(&state.executor)
        .await?;
    WriteBuilder::new(evento::hash_ids(vec![&recipe_id, "cook"]))
        .event(&saved)
        .requested_by("cook")
        .commit(&state.executor)
        .await?;
    helpers::run_pool_subscription(&state).await?;

    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT user_id FROM meal_plan_recipe WHERE id = ? ORDER BY user_id")
            .bind(&recipe_id)
            .fetch_all(&state.read_db)
            .await?;
    assert_eq!(rows, vec![("chef".to_owned(),), ("cook".to_owned(),)]);

    Ok(())
}
