use crate::types::user::{AdConsentGranted, AdConsentRevoked};
use evento::{Executor, ProjectionAggregate};

impl<E: Executor> super::Module<E> {
    pub async fn grant_ad_consent(&self, id: impl Into<String>) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.ad_consent {
            return Ok(());
        }

        user.write()?
            .event(&AdConsentGranted)
            .requested_by(&user.id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }

    pub async fn revoke_ad_consent(&self, id: impl Into<String>) -> imkitchen_core::Result<()> {
        let Some(user) = self.load(id).await? else {
            imkitchen_core::not_found!("user");
        };

        if !user.ad_consent {
            return Ok(());
        }

        user.write()?
            .event(&AdConsentRevoked)
            .requested_by(&user.id)
            .commit(&self.executor)
            .await?;

        Ok(())
    }
}
