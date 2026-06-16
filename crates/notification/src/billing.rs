use evento::{
    Executor,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_billing::types::invoice::Created;
use imkitchen_billing::types::subscription::Cancelled;
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::{
    EmailService, recipient,
    template::{Template, filters},
};

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("notification-billing")
        .handler(handle_invoice_created())
        .handler(handle_subscription_cancelled())
}

#[derive(askama::Template)]
#[template(path = "invoice-created.html")]
pub struct InvoiceCreatedHtmlTemplate {
    pub email: String,
    pub year: i32,
    pub invoice_number: String,
    pub plan: String,
    pub amount: String,
    pub invoice_url: String,
    pub lang: String,
}

#[derive(askama::Template)]
#[template(path = "invoice-created.txt")]
pub struct InvoiceCreatedPlainTemplate {
    pub email: String,
    pub year: i32,
    pub invoice_number: String,
    pub plan: String,
    pub amount: String,
    pub invoice_url: String,
    pub lang: String,
}

#[derive(askama::Template)]
#[template(path = "subscription-cancelled.html")]
pub struct SubscriptionCancelledHtmlTemplate {
    pub email: String,
    pub year: i32,
    pub lang: String,
}

#[derive(askama::Template)]
#[template(path = "subscription-cancelled.txt")]
pub struct SubscriptionCancelledPlainTemplate {
    pub email: String,
    pub year: i32,
    pub lang: String,
}

#[evento::subscription]
async fn handle_invoice_created<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Created>,
) -> anyhow::Result<()> {
    let service = context.extract::<EmailService>();
    let email = event.data.to.email.to_owned();
    let year = OffsetDateTime::from_unix_timestamp(event.timestamp.try_into()?)?.year();

    let user_id = event.metadata.requested_by()?;
    let (read_db, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    let lang = match recipient::load(context.executor, &read_db, &write_db, &user_id).await? {
        Some(r) => r.lang,
        None => "en".to_owned(),
    };

    let template = Template::new(&lang);

    let price = event.data.details.price + event.data.details.tax;
    let amount = format!("{}.{:02} EUR", price / 100, price % 100);
    let invoice_number = format!("{}-{:04}", event.data.key, event.data.number);
    let invoice_url = format!("{}/invoices/{}", service.app_url, event.aggregate_id);

    let html = template.to_string(InvoiceCreatedHtmlTemplate {
        email: email.to_owned(),
        lang: lang.to_owned(),
        invoice_number: invoice_number.to_owned(),
        plan: event.data.details.plan.to_owned(),
        amount: amount.to_owned(),
        invoice_url: invoice_url.to_owned(),
        year,
    });

    let subject = rust_i18n::t!("Your Invoice Is Ready", locale = lang).to_string();

    let plain = template.to_string(InvoiceCreatedPlainTemplate {
        email: email.to_owned(),
        lang,
        invoice_number,
        plan: event.data.details.plan,
        amount,
        invoice_url,
        year,
    });
    if let Err(err) = service.send(&email, subject, html, plain).await {
        tracing::warn!(error = ?err, "handle_invoice_created.send");
    }

    Ok(())
}

#[evento::subscription]
async fn handle_subscription_cancelled<E: Executor>(
    context: &Context<'_, E>,
    event: Event<Cancelled>,
) -> anyhow::Result<()> {
    let service = context.extract::<EmailService>();
    let year = OffsetDateTime::from_unix_timestamp(event.timestamp.try_into()?)?.year();

    let user_id = event.aggregate_id.to_owned();
    let (read_db, write_db) = context.extract::<(SqlitePool, SqlitePool)>();
    let recipient = match recipient::load(context.executor, &read_db, &write_db, &user_id).await? {
        Some(r) => r,
        None => {
            tracing::warn!(user_id = %user_id, "handle_subscription_cancelled: recipient not found");
            return Ok(());
        }
    };

    let template = Template::new(&recipient.lang);

    let html = template.to_string(SubscriptionCancelledHtmlTemplate {
        email: recipient.email.to_owned(),
        lang: recipient.lang.to_owned(),
        year,
    });

    let plain = template.to_string(SubscriptionCancelledPlainTemplate {
        email: recipient.email.to_owned(),
        lang: recipient.lang.to_owned(),
        year,
    });

    let subject = rust_i18n::t!("Subscription Cancelled", locale = &recipient.lang).to_string();
    if let Err(err) = service.send(&recipient.email, subject, html, plain).await {
        tracing::warn!(error = ?err, "handle_subscription_cancelled.send");
    }

    Ok(())
}
