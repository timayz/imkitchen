use evento::{Action, Executor, Projection, Snapshot, SubscriptionBuilder, metadata::Event};
use imkitchen_user::password::ResetRequested;
use time::OffsetDateTime;

use crate::{
    EmailService,
    template::{Template, filters},
};

#[derive(Default, Clone)]
pub struct ContactView;

impl Snapshot for ContactView {}

pub fn create_projection<E: Executor>() -> Projection<ContactView, E> {
    Projection::new("notification-user").handler(handle_reset_requested())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<ContactView, E> {
    create_projection().no_safety_check().subscription()
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

#[evento::handler]
async fn handle_reset_requested<E: Executor>(
    event: Event<ResetRequested>,
    action: Action<'_, ContactView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
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
        }
    };

    Ok(())
}
