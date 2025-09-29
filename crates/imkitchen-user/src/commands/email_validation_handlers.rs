// Simple async command handlers for email validation

use sqlx::SqlitePool;
use validator::Validate;

use crate::commands::email_validation::{
    CheckEmailExistsCommand, CheckEmailExistsResponse, EmailValidationError,
    ValidateUsernameAvailabilityCommand, ValidateUsernameAvailabilityResponse,
};
use crate::queries::email_queries::EmailExistsQueryHandler;

/// Command handler service for email validation
pub struct EmailValidationService {
    query_handler: EmailExistsQueryHandler,
}

impl EmailValidationService {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            query_handler: EmailExistsQueryHandler::new(db_pool),
        }
    }

    /// Handle email existence check command
    pub async fn handle_email_exists_check(
        &self,
        command: CheckEmailExistsCommand,
    ) -> Result<CheckEmailExistsResponse, EmailValidationError> {
        // Validate the command first
        command.validate()?;

        // Use the query handler to check email existence
        self.query_handler
            .handle_email_exists_check(&command.email, command.request_id)
            .await
    }

    /// Handle username availability validation command
    pub async fn handle_username_availability_check(
        &self,
        command: ValidateUsernameAvailabilityCommand,
    ) -> Result<ValidateUsernameAvailabilityResponse, EmailValidationError> {
        // Validate the command first
        command.validate()?;

        // Use the query handler to check username availability
        self.query_handler
            .handle_username_availability_check(&command.username, command.request_id)
            .await
    }
}
