use bincode::{Decode, Encode};

use crate::{LoggedIn, Registered, UserEvent};

#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {}

#[evento::aggregator]
impl User {
    async fn handle_registered(&mut self, _event: UserEvent<Registered>) -> anyhow::Result<()> {
        Ok(())
    }

    async fn handle_logged_in(&mut self, _event: UserEvent<LoggedIn>) -> anyhow::Result<()> {
        Ok(())
    }
}
