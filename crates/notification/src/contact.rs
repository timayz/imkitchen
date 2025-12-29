use evento::{Action, Executor, Projection, Snapshot, SubscriptionBuilder, metadata::Event};
use imkitchen_contact::FormSubmitted;

use crate::EmailService;

#[derive(Default, Clone)]
pub struct ContactView;

impl Snapshot for ContactView {}

pub fn create_projection<E: Executor>() -> Projection<ContactView, E> {
    Projection::new("notification-contact").handler(handle_form_submitted())
}

pub fn subscription<E: Executor>() -> SubscriptionBuilder<ContactView, E> {
    create_projection().no_safety_check().subscription()
}

#[evento::handler]
async fn handle_form_submitted<E: Executor>(
    event: Event<FormSubmitted>,
    action: Action<'_, ContactView, E>,
) -> anyhow::Result<()> {
    match action {
        Action::Apply(_data) => {}
        Action::Handle(context) => {
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
        }
    };

    Ok(())
}
