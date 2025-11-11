use std::time::{Duration, SystemTime, UNIX_EPOCH};

use imkitchen_shared::Metadata;
use imkitchen_user::subscribe_command;

mod helpers;

#[tokio::test]
async fn test_toggle_life_premium() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let user = helpers::create_user(&state, "john.doe").await?;

    command
        .toggle_life_premium(&user, Metadata::default())
        .await?;

    let subscription = command.load_subscription(&user).await?;
    assert!(!subscription.item.expired);
    let expire_at = (SystemTime::now() + Duration::from_secs(9 * 365 * 24 * 60 * 60))
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    assert!(subscription.item.expire_at > expire_at);

    command
        .toggle_life_premium(&user, Metadata::default())
        .await?;

    let subscription = command.load_subscription(&user).await?;
    assert!(subscription.item.expired);

    command
        .toggle_life_premium(&user, Metadata::default())
        .await?;

    let subscription = command.load_subscription(&user).await?;
    assert!(!subscription.item.expired);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
