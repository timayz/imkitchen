use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::{FormSubmitted, MarkedReadAndReply, Reopened, Resolved, Status};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct Contact {
    pub status: Status,
}

#[evento::aggregator]
impl Contact {
    async fn handle_form_submitted(&mut self, event: Event<FormSubmitted>) -> anyhow::Result<()> {
        self.status = event.data.status;

        Ok(())
    }

    async fn handle_marked_read_and_reply(
        &mut self,
        event: Event<MarkedReadAndReply>,
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
