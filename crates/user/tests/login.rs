use imkitchen_user::LoginInput;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_login_failure() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state);
    let resp = cmd
        .login(LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        })
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let user = helpers::create_user(&cmd, "john.doe").await?;

    let resp = cmd
        .login(LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password3".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        })
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = cmd
        .login(LoginInput {
            email: "john.doe@imkitchen.localhos".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        })
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = cmd
        .login(LoginInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
            user_agent: "".to_owned(),
        })
        .await;

    assert_eq!(resp.unwrap().0, user);

    Ok(())
}
