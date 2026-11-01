use evento::{Executor, ProjectionAggregate};
use imkitchen_types::chef_follow::{Followed, Unfollowed};

impl<E: Executor + Clone> super::Module<E> {
    pub async fn follow(
        &self,
        chef_id: impl Into<String>,
        chef_name: impl Into<String>,
        user_id: impl Into<String>,
    ) -> crate::Result<()> {
        let chef_id = chef_id.into();
        let user_id = user_id.into();
        let follow = self.load(&chef_id, &user_id).await?;

        if !follow.following {
            follow
                .write()?
                .event(&Followed {
                    chef_id,
                    chef_name: chef_name.into(),
                })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}

impl<E: Executor + Clone> super::Module<E> {
    pub async fn unfollow(
        &self,
        chef_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> crate::Result<()> {
        let chef_id = chef_id.into();
        let user_id = user_id.into();
        let follow = self.load(&chef_id, &user_id).await?;

        if follow.following {
            follow
                .write()?
                .event(&Unfollowed { chef_id })
                .requested_by(user_id)
                .commit(&self.executor)
                .await?;
        }

        Ok(())
    }
}
