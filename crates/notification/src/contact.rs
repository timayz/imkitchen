use evento::{
    Executor, Snapshot,
    metadata::Event,
    subscription::{Context, SubscriptionBuilder},
};
use imkitchen_contact::FormSubmitted;

use crate::EmailService;

#[derive(Default, Clone)]
pub struct ContactView;

impl Snapshot for ContactView {}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<E> {
    SubscriptionBuilder::new("notification-contact").handler(handle_form_submitted())
}

#[evento::sub_handler]
async fn handle_form_submitted<E: Executor>(
    context: &Context<'_, E>,
    event: Event<FormSubmitted>,
) -> anyhow::Result<()> {
    let service = context.extract::<EmailService>();
    service
        .send_plain(
            &event.data.to,
            event.data.subject.to_string(),
            format!(
                r#"
{} <{}>,

{}
            "#,
                event.data.name, event.data.email, event.data.message
            ),
        )
        .await?;

    Ok(())
}
