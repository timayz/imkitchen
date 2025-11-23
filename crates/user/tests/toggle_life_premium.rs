use std::time::{Duration, SystemTime, UNIX_EPOCH};

use imkitchen_shared::Metadata;
use imkitchen_user::subscribe_command;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_toggle_life_premium() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_user::subscription::Command(state.evento.clone(), state.pool.clone());
    let user = helpers::create_user(&state, "john.doe").await?;
    let metadata = Metadata::default();

    command.toggle_life_premium(&user, &metadata).await?;

    let subscription = command.load(&user).await?;
    assert!(!subscription.item.expired);
    let expire_at = (SystemTime::now() + Duration::from_secs(9 * 52))
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    assert!(subscription.item.expire_at > expire_at);

    command.toggle_life_premium(&user, &metadata).await?;

    let subscription = command.load(&user).await?;
    assert!(subscription.item.expired);

    command.toggle_life_premium(&user, &metadata).await?;

    let subscription = command.load(&user).await?;
    assert!(!subscription.item.expired);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
