use imkitchen_shared::user::{Role, State};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_suspend() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert_eq!(user.role, Role::User);

    cmd.suspend(&user_id, "").await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert_eq!(user.state, State::Suspended);

    cmd.activate(&user_id, "").await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert_eq!(user.role, Role::User);

    Ok(())
}
