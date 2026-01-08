use imkitchen_shared::user::{Role, State};
use imkitchen_user::load;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_suspend() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let user_id = helpers::create_user(&state, "john.doe").await?;

    let user = load(&state.evento, &state.pool, &user_id).await?.unwrap();
    assert_eq!(user.role, Role::User);

    user.suspend("").await?;

    let user = load(&state.evento, &state.pool, &user_id).await?.unwrap();
    assert_eq!(user.state, State::Suspended);

    user.activate("").await?;

    let user = load(&state.evento, &state.pool, &user_id).await?.unwrap();
    assert_eq!(user.role, Role::User);

    Ok(())
}
