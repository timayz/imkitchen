use imkitchen_user::{Action, Metadata, RegisterInput, Status, User, subscribe_command};

mod helpers;

#[tokio::test]
async fn validate_unique_emails() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let user_1 = command
        .register(
            RegisterInput {
                email: "john.doe@imkiichen.localhost".to_owned(),
                password: "my_password".to_owned(),
            },
            Metadata::default(),
        )
        .await?;
    let user_2 = command
        .register(
            RegisterInput {
                email: "john.doe@imkiichen.localhost".to_owned(),
                password: "my_password_v2".to_owned(),
            },
            Metadata::default(),
        )
        .await?;

    let user_1_agg = evento::load::<User, _>(&state.evento, &user_1).await?;
    let user_2_agg = evento::load::<User, _>(&state.evento, &user_2).await?;

    assert_eq!(
        user_1_agg.item.status,
        Status::Processing(Action::Registration)
    );
    assert_eq!(
        user_2_agg.item.status,
        Status::Processing(Action::Registration)
    );

    subscribe_command()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    let user_1_agg = evento::load::<User, _>(&state.evento, user_1).await?;
    let user_2_agg = evento::load::<User, _>(&state.evento, user_2).await?;

    assert_eq!(user_1_agg.item.status, Status::Idle);
    assert_eq!(
        user_2_agg.item.status,
        Status::Failed("Email already exists".to_owned())
    );

    subscribe_command()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    Ok(())
}
