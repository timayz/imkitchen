// UserAccountDeleted event for account deletion workflow with audit trail

use chrono::{DateTime, Utc};
use imkitchen_shared::Email;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::event_store::DomainEvent;

/// Event fired when a user account is deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountDeleted {
    pub user_id: Uuid,
    pub email: Email,
    pub deleted_at: DateTime<Utc>,
    pub deletion_reason: DeletionReason,

    /// IP address from which deletion was requested (for security audit)
    pub deletion_ip: Option<String>,

    /// User agent string (for security audit)
    pub user_agent: Option<String>,

    /// Whether user explicitly requested deletion vs admin/system deletion
    pub initiated_by_user: bool,

    /// Data retention compliance - when personal data will be permanently purged
    pub data_purge_scheduled_at: DateTime<Utc>,

    /// Original registration date for audit purposes
    pub account_created_at: DateTime<Utc>,

    /// Account age in days at deletion
    pub account_age_days: i64,
}

/// Enumeration of possible deletion reasons for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionReason {
    /// User requested deletion through account settings
    UserRequested,
    /// GDPR right to be forgotten request
    GdprRequest,
    /// Account inactive for extended period
    InactiveAccount,
    /// Terms of service violation
    TosViolation,
    /// Suspected fraudulent or spam account
    Fraud,
    /// Administrative deletion
    AdminAction,
    /// System cleanup (e.g., test accounts)
    SystemCleanup,
}

impl UserAccountDeleted {
    /// Create a new UserAccountDeleted event
    pub fn new(
        user_id: Uuid,
        email: Email,
        deletion_reason: DeletionReason,
        initiated_by_user: bool,
        account_created_at: DateTime<Utc>,
        deletion_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let deleted_at = Utc::now();
        let account_age_days = (deleted_at - account_created_at).num_days();

        // Default data purge to 30 days after deletion (GDPR compliance)
        let data_purge_scheduled_at = deleted_at + chrono::Duration::days(30);

        Self {
            user_id,
            email,
            deleted_at,
            deletion_reason,
            deletion_ip,
            user_agent,
            initiated_by_user,
            data_purge_scheduled_at,
            account_created_at,
            account_age_days,
        }
    }

    /// Create a user-initiated deletion event
    pub fn user_requested(
        user_id: Uuid,
        email: Email,
        account_created_at: DateTime<Utc>,
        deletion_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self::new(
            user_id,
            email,
            DeletionReason::UserRequested,
            true,
            account_created_at,
            deletion_ip,
            user_agent,
        )
    }

    /// Create a GDPR right-to-be-forgotten deletion event
    pub fn gdpr_request(user_id: Uuid, email: Email, account_created_at: DateTime<Utc>) -> Self {
        Self::new(
            user_id,
            email,
            DeletionReason::GdprRequest,
            true,
            account_created_at,
            None,
            None,
        )
    }

    /// Create an admin-initiated deletion event
    pub fn admin_action(
        user_id: Uuid,
        email: Email,
        account_created_at: DateTime<Utc>,
        deletion_reason: DeletionReason,
    ) -> Self {
        Self::new(
            user_id,
            email,
            deletion_reason,
            false,
            account_created_at,
            None,
            None,
        )
    }

    /// Check if this deletion requires immediate data purge
    pub fn requires_immediate_purge(&self) -> bool {
        matches!(self.deletion_reason, DeletionReason::GdprRequest)
    }

    /// Calculate days remaining until data purge
    pub fn days_until_purge(&self) -> i64 {
        let now = Utc::now();
        (self.data_purge_scheduled_at - now).num_days()
    }

    /// Check if data should be purged now
    pub fn should_purge_data(&self) -> bool {
        Utc::now() >= self.data_purge_scheduled_at
    }
}

impl DeletionReason {
    /// Get human-readable description of deletion reason
    pub fn description(&self) -> &'static str {
        match self {
            DeletionReason::UserRequested => "User requested account deletion",
            DeletionReason::GdprRequest => "GDPR right to be forgotten request",
            DeletionReason::InactiveAccount => "Account inactive for extended period",
            DeletionReason::TosViolation => "Terms of service violation",
            DeletionReason::Fraud => "Suspected fraudulent or spam account",
            DeletionReason::AdminAction => "Administrative action",
            DeletionReason::SystemCleanup => "System maintenance cleanup",
        }
    }

    /// Check if this deletion type requires audit logging
    pub fn requires_audit_log(&self) -> bool {
        matches!(
            self,
            DeletionReason::TosViolation | DeletionReason::Fraud | DeletionReason::AdminAction
        )
    }
}

impl DomainEvent for UserAccountDeleted {
    fn event_type(&self) -> &'static str {
        "UserAccountDeleted"
    }

    fn aggregate_id(&self) -> Uuid {
        self.user_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.deleted_at
    }
}
