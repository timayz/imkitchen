use evento::{
    Executor, Snapshot,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_user::password::ResetRequested;
use time::OffsetDateTime;

use crate::{
    EmailService,
    template::{Template, filters},
};

#[derive(Default, Clone)]
pub struct ContactView;

impl Snapshot for ContactView {}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("notification-user").handler(handle_reset_requested())
}

#[derive(askama::Template)]
#[template(path = "reset-password.html")]
pub struct ResetPasswordHtmlTemplate {
    pub email: String,
    pub year: i32,
    pub reset_url: String,
    pub lang: String,
}

#[derive(askama::Template)]
#[template(path = "reset-password.txt")]
pub struct ResetPasswordPlainTemplate {
    pub email: String,
    pub year: i32,
    pub reset_url: String,
    pub lang: String,
}

#[evento::sub_handler]
async fn handle_reset_requested<E: Executor>(
    context: &Context<'_, E>,
    event: Event<ResetRequested>,
) -> anyhow::Result<()> {
    let service = context.extract::<EmailService>();
    let template = Template::new(&event.data.lang);
    let year = OffsetDateTime::from_unix_timestamp(event.timestamp.try_into()?)?.year();

    let reset_url = format!(
        "{}/reset-password/new/{}",
        event.data.host, event.aggregator_id
    );

    let html = template.to_string(ResetPasswordHtmlTemplate {
        email: event.data.email.to_owned(),
        lang: event.data.lang.to_owned(),
        reset_url: reset_url.to_owned(),
        year,
    });

    let plain = template.to_string(ResetPasswordPlainTemplate {
        email: event.data.email.to_owned(),
        lang: event.data.lang.to_owned(),
        reset_url,
        year,
    });

    let subject = rust_i18n::t!("", locales = event.data.lang).to_string();
    service.send(event.data.email, subject, html, plain).await?;

    Ok(())
}
