//! User aggregate

use crate::event::{
    EventMetadata, UserLoggedIn, UserProfileUpdated, UserRegistered, UserRegistrationFailed,
    UserRegistrationSucceeded,
};
use bincode::{Decode, Encode};
use evento::EventDetails;

/// User aggregate representing a user in the system
#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {
    /// Registration status: pending, active, or failed
    pub status: Option<String>,
    /// Whether user is suspended (used for login validation)
    pub is_suspended: bool,
}

#[evento::aggregator]
impl User {
    /// Handle user registration initiated event
    async fn user_registered(
        &mut self,
        _event: EventDetails<UserRegistered, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("pending".to_string());
        Ok(())
    }

    /// Handle successful user registration
    async fn user_registration_succeeded(
        &mut self,
        _event: EventDetails<UserRegistrationSucceeded, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("active".to_string());
        Ok(())
    }

    /// Handle failed user registration
    async fn user_registration_failed(
        &mut self,
        _event: EventDetails<UserRegistrationFailed, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("failed".to_string());
        Ok(())
    }

    /// Handle user login event
    async fn user_logged_in(
        &mut self,
        _event: EventDetails<UserLoggedIn, EventMetadata>,
    ) -> anyhow::Result<()> {
        // Login event processed, timestamp tracked automatically by Evento
        Ok(())
    }

    /// Handle user profile update event
    async fn user_profile_updated(
        &mut self,
        _event: EventDetails<UserProfileUpdated, EventMetadata>,
    ) -> anyhow::Result<()> {
        // Profile update processed, data stored in projection
        Ok(())
    }
}
