use imkitchen_shared::contact::Status;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_reopen() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let contact_id = helpers::create_submit(&state, "john.doe").await?;

    let contact = imkitchen_contact::load(&state.evento, &state.pool, &contact_id)
        .await?
        .unwrap();
    assert_eq!(contact.status, Status::Unread);

    contact.reopen("").await?;

    let contact = imkitchen_contact::load(&state.evento, &state.pool, &contact_id)
        .await?
        .unwrap();
    assert_eq!(contact.status, Status::Read);

    Ok(())
}
