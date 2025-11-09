use imkitchen_shared::Metadata;
use imkitchen_user::{ActivateInput, Role, SuspendInput, subscribe_command};

mod helpers;

#[tokio::test]
async fn test_login_failure() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let user = helpers::create_user(&state, "john.doe").await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    command
        .suspend(
            SuspendInput {
                id: user.to_owned(),
            },
            Metadata::default(),
        )
        .await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::Suspend);

    command
        .activate(
            ActivateInput {
                id: user.to_owned(),
            },
            Metadata::default(),
        )
        .await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    subscribe_command()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    Ok(())
}
