use imkitchen_contact::Status;
use imkitchen_shared::Metadata;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_resolved() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_contact::Command(state.evento.clone(), state.pool.clone());
    let contact = helpers::create_submit(&state, "john.doe").await?;

    let loaded = command.load(&contact).await?;
    assert_eq!(loaded.item.status, Status::Unread);

    command.resolve(&contact, &Metadata::default()).await?;

    let loaded = command.load(&contact).await?;
    assert_eq!(loaded.item.status, Status::Resolved);

    Ok(())
}
