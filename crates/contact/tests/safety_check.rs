use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_mark_read_and_reply() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let contact_id = helpers::create_submit(&state, "john.doe").await?;

    let contact = imkitchen_contact::load(&state.evento, &state.pool, &contact_id)
        .await?
        .unwrap();

    contact.mark_read_and_reply("").await?;

    let contact = imkitchen_contact::load(&state.evento, &state.pool, &contact_id)
        .await?
        .unwrap();

    contact.reopen("").await?;

    let contact = imkitchen_contact::load(&state.evento, &state.pool, &contact_id)
        .await?
        .unwrap();

    contact.resolve("").await?;

    imkitchen_contact::admin::create_projection()
        .subscription()
        .data(state.pool.clone())
        .unretry_execute(&state.evento)
        .await?;

    imkitchen_contact::global_stat::create_projection()
        .subscription()
        .data(state.pool.clone())
        .unretry_execute(&state.evento)
        .await?;

    Ok(())
}
