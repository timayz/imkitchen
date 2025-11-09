use imkitchen_contact::ContactStatus;
use imkitchen_shared::Metadata;

mod helpers;

#[tokio::test]
async fn test_mark_as_read_and_replay() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_contact::Command(state.evento.clone(), state.pool.clone());
    let contact = helpers::create_submit(&state, "john.doe").await?;

    let loaded = command.load(&contact).await?;
    assert_eq!(loaded.item.status, ContactStatus::Unread.to_string());

    command
        .mark_as_read_and_replay(&contact, Metadata::default())
        .await?;

    let loaded = command.load(&contact).await?;
    assert_eq!(loaded.item.status, ContactStatus::Read.to_string());

    Ok(())
}
