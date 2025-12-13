use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::reset_password::{ResetRequested, Resetted};

#[derive(Encode, Decode, Clone, Debug, Default)]
pub struct UserResetPassword {
    pub user_id: String,
    pub resetted: bool,
}

#[evento::aggregator]
impl UserResetPassword {
    async fn handle_reset_requested(&mut self, event: Event<ResetRequested>) -> anyhow::Result<()> {
        self.user_id = event.metadata.trigger_by()?;

        Ok(())
    }

    async fn handle_resetted(&mut self, _event: Event<Resetted>) -> anyhow::Result<()> {
        self.resetted = true;

        Ok(())
    }
}
