use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_shared::Event;
use imkitchen_user::reset_password::{ResetRequested, UserResetPassword};
use time::OffsetDateTime;

use crate::{
    EmailService,
    template::{Template, filters},
};

pub fn subscribe_user<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("notification-user")
        .handler(handle_reset_requested())
        .handler_check_off()
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

#[evento::handler(UserResetPassword)]
async fn handle_reset_requested<E: Executor>(
    context: &evento::Context<'_, E>,
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
