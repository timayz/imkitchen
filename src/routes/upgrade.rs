use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use stripe_billing::subscription::{
    CreateSubscription, CreateSubscriptionItems, CreateSubscriptionPaymentBehavior,
};
use stripe_core::customer::CreateCustomer;

use crate::{
    auth::AuthUser,
    routes::AppState,
    template::{Template, filters},
};

#[derive(askama::Template)]
#[template(path = "upgrade.html")]
pub struct UpgradeTemplate {
    pub current_path: String,
    pub user: AuthUser,
}

impl Default for UpgradeTemplate {
    fn default() -> Self {
        Self {
            current_path: "profile".to_owned(),
            user: Default::default(),
        }
    }
}

pub async fn page(template: Template, user: AuthUser, app: State<AppState>) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    template
        .render(UpgradeTemplate {
            user,
            ..Default::default()
        })
        .into_response()
}

pub async fn action(template: Template, user: AuthUser, app: State<AppState>) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    let user_info = crate::try_response!(anyhow_opt: app.user_query.admin(&user.id), template);

    let subscription =
        crate::try_response!(anyhow: app.user_cmd.subscription.load(&user.id), template);

    let customer_id = if let Some(id) = subscription.customer_id {
        id
    } else {
        let customer = crate::try_response!(anyhow:
            CreateCustomer::new()
            .email(user_info.email)
            .metadata([("imkitchen_user_id".to_owned(), user.id.to_owned())])
            .send(&app.stripe),
            template
        );

        crate::try_response!(
            app.user_cmd
                .subscription
                .create_stripe_customer(&customer.id, &user.id),
            template
        );
        customer.id.to_string()
    };

    let subscription = crate::try_response!(anyhow: CreateSubscription::new()
        .customer(&customer_id)
        .items(vec![CreateSubscriptionItems {
            price: Some("price_1T6YadRPUMbzRPUhnzWr30za".to_owned()),
            ..Default::default()
        }])
        .payment_behavior(CreateSubscriptionPaymentBehavior::DefaultIncomplete)
        .expand(["latest_invoice.confirmation_secret".to_owned()]).send(&app.stripe), template);

    let client_secret = subscription
        .latest_invoice
        .as_ref()
        .and_then(|inv| inv.as_object())
        .and_then(|inv| inv.confirmation_secret.clone())
        .unwrap()
        .client_secret;

    format!("<div ts-trigger=\"load\" ts-action=\"stripe-confirm-payment\" data-client-secret=\"{client_secret}\"></div>").into_response()
}
