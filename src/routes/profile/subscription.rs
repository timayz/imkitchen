// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use stripe_core::customer::RetrievePaymentMethodCustomer;
use stripe_core::payment_intent::RetrievePaymentIntent;

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

#[derive(askama::Template)]
#[template(path = "partials/subscription-payment.html")]
pub struct PaymentMethodTemplate {
    pub payment_method: stripe_shared::PaymentMethod,
}

pub async fn payment_method(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    let (Some(payment_method_id), Some(customer_id)) =
        (subscription.payment_method_id, subscription.customer_id)
    else {
        return "<div></div>".into_response();
    };

    let payment_mehod = crate::try_response!(anyhow:
        RetrievePaymentMethodCustomer::new(customer_id, payment_method_id).send(&app.stripe), 
        template);

    template
        .render(PaymentMethodTemplate {
            payment_method: payment_mehod,
        })
        .into_response()
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

#[derive(askama::Template)]
#[template(path = "partials/subscription-cancel-modal.html")]
struct CancelModalTemplate {
    pub subscription: imkitchen_user::subscription::Subscription,
}

pub async fn cancel_modal(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    template.render(CancelModalTemplate { subscription })
}

#[derive(askama::Template)]
#[template(path = "partials/subscription-cancel.html")]
struct CancelTemplate {
    pub subscription: imkitchen_user::subscription::Subscription,
}

pub async fn cancel(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let mut subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    crate::try_response!(app.user_cmd.subscription.cancel(&user.id), template);

    subscription.is_active = false;

    template.render(CancelTemplate { subscription })
}
