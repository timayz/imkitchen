use imkitchen_shared::Metadata;
use imkitchen_user::{Role, State, subscribe_command};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_suspend() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_user::Command {
        evento: state.evento.clone(),
        read_db: state.pool.clone(),
        write_db: state.pool.clone(),
    };
    let user = helpers::create_user(&state, "john.doe").await?;
    let metadata = Metadata::default();

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    command.suspend(&user, &metadata).await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.state, State::Suspended);

    command.activate(&user, &metadata).await?;

    let loaded = command.load(&user).await?;
    assert_eq!(loaded.item.role, Role::User);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
