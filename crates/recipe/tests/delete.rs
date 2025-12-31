use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_delete() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    let recipe_id =
        imkitchen_recipe::Command::create(&state.evento, "john", "john_doe".to_owned()).await?;
    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    assert!(!recipe.is_deleted);

    recipe.delete("john").await?;

    let recipe = imkitchen_recipe::load(&state.evento, &state.pool, &recipe_id)
        .await?
        .unwrap();
    assert!(recipe.is_deleted);

    let err = recipe.delete("john").await.unwrap_err();

    assert_eq!(err.to_string(), "recipe not found".to_owned());

    Ok(())
}
