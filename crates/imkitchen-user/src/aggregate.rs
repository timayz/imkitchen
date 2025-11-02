//! User aggregate

use crate::event::{
    ContactFormSubmitted, ContactMessageMarkedRead, ContactMessageResolved, EventMetadata,
    UserActivated, UserDemotedFromAdmin, UserLoggedIn, UserPremiumBypassToggled,
    UserProfileUpdated, UserPromotedToAdmin, UserRegistered, UserRegistrationFailed,
    UserRegistrationSucceeded, UserSuspended,
};
use bincode::{Decode, Encode};
use evento::EventDetails;

/// User aggregate representing a user in the system
#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct User {
    /// Registration status: pending, active, or failed
    pub status: Option<String>,
    /// Whether user is an admin
    pub is_admin: bool,
    /// Whether user is suspended (used for login validation)
    pub is_suspended: bool,
    /// Whether user has premium bypass (free tier with premium access)
    pub premium_bypass: bool,
}

#[evento::aggregator]
impl User {
    /// Handle user registration initiated event
    async fn user_registered(
        &mut self,
        event: EventDetails<UserRegistered, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("pending".to_string());
        self.is_admin = event.data.is_admin;
        Ok(())
    }

    /// Handle successful user registration
    async fn user_registration_succeeded(
        &mut self,
        event: EventDetails<UserRegistrationSucceeded, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("active".to_string());
        self.is_admin = event.data.is_admin;
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

    /// Handle user suspended event
    async fn user_suspended(
        &mut self,
        _event: EventDetails<UserSuspended, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.is_suspended = true;
        Ok(())
    }

    /// Handle user activated event
    async fn user_activated(
        &mut self,
        _event: EventDetails<UserActivated, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.is_suspended = false;
        Ok(())
    }

    /// Handle premium bypass toggled event
    async fn user_premium_bypass_toggled(
        &mut self,
        event: EventDetails<UserPremiumBypassToggled, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.premium_bypass = event.data.premium_bypass;
        Ok(())
    }

    /// Handle user promoted to admin
    async fn user_promoted_to_admin(
        &mut self,
        _event: EventDetails<UserPromotedToAdmin, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.is_admin = true;
        Ok(())
    }

    /// Handle user demoted from admin
    async fn user_demoted_from_admin(
        &mut self,
        _event: EventDetails<UserDemotedFromAdmin, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.is_admin = false;
        Ok(())
    }
}

/// ContactMessage aggregate representing a contact form submission
#[derive(Default, Encode, Decode, Clone, Debug)]
pub struct ContactMessage {
    /// Message status: new, read, or resolved
    pub status: Option<String>,
}

#[evento::aggregator]
impl ContactMessage {
    /// Handle contact form submission
    async fn contact_form_submitted(
        &mut self,
        _event: EventDetails<ContactFormSubmitted, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("new".to_string());
        Ok(())
    }

    /// Handle message marked as read
    async fn contact_message_marked_read(
        &mut self,
        _event: EventDetails<ContactMessageMarkedRead, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("read".to_string());
        Ok(())
    }

    /// Handle message resolved
    async fn contact_message_resolved(
        &mut self,
        _event: EventDetails<ContactMessageResolved, EventMetadata>,
    ) -> anyhow::Result<()> {
        self.status = Some("resolved".to_string());
        Ok(())
    }
}
