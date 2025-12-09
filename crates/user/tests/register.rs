use imkitchen_shared::Metadata;
use imkitchen_user::{RegisterInput, Status, subscribe_command};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn validate_unique_emails() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let metadata = Metadata::default();
    let user_1 = command
        .register(
            RegisterInput {
                email: "john.doe@imkitchen.localhost".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
            },
            &metadata,
        )
        .await?;
    let user_2 = command
        .register(
            RegisterInput {
                email: "john.doe@imkitchen.localhost".to_owned(),
                password: "my_password_v2".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
            },
            &metadata,
        )
        .await?;

    let user_1_agg = command.load(&user_1).await?;
    let user_2_agg = command.load(&user_2).await?;

    assert_eq!(user_1_agg.item.status, Status::Processing);
    assert_eq!(user_2_agg.item.status, Status::Processing);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    let user_1_agg = command.load(&user_1).await?;
    let user_2_agg = command.load(&user_2).await?;

    assert_eq!(user_1_agg.item.status, Status::Idle);
    assert_eq!(user_2_agg.item.status, Status::Failed);
    assert_eq!(
        user_2_agg.item.failed_reason,
        Some("Email already exists".to_owned())
    );

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
