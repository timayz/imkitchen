use evento::{Executor, LoadResult};
use imkitchen_shared::Metadata;
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{LifePremiumToggled, UserSubscription};

#[derive(Clone)]
pub struct Command<E: Executor + Clone>(pub E, pub SqlitePool);

impl<E: Executor + Clone> Command<E> {
    pub async fn load(
        &self,
        id: impl Into<String>,
    ) -> Result<LoadResult<UserSubscription>, evento::ReadError> {
        evento::load(&self.0, id).await
    }

    pub async fn load_optional(
        &self,
        id: impl Into<String>,
    ) -> Result<Option<LoadResult<UserSubscription>>, evento::ReadError> {
        evento::load_optional(&self.0, id).await
    }

    pub async fn toggle_life_premium(
        &self,
        id: impl Into<String>,
        metadata: &Metadata,
    ) -> imkitchen_shared::Result<u64> {
        let id = id.into();
        let mut expire_at = (SystemTime::now() + time::Duration::weeks(10 * 52))
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let builder = match self.load_optional(&id).await? {
            Some(subscription) => {
                if !subscription.item.expired {
                    expire_at = 0;
                }

                evento::save_with(subscription).data(&LifePremiumToggled { expire_at })?
            }
            _ => evento::create_with(id).data(&LifePremiumToggled { expire_at })?,
        };

        builder.metadata(metadata)?.commit(&self.0).await?;

        Ok(expire_at)
    }
}
