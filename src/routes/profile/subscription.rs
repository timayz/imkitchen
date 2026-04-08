// use axum::Form;
use axum::extract::State;
use axum::response::IntoResponse;
use axum_extra::extract::Form;
use serde::Deserialize;
use stripe_core::customer::RetrievePaymentMethodCustomer;
use stripe_core::payment_intent::RetrievePaymentIntent;
use stripe_core::setup_intent::CreateSetupIntent;
use stripe_core::setup_intent::CreateSetupIntentAutomaticPaymentMethods;
use stripe_core::setup_intent::CreateSetupIntentUsage;
use stripe_core::setup_intent::RetrieveSetupIntent;
use stripe_payment::payment_method::DetachPaymentMethod;

use crate::auth::AuthUser;
use crate::routes::AppState;
use crate::template;
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

#[tracing::instrument(skip_all, fields(user = user.id))]
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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn payment_method(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let mut subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    let Some(payment_method_id) = subscription.payment_method_id.to_owned() else {
        return "<div></div>".into_response();
    };

    if let Some(setup_intent_id) = subscription.setup_intent_id
        && let Ok(intent) = RetrieveSetupIntent::new(setup_intent_id)
            .expand(&["payment_method".to_owned()])
            .send(&app.stripe)
            .await
    {
        if let Err(e) = app
            .user_cmd
            .subscription
            .update_stripe_setup_intent_status(intent, &user.id)
            .await
        {
            tracing::error!("{e}");
        } else {
            crate::try_response!(anyhow:
                DetachPaymentMethod::new(payment_method_id).send(&app.stripe),
                template
            );

            subscription =
                crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);
        };
    };

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

#[derive(askama::Template)]
#[template(path = "partials/subscription-payment-update-modal.html")]
pub struct PaymentMethodModalTemplate {}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn update_payment_modal(template: Template, user: AuthUser) -> impl IntoResponse {
    template
        .render(PaymentMethodModalTemplate {})
        .into_response()
}

#[derive(Deserialize, Debug)]
pub struct UpdatePaymentInput {
    pub country: String,
    pub state: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn update_payment(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
    Form(input): Form<UpdatePaymentInput>,
) -> impl IntoResponse {
    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    let Some(customer_id) = subscription.customer_id else {
        tracing::error!("customer not found");

        return template.render(template::ServerTemplate).into_response();
    };

    let setup_intent = crate::try_response!(anyhow: CreateSetupIntent::new()
        .customer(customer_id)
        .metadata([("country".to_owned(), input.country), ("state".to_owned(), input.state)])
        .automatic_payment_methods(CreateSetupIntentAutomaticPaymentMethods::new(true))
        .usage(CreateSetupIntentUsage::OffSession)
        .send(&app.stripe), template);

    crate::try_response!(
        app.user_cmd
            .subscription
            .create_stripe_setup_intent(&setup_intent.id, &user.id),
        template
    );

    let client_secret = setup_intent.client_secret.unwrap_or_default();

    format!("<div ts-trigger=\"load\" ts-action=\"stripe-confirm-setup\" data-client-secret=\"{client_secret}\"></div>").into_response()
}

#[tracing::instrument(skip_all, fields(user = user.id))]
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
        .expand(&["payment_method".to_owned()])
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

#[tracing::instrument(skip_all, fields(user = user.id))]
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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn cancel(
    template: Template,
    State(app): State<AppState>,
    user: AuthUser,
) -> impl IntoResponse {
    let mut subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    crate::try_response!(app.user_cmd.subscription.cancel(&user.id), template);

    if let Some(id) = subscription.payment_method_id.to_owned() {
        crate::try_response!(anyhow:
            DetachPaymentMethod::new(id).send(&app.stripe),
            template
        );
    }

    subscription.is_active = false;

    template.render(CancelTemplate { subscription })
}
