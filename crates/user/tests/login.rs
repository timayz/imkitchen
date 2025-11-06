use imkitchen_shared::Metadata;
use imkitchen_user::{LoginInput, subscribe_command};

mod helpers;

#[tokio::test]
async fn test_login_failure() -> anyhow::Result<()> {
    let state = helpers::setup_test_state().await?;
    let command = imkitchen_user::Command(state.evento.clone(), state.pool.clone());
    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkiichen.localhost".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
            },
            Metadata::default(),
        )
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let user = helpers::create_user(&state, "john.doe").await?;

    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkiichen.localhost".to_owned(),
                password: "my_password3".to_owned(),
                lang: "en".to_owned(),
            },
            Metadata::default(),
        )
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkiichen.localhos".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
            },
            Metadata::default(),
        )
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkiichen.localhost".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
            },
            Metadata::default(),
        )
        .await;

    assert_eq!(resp.unwrap(), user);

    subscribe_command()
        .data(state.pool.clone())
        .unsafe_oneshot(&state.evento)
        .await?;

    Ok(())
}
