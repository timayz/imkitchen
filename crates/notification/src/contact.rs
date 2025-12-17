use evento::{AggregatorName, Executor, SubscribeBuilder};
use imkitchen_contact::{Contact, FormSubmitted};
use imkitchen_shared::Event;

use crate::EmailService;

pub fn subscribe_contact<E: Executor + Clone>() -> SubscribeBuilder<E> {
    evento::subscribe("notification-contact")
        .handler(handle_form_submitted())
        .handler_check_off()
}

#[evento::handler(Contact)]
async fn handle_form_submitted<E: Executor>(
    context: &evento::Context<'_, E>,
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
