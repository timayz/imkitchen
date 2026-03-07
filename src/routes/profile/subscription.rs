// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use stripe_core::payment_intent::RetrievePaymentIntent;
// use serde::Deserialize;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template::Template;
use crate::template::filters;

#[derive(askama::Template)]
#[template(path = "profile-subscription.html")]
pub struct SubscriptionTemplate {
    pub current_path: String,
    pub profile_path: String,
    pub subscription: imkitchen_user::subscription::Subscription,
    pub user: AuthUser,
}

pub async fn page(
    template: Template,
    user: AuthUser,
    State(app): State<AppState>,
) -> impl IntoResponse {
    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    template.render(SubscriptionTemplate {
        current_path: "profile".to_owned(),
        profile_path: "subscription".to_owned(),
        subscription,
        user,
    })
}

pub async fn check(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    let Some(payment_intent_id) = subscription.payment_intent_id else {
        return "<div></div>".into_response();
    };

    let Ok(intent) = RetrievePaymentIntent::new(payment_intent_id)
        .send(&app.stripe)
        .await
    else {
        return "<div></div>".into_response();
    };

    if let Err(e) = app
        .user_cmd
        .subscription
        .update_stripe_payment_intent_status(intent, &user.id)
        .await
    {
        tracing::error!("{e}");
    }

    "<div></div>".into_response()
}
