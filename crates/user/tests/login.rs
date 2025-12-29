use imkitchen_user::{Command, LoginInput};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_login_failure() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let resp = Command::login(
        &state.evento,
        &state.pool,
        LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        },
    )
    .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let user = helpers::create_user(&state, "john.doe").await?;

    let resp = Command::login(
        &state.evento,
        &state.pool,
        LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password3".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        },
    )
    .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = Command::login(
        &state.evento,
        &state.pool,
        LoginInput {
            email: "john.doe@imkitchen.localhos".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        },
    )
    .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = Command::login(
        &state.evento,
        &state.pool,
        LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        },
    )
    .await;

    assert_eq!(resp.unwrap().0, user);

    Ok(())
}
