use imkitchen_types::contact::Status;
use temp_dir::TempDir;

#[tokio::test]
async fn test_mark_read_and_reply() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = crate::helpers::setup_test_state(path).await?;
    let cmd = imkitchen_core::contact::Module::new(state);
    let contact_id = crate::helpers::create_submit(&cmd, "john.doe").await?;

    let contact = cmd.load(&contact_id).await?.unwrap();
    assert_eq!(contact.status, Status::Unread);

    cmd.mark_read_and_reply(&contact_id, "").await?;

    let contact = cmd.load(&contact_id).await?.unwrap();
    assert_eq!(contact.status, Status::Read);

    Ok(())
}
