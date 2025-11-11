use imkitchen_shared::Metadata;
use imkitchen_user::{Role, subscribe_command};

mod helpers;

#[tokio::test]
async fn test_suspend() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let user = helpers::create_user(&state, "john.doe").await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    command.suspend(&user, Metadata::default()).await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::Suspend);

    command.activate(&user, Metadata::default()).await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
