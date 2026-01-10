use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_mark_read_and_reply() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_contact::Command::new(state.clone());
    let contact_id = helpers::create_submit(&cmd, "john.doe").await?;

    cmd.mark_read_and_reply(&contact_id, "").await?;
    cmd.reopen(&contact_id, "").await?;
    cmd.resolve(contact_id, "").await?;

    imkitchen_contact::global_stat::subscription()
        .data(state.write_db.clone())
        .unretry_execute(&state.executor)
        .await?;

    Ok(())
}
