use imkitchen_user::RegisterInput;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn validate_unique_emails() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state);
    cmd.register(RegisterInput {
        email: "john.doe@imkitchen.localhost".to_owned(),
        password: "my_password".to_owned(),
        lang: "en".to_owned(),
        timezone: "UTC".to_owned(),
    })
    .await?;
    let user_2 = cmd
        .register(RegisterInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password_v2".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
        })
        .await;

    assert_eq!(
        user_2.unwrap_err().to_string(),
        "Email already exists".to_owned()
    );

    Ok(())
}
