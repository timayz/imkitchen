use std::time::{Duration, SystemTime, UNIX_EPOCH};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_toggle_life_premium() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    cmd.subscription.toggle_life_premium(&user_id, "").await?;

    let subscription = cmd.subscription.load(&user_id).await?;
    let expire_at = (SystemTime::now() + Duration::from_secs(9 * 52))
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    assert!(subscription.expire_at > expire_at);

    cmd.subscription.toggle_life_premium(&user_id, "").await?;

    let subscription = cmd.subscription.load(&user_id).await?;
    assert!(subscription.expire_at == 0);

    cmd.subscription.toggle_life_premium(&user_id, "").await?;

    let subscription = cmd.subscription.load(&user_id).await?;
    assert!(subscription.expire_at > expire_at);

    Ok(())
}
