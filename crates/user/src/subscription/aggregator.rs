use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{Decode, Encode};
use imkitchen_shared::Event;

use crate::subscription::LifePremiumToggled;

#[derive(Encode, Decode, Clone, Debug)]
pub struct UserSubscription {
    pub expire_at: u64,
    pub expired: bool,
}

impl Default for UserSubscription {
    fn default() -> Self {
        Self {
            expire_at: 0,
            expired: true,
        }
    }
}

#[evento::aggregator]
impl UserSubscription {
    async fn handle_premium_for_life_toggled(
        &mut self,
        event: Event<LifePremiumToggled>,
    ) -> anyhow::Result<()> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        self.expired = now > event.data.expire_at;
        self.expire_at = event.data.expire_at;

        Ok(())
    }
}
