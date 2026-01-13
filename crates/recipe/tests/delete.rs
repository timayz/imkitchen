use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_delete() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_recipe::Command::new(state);

    let recipe_id = cmd.create("john", "john_doe".to_owned()).await?;
    let recipe = cmd.load(&recipe_id).await?.unwrap();
    assert!(!recipe.is_deleted);

    cmd.delete(&recipe_id, "john").await?;

    let result = cmd.load(&recipe_id).await?;
    assert!(result.is_none());

    let err = cmd.delete(&recipe_id, "john").await.unwrap_err();

    assert_eq!(err.to_string(), "recipe not found".to_owned());

    Ok(())
}
