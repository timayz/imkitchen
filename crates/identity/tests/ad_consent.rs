use imkitchen_identity::LoginInput;
use imkitchen_identity::login::Login;
use temp_dir::TempDir;

mod helpers;

#[tokio::test]
async fn test_ad_consent() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert!(!user.ad_consent);

    let view = cmd.find_login(&user_id).await?.unwrap();
    assert_eq!(view.ad_consent_at, 0);

    cmd.grant_ad_consent(&user_id).await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert!(user.ad_consent);

    let view = cmd.find_login(&user_id).await?.unwrap();
    assert!(view.ad_consent_at > 0);

    // granting again is a no-op
    cmd.grant_ad_consent(&user_id).await?;
    let user = cmd.load(&user_id).await?.unwrap();
    assert!(user.ad_consent);

    cmd.revoke_ad_consent(&user_id).await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert!(!user.ad_consent);

    let view = cmd.find_login(&user_id).await?.unwrap();
    assert_eq!(view.ad_consent_at, 0);

    // revoking again is a no-op
    cmd.revoke_ad_consent(&user_id).await?;

    Ok(())
}

#[tokio::test]
async fn test_premium_overrides_ad_consent() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state.clone());
    let billing = imkitchen_billing::Billing::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    // real session path: log in first, then accept ads, then become premium —
    // the UI reads the per-session entry in logins[], not the view fields
    cmd.login(LoginInput {
        email: "john.doe@imkitchen.localhost".to_owned(),
        password: "my_password".to_owned(),
        lang: "en".to_owned(),
        timezone: "UTC".to_owned(),
        user_agent: "test-agent".to_owned(),
    })
    .await?;

    cmd.grant_ad_consent(&user_id).await?;

    let view = cmd.find_login(&user_id).await?.unwrap();
    let session = view
        .logins
        .iter()
        .find(|l| l.user_agent == "test-agent")
        .unwrap();
    assert!(session.has_ad_consent());
    assert!(session.show_ads());

    billing
        .subscription
        .toggle_life_premium(&user_id, "admin")
        .await?;

    let view = cmd.find_login(&user_id).await?.unwrap();
    assert!(view.ad_consent_at > 0);

    let session = view
        .logins
        .iter()
        .find(|l| l.user_agent == "test-agent")
        .unwrap();
    assert!(session.is_premium());
    assert!(session.has_full_access());
    assert!(!session.show_ads());

    let login = Login {
        subscription_expire_at: view.subscription_expire_at,
        ad_consent_at: view.ad_consent_at,
        ..Default::default()
    };
    assert!(login.is_premium());
    assert!(login.has_full_access());
    assert!(!login.show_ads());

    Ok(())
}

#[test]
fn test_access_predicates() {
    let far_future = u64::MAX;

    let free = Login::default();
    assert!(!free.has_ad_consent());
    assert!(!free.has_full_access());
    assert!(!free.show_ads());

    let consented = Login {
        ad_consent_at: 1,
        ..Default::default()
    };
    assert!(consented.has_ad_consent());
    assert!(consented.has_full_access());
    assert!(consented.show_ads());

    let premium = Login {
        subscription_expire_at: far_future,
        ..Default::default()
    };
    assert!(premium.has_full_access());
    assert!(!premium.show_ads());

    // premium wins over consent: full access, no ads
    let premium_and_consented = Login {
        subscription_expire_at: far_future,
        ad_consent_at: 1,
        ..Default::default()
    };
    assert!(premium_and_consented.has_full_access());
    assert!(!premium_and_consented.show_ads());

    // expired subscription with consent falls back to ad-supported access
    let expired_premium_consented = Login {
        subscription_expire_at: 1,
        ad_consent_at: 1,
        ..Default::default()
    };
    assert!(expired_premium_consented.has_full_access());
    assert!(expired_premium_consented.show_ads());
}
