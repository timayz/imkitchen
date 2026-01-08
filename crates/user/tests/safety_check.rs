use imkitchen_shared::user::{LoggedIn, Logout, MadeAdmin, UsernameChanged};
use temp_dir::TempDir;
mod helpers;

#[tokio::test]
async fn test_safety_check() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;

    imkitchen_user::Command::register(
        &state.evento,
        &state.pool,
        &state.pool,
        imkitchen_user::RegisterInput {
            email: "john@imkitchen.test".to_owned(),
            password: "qwerty12345".to_owned(),
            lang: "fr".to_owned(),
            timezone: "Europe/Paris".to_owned(),
        },
    )
    .await?;

    let (id, access_id) = imkitchen_user::Command::login(
        &state.evento,
        &state.pool,
        imkitchen_user::LoginInput {
            email: "john@imkitchen.test".to_owned(),
            password: "qwerty12345".to_owned(),
            lang: "".to_owned(),
            timezone: "".to_owned(),
            user_agent: "".to_owned(),
        },
    )
    .await?;

    let user = imkitchen_user::load(&state.evento, &state.pool, &id)
        .await?
        .unwrap();

    user.set_username(&state.pool, &state.pool, "john_smith".to_owned())
        .await?;

    let user = imkitchen_user::load(&state.evento, &state.pool, &id)
        .await?
        .unwrap();

    user.made_admin().await?;
    let user = imkitchen_user::load(&state.evento, &state.pool, &id)
        .await?
        .unwrap();
    user.suspend("").await?;
    let user = imkitchen_user::load(&state.evento, &state.pool, &id)
        .await?
        .unwrap();
    user.activate("").await?;

    let subscription = imkitchen_user::subscription::load(&state.evento, &id).await?;
    subscription.toggle_life_premium("").await?;

    let password_id = imkitchen_user::password::Command::request(
        &state.evento,
        &state.pool,
        imkitchen_user::password::RequestInput {
            email: "john@imkitchen.test".to_owned(),
            lang: "fr".to_owned(),
            host: "https://imkitchen.test".to_owned(),
        },
    )
    .await?
    .unwrap();
    let password = imkitchen_user::password::load(&state.evento, &password_id)
        .await?
        .unwrap();

    password
        .reset(
            &state.pool,
            imkitchen_user::password::ResetInput {
                password: "my_new_password".to_owned(),
            },
        )
        .await?;

    let user = imkitchen_user::load(&state.evento, &state.pool, &id)
        .await?
        .unwrap();
    user.logout(access_id).await?;

    imkitchen_user::global_stat::subscription()
        .safety_check()
        .skip::<LoggedIn>()
        .skip::<Logout>()
        .skip::<MadeAdmin>()
        .skip::<UsernameChanged>()
        .data(state.pool.clone())
        .unretry_execute(&state.evento)
        .await?;

    Ok(())
}
