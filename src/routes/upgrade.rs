use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::Form;
use imkitchen_shared::user::subscription::PaymentDetails;
use serde::Deserialize;
use stripe_core::{
    PaymentIntentSetupFutureUsage,
    customer::CreateCustomer,
    payment_intent::{CreatePaymentIntent, CreatePaymentIntentAutomaticPaymentMethods},
};
use stripe_types::Currency;
use world_tax::{Region, TaxDatabase, TaxScenario, TaxType};

use crate::{
    auth::AuthUser,
    config::PremiumConfig,
    routes::AppState,
    template::{self, Template, filters},
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
            current_path: "settings".to_owned(),
            user: Default::default(),
        }
    }
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn page(template: Template, user: AuthUser) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/settings/billing").into_response();
    }

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
    pub amount: u32,
    pub country: String,
    pub state: String,
}

#[tracing::instrument(skip_all, fields(user = user.id))]
pub async fn action(
    template: Template,
    user: AuthUser,
    app: State<AppState>,
    Form(input): Form<ActionInput>,
) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/settings/billing").into_response();
    }

    let Some(ref premium) = app.config.premium else {
        tracing::error!("premium not configured");

        return template.render(template::ServerTemplate).into_response();
    };

    let tax_db = crate::try_response!(sync anyhow: TaxDatabase::new(), template);
    let price_tax = crate::try_response!(sync anyhow:
        get_price_with_tax(&tax_db, premium.clone(), input.plan.to_owned(), input.country.to_owned(), input.state.to_owned()),
        template
    );

    let amount = price_tax.price + price_tax.tax;

    if amount != input.amount {
        return template.render(template::ServerTemplate).into_response();
    }

    let user_info = crate::try_response!(anyhow_opt: app.identity_query.admin(&user.id), template);

    let subscription =
        crate::try_response!(anyhow: app.billing_subscription_cmd.load(&user.id), template);

    let customer_id = if let Some(id) = subscription.customer_id {
        id
    } else {
        let customer = crate::try_response!(anyhow:
            CreateCustomer::new()
            .email(&user_info.email)
            .metadata([("imkitchen_user_id".to_owned(), user.id.to_owned())])
            .send(&app.stripe),
            template
        );

        crate::try_response!(
            app.billing_subscription_cmd
                .create_stripe_customer(&customer.id, &user.id),
            template
        );

        customer.id.to_string()
    };

    let payment_details = PaymentDetails {
        plan: input.plan.to_owned(),
        price: price_tax.price,
        tax: price_tax.tax,
        tax_rate: price_tax.tax_type.map(|(rate, _)| rate),
    };

    let payment_intent = crate::try_response!(anyhow: CreatePaymentIntent::new(amount, Currency::USD)
        .customer(customer_id)
        .automatic_payment_methods(CreatePaymentIntentAutomaticPaymentMethods::new(true))
        .setup_future_usage(PaymentIntentSetupFutureUsage::OffSession)
        .send(&app.stripe), template);

    crate::try_response!(
        app.billing_subscription_cmd.create_stripe_payment_intent(
            &payment_intent.id,
            &user_info.email,
            &user.id,
            payment_details
        ),
        template
    );

    let client_secret = payment_intent.client_secret.unwrap_or_default();

    format!("<div ts-trigger=\"load\" ts-action=\"stripe-confirm-payment\" data-client-secret=\"{client_secret}\"></div>").into_response()
}

#[derive(askama::Template)]
#[template(path = "partials/upgrade-order-summary.html")]
pub struct UpgradeOrderSummaryTemplate {
    pub plan: String,
    pub amount: u32,
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
    State(app): State<AppState>,
    Form(input): Form<OrderSummary>,
) -> impl IntoResponse {
    if user.is_premium() {
        return Redirect::to("/settings/billing").into_response();
    }

    let Some(premium) = app.config.premium else {
        tracing::error!("premium not configured");

        return template.render(template::ServerTemplate).into_response();
    };

    let tax_db = crate::try_response!(sync anyhow: TaxDatabase::new(), template);
    let price_tax = crate::try_response!(sync anyhow:
        get_price_with_tax(&tax_db, premium, input.plan.to_owned(), input.country, input.state),
        template
    );

    let tax_label = if let Some((rate, TaxType::VAT(_))) = price_tax.tax_type {
        format!("VAT ({}%)", rate * 100.0)
    } else {
        "Tax".to_owned()
    };
    let mut chars = input.plan.chars();
    let plan = match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    };

    template
        .render(UpgradeOrderSummaryTemplate {
            plan,
            price: price_tax.price as f64 / 100.0,
            amount: price_tax.price + price_tax.tax,
            tax: price_tax.tax as f64 / 100.0,
            tax_label,
        })
        .into_response()
}

struct PriceTax {
    price: u32,
    tax: u32,
    tax_type: Option<(f64, TaxType)>,
}

fn get_price_with_tax(
    db: &TaxDatabase,
    config: PremiumConfig,
    plan: String,
    country: String,
    state: String,
) -> anyhow::Result<PriceTax> {
    let region = if !state.is_empty() {
        if db.get_country(&state).is_ok() {
            Some(format!("{}-{}", country, state))
        } else {
            None
        }
    } else {
        None
    };
    let mut scenario = TaxScenario::new(
        Region::new("FR".to_owned(), None)?,
        Region::new(country, region)?,
        world_tax::TransactionType::B2C,
    );

    scenario.is_digital_product_or_service = true;

    let price = match (plan.as_str(), &config) {
        ("monthly", premium) => premium.monthly_price as u32,
        ("annual", premium) if premium.annual_rate < 100 => premium.annual_price(),
        _ => anyhow::bail!("bad request"),
    };

    let rates = scenario.get_rates(price.into(), db).unwrap_or_default();

    Ok(PriceTax {
        price,
        tax: (scenario
            .calculate_tax(price as f64 / 100.0, db)
            .unwrap_or(0.0)
            * 100.0) as u32,
        tax_type: rates.first().map(|r| (r.rate, r.tax_type.clone())),
    })
}
