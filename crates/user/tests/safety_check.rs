use imkitchen_shared::user::{LoggedIn, Logout, MadeAdmin, UsernameChanged};
use temp_dir::TempDir;
mod helpers;

#[tokio::test]
async fn test_safety_check() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_user::Command::new(state.clone());

    cmd.register(imkitchen_user::RegisterInput {
        email: "john@imkitchen.test".to_owned(),
        password: "qwerty12345".to_owned(),
        lang: "fr".to_owned(),
        timezone: "Europe/Paris".to_owned(),
    })
    .await?;

    let (id, access_id) = cmd
        .login(imkitchen_user::LoginInput {
            email: "john@imkitchen.test".to_owned(),
            password: "qwerty12345".to_owned(),
            lang: "".to_owned(),
            timezone: "".to_owned(),
            user_agent: "".to_owned(),
        })
        .await?;

    cmd.set_username(&id, "john_smith".to_owned()).await?;
    cmd.made_admin(&id).await?;
    cmd.suspend(&id, "").await?;
    cmd.activate(&id, "").await?;

    cmd.subscription.toggle_life_premium(&id, "").await?;

    let password_id = cmd
        .password
        .request(imkitchen_user::password::RequestInput {
            email: "john@imkitchen.test".to_owned(),
            lang: "fr".to_owned(),
            host: "https://imkitchen.test".to_owned(),
        })
        .await?
        .unwrap();

    cmd.password
        .reset(imkitchen_user::password::ResetInput {
            id: password_id,
            password: "my_new_password".to_owned(),
        })
        .await?;

    cmd.logout(&id, access_id).await?;

    imkitchen_user::global_stat::subscription()
        .safety_check()
        .skip::<LoggedIn>()
        .skip::<Logout>()
        .skip::<MadeAdmin>()
        .skip::<UsernameChanged>()
        .data(state.write_db.clone())
        .unretry_execute(&state.executor)
        .await?;

    Ok(())
}
