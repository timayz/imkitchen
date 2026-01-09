use evento::{Executor, ProjectionAggregator, metadata::Metadata};
use time::UtcDateTime;

use imkitchen_shared::user::subscription::LifePremiumToggled;

impl<E: Executor> super::Command<E> {
    pub async fn toggle_life_premium(
        &self,
        id: impl Into<String>,
        request_by: impl Into<String>,
    ) -> imkitchen_shared::Result<()> {
        let subscription = self.load(id).await?;
        let now = UtcDateTime::now();
        let expire_at = if subscription.expire_at > now.unix_timestamp().try_into()? {
            0
        } else {
            (now + time::Duration::weeks(10 * 53)).unix_timestamp()
        };

        subscription
            .aggregator()?
            .event(&LifePremiumToggled {
                expire_at: expire_at.try_into()?,
            })
            .metadata(&Metadata::new(request_by))
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
