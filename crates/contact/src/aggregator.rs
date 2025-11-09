use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{FormSubmitted, MarkedAsReadAndReplay, Reopened, Resolved};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct Contact {
    pub status: String,
}

#[evento::aggregator]
impl Contact {
    async fn handle_form_submitted(&mut self, event: Event<FormSubmitted>) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_marked_as_read_and_replay(
        &mut self,
        event: Event<MarkedAsReadAndReplay>,
    ) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_resolved(&mut self, event: Event<Resolved>) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_reopened(&mut self, event: Event<Reopened>) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }
}
