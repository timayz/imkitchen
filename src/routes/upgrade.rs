use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use serde::Deserialize;
use stripe_billing::subscription::{
    CreateSubscription, CreateSubscriptionItems, CreateSubscriptionPaymentBehavior,
};
use stripe_core::customer::CreateCustomer;
use world_tax::{Region, TaxDatabase, TaxScenario, TaxType};

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

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    // let region = if let Some((_, region)) = template.preferred_language.split_once("-") {
    //     region.to_uppercase()
    // } else {
    //     "US".to_owned()
    // };
    //
    // let tax = try_page_response!(sync: TaxDatabase::new(), template);
    // let mut scenario = TaxScenario::new(
    //     crate::try_page_response!(sync: Region::new("FR".to_owned(), None), template),
    //     crate::try_page_response!(sync: Region::new(region, None), template),
    //     world_tax::TransactionType::B2C,
    // );
    //
    // scenario.is_digital_product_or_service = true;
    //
    // let tax = crate::try_page_response!(sync: scenario.calculate_tax(47.90, &tax), template);
    // println!("{tax}");

    template
        .render(UpgradeTemplate {
            user,
            ..Default::default()
        })
        .into_response()
}

#[derive(Deserialize, Debug)]
pub struct ActionInput {
    pub plan: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn action(
    template: Template,
    user: AuthUser,
    app: State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    let price_id = match input.plan.as_str() {
        "monthly" => app.config.stripe.monthly_price_id.to_owned(),
        "annual" => app.config.stripe.annual_price_id.to_owned(),
        _ => return (StatusCode::BAD_REQUEST, "").into_response(),
    };

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
            price: Some(price_id),
            ..Default::default()
        }])
        .payment_behavior(CreateSubscriptionPaymentBehavior::DefaultIncomplete)
        .expand(["latest_invoice.confirmation_secret".to_owned()]).send(&app.stripe), template);

    crate::try_response!(
        app.user_cmd
            .subscription
            .create_stripe_subscription(&subscription.id, &user.id),
        template
    );

    let client_secret = subscription
        .latest_invoice
        .as_ref()
        .and_then(|inv| inv.as_object())
        .and_then(|inv| inv.confirmation_secret.clone())
        .unwrap()
        .client_secret;

    format!("<div ts-trigger=\"load\" ts-action=\"stripe-confirm-payment\" data-client-secret=\"{client_secret}\"></div>").into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/upgrade-order-summary.html")]
pub struct UpgradeOrderSummaryTemplate {
    pub plan: String,
    pub price: f64,
    pub tax: f64,
    pub tax_label: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderSummary {
    pub plan: String,
    pub country: String,
    pub state: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn order_summary(
    template: Template,
    user: AuthUser,
    Form(input): Form<OrderSummary>,
) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/profile/subscription").into_response();
    }

    let tax = crate::try_response!(sync anyhow: TaxDatabase::new(), template);

    let country = match input.country.as_str() {
        "MQ" => "FR".to_owned(),
        "GP" => "FR".to_owned(),
        "RE" => "FR".to_owned(),
        _ => input.country,
    };

    let region = if !input.state.is_empty() {
        if tax.get_country(&input.state).is_ok() {
            Some(format!("{}-{}", country, input.state))
        } else {
            None
        }
    } else {
        None
    };

    let mut scenario = TaxScenario::new(
        crate::try_response!(sync anyhow: Region::new("FR".to_owned(), None), template),
        crate::try_response!(sync anyhow: Region::new(country, region), template),
        world_tax::TransactionType::B2C,
    );

    scenario.is_digital_product_or_service = true;

    let price = match input.plan.as_str() {
        "monthly" => 4.99,
        "annual" => 47.90,
        _ => return (StatusCode::BAD_REQUEST, "").into_response(),
    };

    let rates = scenario.get_rates(price, &tax).unwrap_or_default();
    let tax_label = if let Some((rate, TaxType::VAT(_))) =
        rates.first().map(|r| (r.rate, r.tax_type.clone()))
    {
        format!("VAT ({}%)", rate * 100.0)
    } else {
        "Tax".to_owned()
    };
    let tax = scenario.calculate_tax(price, &tax).unwrap_or(0.0);
    let mut chars = input.plan.chars();
    let plan = match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    };

    template
        .render(UpgradeOrderSummaryTemplate {
            plan,
            price,
            tax,
            tax_label,
        })
        .into_response()
}
