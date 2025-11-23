use imkitchen_shared::Metadata;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_delete() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_recipe::Command(state.evento.clone(), state.pool.clone());
    let john = Metadata::by("john".to_owned());

    let recipe = command.create(&john).await?;
    let loaded = command.load(&recipe).await?;
    assert!(!loaded.item.deleted);

    command.delete_with(loaded, &john).await?;

    let loaded = command.load(&recipe).await?;

    assert!(loaded.item.deleted);

    let err = command.delete_with(loaded, &john).await.unwrap_err();

    assert_eq!(err.to_string(), "recipe already deleted".to_owned());

    Ok(())
}
