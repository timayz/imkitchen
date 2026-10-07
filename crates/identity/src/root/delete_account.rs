use crate::types::user::Deleted;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use evento::{Executor, ProjectionAggregate};

use crate::repository;

impl<E: Executor> super::Module<E> {
    /// Re-authenticates the signed-in user before a destructive action
    /// (account deletion). A wrong password is a user error, a missing or
    /// deleted account is not found.
    pub async fn check_password(
        &self,
        id: impl Into<String>,
        password: &str,
    ) -> imkitchen_core::Result<()> {
        let Some(row) =
            repository::find(&self.read_db, repository::FindType::Id(id.into())).await?
        else {
            imkitchen_core::not_found!("user");
        };

        let parsed_hash = PasswordHash::new(&row.password)?;
        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_err()
        {
            imkitchen_core::user!("Incorrect password.");
        }

        Ok(())
    }

    /// Deletes the account: records [`Deleted`] (every projection drops the
    /// personal data it holds and all sessions end) and removes the
    /// credentials row so the email can be registered again. Callers verify
    /// the password with [`Self::check_password`] first and clean up the
    /// user's data in other modules before calling this, which is the point
    /// of no return.
    pub async fn delete_account(&self, id: impl Into<String>) -> imkitchen_core::Result<()> {
        let id = id.into();

        let Some(user) = self.load(&id).await? else {
            imkitchen_core::not_found!("user");
        };

        if user.deleted {
            imkitchen_core::not_found!("user");
        }

        user.write()?
            .event(&Deleted)
            .requested_by(&id)
            .commit(&self.executor)
            .await?;

        repository::delete(&self.write_db, id).await?;

        Ok(())
    }
}
