use evento::ProjectionAggregate;
use imkitchen_identity::LoginInput;
use imkitchen_identity::login::Login;
use imkitchen_identity::types::user::{AdConsentGranted, AdConsentRevoked};
use temp_dir::TempDir;

mod helpers;

#[test]
fn test_show_ads_predicate() {
    // free tier is ad-supported by default
    let free = Login::default();
    assert!(free.show_ads());

    let premium = Login {
        subscription_expire_at: u64::MAX,
        ..Default::default()
    };
    assert!(!premium.show_ads());

    // expired subscription falls back to the ad-supported free tier
    let expired_premium = Login {
        subscription_expire_at: 1,
        ..Default::default()
    };
    assert!(expired_premium.show_ads());

    // a stale consent timestamp from the retired opt-in flow has no effect
    let stale_consent = Login {
        ad_consent_at: 1,
        ..Default::default()
    };
    assert!(stale_consent.show_ads());
}

#[tokio::test]
async fn test_premium_removes_ads() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state.clone());
    let billing = imkitchen_billing::Billing::new(state);
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    cmd.login(LoginInput {
        email: "john.doe@imkitchen.localhost".to_owned(),
        password: "my_password".to_owned(),
        lang: "en".to_owned(),
        timezone: "UTC".to_owned(),
        user_agent: "test-agent".to_owned(),
    })
    .await?;

    let view = cmd.find_login(&user_id).await?.unwrap();
    let session = view
        .logins
        .iter()
        .find(|l| l.user_agent == "test-agent")
        .unwrap();
    assert!(session.show_ads());

    billing
        .subscription
        .toggle_life_premium(&user_id, "admin")
        .await?;

    let view = cmd.find_login(&user_id).await?.unwrap();
    let session = view
        .logins
        .iter()
        .find(|l| l.user_agent == "test-agent")
        .unwrap();
    assert!(session.is_premium());
    assert!(!session.show_ads());

    Ok(())
}

// The opt-in ad-consent flow is retired, but streams written before the
// removal still contain its events — strict projections must keep decoding
// them.
#[tokio::test]
async fn test_retired_ad_consent_events_still_replay() -> anyhow::Result<()> {
    let dir = TempDir::new()?;
    let path = dir.child("db.sqlite3");
    let state = helpers::setup_test_state(path).await?;
    let cmd = imkitchen_identity::Module::new(state.clone());
    let user_id = helpers::create_user(&cmd, "john.doe").await?;

    let user = cmd.load(&user_id).await?.unwrap();
    user.write()?
        .event(&AdConsentGranted)
        .requested_by(&user_id)
        .commit(&state.executor)
        .await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert!(user.ad_consent);
    let view = cmd.find_login(&user_id).await?.unwrap();
    assert!(view.ad_consent_at > 0);

    let user = cmd.load(&user_id).await?.unwrap();
    user.write()?
        .event(&AdConsentRevoked)
        .requested_by(&user_id)
        .commit(&state.executor)
        .await?;

    let user = cmd.load(&user_id).await?.unwrap();
    assert!(!user.ad_consent);
    let view = cmd.find_login(&user_id).await?.unwrap();
    assert_eq!(view.ad_consent_at, 0);

    Ok(())
}
