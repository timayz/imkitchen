use imkitchen_shared::contact::Status;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_mark_read_and_reply() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_contact::Command::new(state);
    let contact_id = helpers::create_submit(&cmd, "john.doe").await?;

    let contact = cmd.load(&contact_id).await?.unwrap();
    assert_eq!(contact.status, Status::Unread);

    cmd.mark_read_and_reply(&contact_id, "").await?;

    let contact = cmd.load(&contact_id).await?.unwrap();
    assert_eq!(contact.status, Status::Read);

    Ok(())
}
