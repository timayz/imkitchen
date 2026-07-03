use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_change_email() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    assert_eq!(
        cmd.find_email(&user_id).await?.as_deref(),
        Some("john.doe@imkitchen.localhost")
    );

    cmd.change_email(
        &user_id,
        "new.email@imkitchen.localhost".to_owned(),
        "admin",
    )
    .await?;

    assert_eq!(
        cmd.find_email(&user_id).await?.as_deref(),
        Some("new.email@imkitchen.localhost")
    );

    // The old email no longer resolves to an account; the new one does.
    assert!(
        cmd.find_account("john.doe@imkitchen.localhost")
            .await?
            .is_none()
    );
    assert!(
        cmd.find_account("new.email@imkitchen.localhost")
            .await?
            .is_some()
    );

    Ok(())
}

#[tokio::test]
async fn test_change_email_same_is_noop() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    cmd.change_email(&user_id, "john.doe@imkitchen.localhost".to_owned(), "admin")
        .await?;

    assert_eq!(
        cmd.find_email(&user_id).await?.as_deref(),
        Some("john.doe@imkitchen.localhost")
    );

    Ok(())
}

#[tokio::test]
async fn test_change_email_rejects_duplicate() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let ids = helpers::create_users(&cmd, vec!["john.doe", "jane.doe"]).await?;
    let john = ids.first().unwrap();

    let err = cmd
        .change_email(john, "jane.doe@imkitchen.localhost".to_owned(), "admin")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Email already exists"));

    // John's email is unchanged.
    assert_eq!(
        cmd.find_email(john).await?.as_deref(),
        Some("john.doe@imkitchen.localhost")
    );

    Ok(())
}

#[tokio::test]
async fn test_change_email_rejects_invalid_format() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    assert!(
        cmd.change_email(&user_id, "not-an-email".to_owned(), "admin")
            .await
            .is_err()
    );

    assert_eq!(
        cmd.find_email(&user_id).await?.as_deref(),
        Some("john.doe@imkitchen.localhost")
    );

    Ok(())
}
