use imkitchen_identity::{LoginInput, RegisterInput};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_delete_account() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    let login = |ua: &str| LoginInput {
        email: "john.doe@imkitchen.localhost".to_owned(),
        password: "my_password".to_owned(),
        lang: "en".to_owned(),
        timezone: "UTC".to_owned(),
        user_agent: ua.to_owned(),
    };
    cmd.login(login("phone")).await?;
    cmd.login(login("laptop")).await?;
    assert_eq!(cmd.find_login(&user_id).await?.unwrap().logins.len(), 2);

    // Wrong password: a user error, nothing changes.
    assert!(
        cmd.check_password(&user_id, "not-my-password")
            .await
            .is_err()
    );
    cmd.check_password(&user_id, "my_password").await?;

    cmd.delete_account(&user_id).await?;

    // The aggregate remembers, the credentials row and every session are gone.
    assert!(cmd.load(&user_id).await?.unwrap().deleted);
    assert!(cmd.find_email(&user_id).await?.is_none());
    let view = cmd.find_login(&user_id).await?.unwrap();
    assert!(view.logins.is_empty());
    assert_eq!(view.email, "");

    // Signing in fails like an unknown address, and deleting twice is not found.
    assert!(cmd.login(login("phone")).await.is_err());
    assert!(cmd.delete_account(&user_id).await.is_err());

    // The email is free again, under a new id.
    let again = cmd
        .register(RegisterInput {
            email: "john.doe@imkitchen.localhost".to_owned(),
            password: "my_password".to_owned(),
            lang: "en".to_owned(),
            timezone: "UTC".to_owned(),
        })
        .await?;
    assert_ne!(again, user_id);

    Ok(())
}
