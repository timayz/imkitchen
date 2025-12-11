use imkitchen_shared::Metadata;
use imkitchen_user::{LoginInput, subscribe_command};
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_login_failure() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let command = imkitchen_user::Command {
        evento: state.evento.clone(),
        read_db: state.pool.clone(),
        write_db: state.pool.clone(),
    };
    let metadata = Metadata::default();
    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkitchen.localhost".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
                user_agent: "".to_owned(),
            },
            &metadata,
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
                email: "john.doe@imkitchen.localhost".to_owned(),
                password: "my_password3".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
                user_agent: "".to_owned(),
            },
            &metadata,
        )
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkitchen.localhos".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
                user_agent: "".to_owned(),
            },
            &metadata,
        )
        .await;

    assert_eq!(
        resp.unwrap_err().to_string(),
        "Invalid email or password. Please try again."
    );

    let resp = command
        .login(
            LoginInput {
                email: "john.doe@imkitchen.localhost".to_owned(),
                password: "my_password".to_owned(),
                lang: "en".to_owned(),
                timezone: "UTC".to_owned(),
                user_agent: "".to_owned(),
            },
            &metadata,
        )
        .await;

    assert_eq!(resp.unwrap().user_id, user);

    subscribe_command()
        .data(state.pool.clone())
        .unretry_oneshot(&state.evento)
        .await?;

    Ok(())
}
