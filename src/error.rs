use std::fmt;
use thiserror::Error;
use tracing::{error, warn};

/// Comprehensive error types for IMKitchen application
#[derive(Error, Debug)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum AppError {
    /// Configuration related errors
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },

    /// Database related errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<sqlx::Error>,
        query: Option<String>,
        correlation_id: Option<String>,
    },

    /// CLI argument parsing errors
    #[error("Command line error: {message}")]
    CommandLine {
        message: String,
        command: Option<String>,
        suggestion: Option<String>,
        correlation_id: Option<String>,
    },

    /// File system operations errors
    #[error("File system error: {message}")]
    FileSystem {
        message: String,
        path: Option<String>,
        operation: FileOperation,
        #[source]
        source: Option<std::io::Error>,
        correlation_id: Option<String>,
    },

    /// Network/HTTP related errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        url: Option<String>,
        status_code: Option<u16>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },

    /// Security related errors
    #[error("Security error: {message}")]
    Security {
        message: String,
        severity: SecuritySeverity,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },

    /// Service startup/shutdown errors
    #[error("Service error: {message}")]
    Service {
        message: String,
        service: String,
        operation: ServiceOperation,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
        value: Option<String>,
        #[source]
        source: Option<validator::ValidationErrors>,
        correlation_id: Option<String>,
    },

    /// Migration related errors
    #[error("Migration error: {message}")]
    Migration {
        message: String,
        migration_name: Option<String>,
        operation: MigrationOperation,
        #[source]
        source: Option<sqlx::migrate::MigrateError>,
        correlation_id: Option<String>,
    },

    /// Monitoring/Metrics errors
    #[error("Monitoring error: {message}")]
    Monitoring {
        message: String,
        component: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },

    /// Process management errors
    #[error("Process error: {message}")]
    Process {
        message: String,
        pid: Option<u32>,
        operation: ProcessOperation,
        #[source]
        source: Option<std::io::Error>,
        correlation_id: Option<String>,
    },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        correlation_id: Option<String>,
    },
}

/// File system operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum FileOperation {
    Read,
    Write,
    Create,
    Delete,
    Copy,
    Move,
    CreateDirectory,
    DeleteDirectory,
}

impl fmt::Display for FileOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileOperation::Read => write!(f, "read"),
            FileOperation::Write => write!(f, "write"),
            FileOperation::Create => write!(f, "create"),
            FileOperation::Delete => write!(f, "delete"),
            FileOperation::Copy => write!(f, "copy"),
            FileOperation::Move => write!(f, "move"),
            FileOperation::CreateDirectory => write!(f, "create directory"),
            FileOperation::DeleteDirectory => write!(f, "delete directory"),
        }
    }
}

/// Security error severity levels
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecuritySeverity::Low => write!(f, "low"),
            SecuritySeverity::Medium => write!(f, "medium"),
            SecuritySeverity::High => write!(f, "high"),
            SecuritySeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Service operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum ServiceOperation {
    Start,
    Stop,
    Restart,
    HealthCheck,
    Configuration,
}

impl fmt::Display for ServiceOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceOperation::Start => write!(f, "start"),
            ServiceOperation::Stop => write!(f, "stop"),
            ServiceOperation::Restart => write!(f, "restart"),
            ServiceOperation::HealthCheck => write!(f, "health check"),
            ServiceOperation::Configuration => write!(f, "configuration"),
        }
    }
}

/// Migration operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum MigrationOperation {
    Up,
    Down,
    Status,
    Validate,
    Check,
}

impl fmt::Display for MigrationOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationOperation::Up => write!(f, "up"),
            MigrationOperation::Down => write!(f, "down"),
            MigrationOperation::Status => write!(f, "status"),
            MigrationOperation::Validate => write!(f, "validate"),
            MigrationOperation::Check => write!(f, "check"),
        }
    }
}

/// Process operations
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are planned for future use
pub enum ProcessOperation {
    Start,
    Stop,
    Signal,
    PidFile,
    Daemon,
}

impl fmt::Display for ProcessOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessOperation::Start => write!(f, "start"),
            ProcessOperation::Stop => write!(f, "stop"),
            ProcessOperation::Signal => write!(f, "signal"),
            ProcessOperation::PidFile => write!(f, "pid file"),
            ProcessOperation::Daemon => write!(f, "daemon"),
        }
    }
}

#[allow(dead_code)] // Some methods are planned for future use
impl AppError {
    /// Create a configuration error with correlation ID
    pub fn configuration<S: Into<String>>(message: S) -> Self {
        AppError::Configuration {
            message: message.into(),
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a configuration error with source and correlation ID
    pub fn configuration_with_source<
        S: Into<String>,
        E: std::error::Error + Send + Sync + 'static,
    >(
        message: S,
        source: E,
    ) -> Self {
        AppError::Configuration {
            message: message.into(),
            source: Some(Box::new(source)),
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a database error with correlation ID
    pub fn database<S: Into<String>>(message: S) -> Self {
        AppError::Database {
            message: message.into(),
            source: None,
            query: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a database error with query and correlation ID
    pub fn database_with_query<S: Into<String>, Q: Into<String>>(message: S, query: Q) -> Self {
        AppError::Database {
            message: message.into(),
            source: None,
            query: Some(query.into()),
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a database error with source and correlation ID
    pub fn database_with_source<S: Into<String>>(message: S, source: sqlx::Error) -> Self {
        AppError::Database {
            message: message.into(),
            source: Some(source),
            query: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a CLI error with suggestion
    pub fn command_line<S: Into<String>>(message: S) -> Self {
        AppError::CommandLine {
            message: message.into(),
            command: None,
            suggestion: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a CLI error with command and suggestion
    pub fn command_line_with_suggestion<S: Into<String>, C: Into<String>, G: Into<String>>(
        message: S,
        command: C,
        suggestion: G,
    ) -> Self {
        AppError::CommandLine {
            message: message.into(),
            command: Some(command.into()),
            suggestion: Some(suggestion.into()),
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a file system error
    pub fn file_system<S: Into<String>, P: Into<String>>(
        message: S,
        path: P,
        operation: FileOperation,
    ) -> Self {
        AppError::FileSystem {
            message: message.into(),
            path: Some(path.into()),
            operation,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a file system error with source
    pub fn file_system_with_source<S: Into<String>, P: Into<String>>(
        message: S,
        path: P,
        operation: FileOperation,
        source: std::io::Error,
    ) -> Self {
        AppError::FileSystem {
            message: message.into(),
            path: Some(path.into()),
            operation,
            source: Some(source),
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a security error
    pub fn security<S: Into<String>>(message: S, severity: SecuritySeverity) -> Self {
        AppError::Security {
            message: message.into(),
            severity,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a service error
    pub fn service<S: Into<String>, N: Into<String>>(
        message: S,
        service: N,
        operation: ServiceOperation,
    ) -> Self {
        AppError::Service {
            message: message.into(),
            service: service.into(),
            operation,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(message: S) -> Self {
        AppError::Validation {
            message: message.into(),
            field: None,
            value: None,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a validation error with field and value
    pub fn validation_with_field<S: Into<String>, F: Into<String>, V: Into<String>>(
        message: S,
        field: F,
        value: V,
    ) -> Self {
        AppError::Validation {
            message: message.into(),
            field: Some(field.into()),
            value: Some(value.into()),
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a validation error with source
    pub fn validation_with_source<S: Into<String>>(
        message: S,
        source: validator::ValidationErrors,
    ) -> Self {
        AppError::Validation {
            message: message.into(),
            field: None,
            value: None,
            source: Some(source),
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a migration error
    pub fn migration<S: Into<String>>(message: S, operation: MigrationOperation) -> Self {
        AppError::Migration {
            message: message.into(),
            migration_name: None,
            operation,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create a process error
    pub fn process<S: Into<String>>(message: S, operation: ProcessOperation) -> Self {
        AppError::Process {
            message: message.into(),
            pid: None,
            operation,
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        AppError::Internal {
            message: message.into(),
            source: None,
            correlation_id: Some(generate_correlation_id()),
        }
    }

    /// Get the correlation ID for this error
    pub fn correlation_id(&self) -> Option<&str> {
        match self {
            AppError::Configuration { correlation_id, .. }
            | AppError::Database { correlation_id, .. }
            | AppError::CommandLine { correlation_id, .. }
            | AppError::FileSystem { correlation_id, .. }
            | AppError::Network { correlation_id, .. }
            | AppError::Security { correlation_id, .. }
            | AppError::Service { correlation_id, .. }
            | AppError::Validation { correlation_id, .. }
            | AppError::Migration { correlation_id, .. }
            | AppError::Monitoring { correlation_id, .. }
            | AppError::Process { correlation_id, .. }
            | AppError::Internal { correlation_id, .. } => correlation_id.as_deref(),
        }
    }

    /// Add context to the error
    pub fn with_context<S: Into<String>>(mut self, context: S) -> Self {
        match &mut self {
            AppError::Configuration { message, .. }
            | AppError::Database { message, .. }
            | AppError::CommandLine { message, .. }
            | AppError::FileSystem { message, .. }
            | AppError::Network { message, .. }
            | AppError::Security { message, .. }
            | AppError::Service { message, .. }
            | AppError::Validation { message, .. }
            | AppError::Migration { message, .. }
            | AppError::Monitoring { message, .. }
            | AppError::Process { message, .. }
            | AppError::Internal { message, .. } => {
                *message = format!("{}: {}", context.into(), message);
            }
        }
        self
    }

    /// Log the error with appropriate level based on severity
    pub fn log_error(&self) {
        match self {
            AppError::Security {
                severity: SecuritySeverity::Critical | SecuritySeverity::High,
                ..
            } => {
                error!(
                    error = %self,
                    correlation_id = ?self.correlation_id(),
                    "Critical security error occurred"
                );
            }
            AppError::Database { .. } | AppError::Service { .. } | AppError::Migration { .. } => {
                error!(
                    error = %self,
                    correlation_id = ?self.correlation_id(),
                    "System error occurred"
                );
            }
            AppError::Configuration { .. } | AppError::CommandLine { .. } => {
                warn!(
                    error = %self,
                    correlation_id = ?self.correlation_id(),
                    "Configuration or command error occurred"
                );
            }
            _ => {
                error!(
                    error = %self,
                    correlation_id = ?self.correlation_id(),
                    "Application error occurred"
                );
            }
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Configuration { message, .. } => {
                format!("Configuration Error: {}", message)
            }
            AppError::Database { message, .. } => {
                format!("Database Error: {}", message)
            }
            AppError::CommandLine {
                message,
                command,
                suggestion,
                ..
            } => {
                let mut msg = format!("Command Error: {}", message);
                if let Some(cmd) = command {
                    msg.push_str(&format!(" (command: {})", cmd));
                }
                if let Some(suggest) = suggestion {
                    msg.push_str(&format!("\nSuggestion: {}", suggest));
                }
                msg
            }
            AppError::FileSystem {
                message,
                path,
                operation,
                ..
            } => {
                let mut msg = format!("File System Error: {}", message);
                if let Some(p) = path {
                    msg.push_str(&format!(" (operation: {}, path: {})", operation, p));
                }
                msg
            }
            AppError::Security {
                message, severity, ..
            } => {
                format!("Security Error ({}): {}", severity, message)
            }
            AppError::Service {
                message,
                service,
                operation,
                ..
            } => {
                format!(
                    "Service Error: {} (service: {}, operation: {})",
                    message, service, operation
                )
            }
            AppError::Validation {
                message,
                field,
                value,
                ..
            } => {
                let mut msg = format!("Validation Error: {}", message);
                if let Some(f) = field {
                    msg.push_str(&format!(" (field: {})", f));
                    if let Some(v) = value {
                        msg.push_str(&format!(", value: '{}')", v));
                    }
                }
                msg
            }
            AppError::Migration {
                message,
                migration_name,
                operation,
                ..
            } => {
                let mut msg = format!("Migration Error: {} (operation: {})", message, operation);
                if let Some(name) = migration_name {
                    msg.push_str(&format!(", migration: {}", name));
                }
                msg
            }
            AppError::Process {
                message,
                pid,
                operation,
                ..
            } => {
                let mut msg = format!("Process Error: {} (operation: {})", message, operation);
                if let Some(p) = pid {
                    msg.push_str(&format!(", PID: {}", p));
                }
                msg
            }
            _ => format!("Error: {}", self),
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            AppError::Configuration { .. }
            | AppError::CommandLine { .. }
            | AppError::Validation { .. } => false, // User input errors are not recoverable
            AppError::Security {
                severity: SecuritySeverity::Critical | SecuritySeverity::High,
                ..
            } => false, // High severity security errors are not recoverable
            AppError::Network { .. } | AppError::FileSystem { .. } => true, // These might be temporary
            AppError::Database { .. } => true, // Database errors might be temporary
            _ => false,                        // Conservative default
        }
    }
}

/// Generate a correlation ID for error tracking
fn generate_correlation_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Convert from anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::internal(format!("Unexpected error: {}", err))
    }
}

/// Convert from std::io::Error to AppError
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::file_system_with_source(
            "File system operation failed",
            "unknown",
            FileOperation::Read,
            err,
        )
    }
}

/// Convert from sqlx::Error to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::database_with_source("Database operation failed", err)
    }
}

/// Convert from validator::ValidationErrors to AppError
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let message = err
            .field_errors()
            .iter()
            .map(|(field, errors)| {
                let error_messages: Vec<String> = errors
                    .iter()
                    .filter_map(|e| e.message.as_ref())
                    .map(|m| m.to_string())
                    .collect();
                format!("{}: {}", field, error_messages.join(", "))
            })
            .collect::<Vec<_>>()
            .join("; ");

        AppError::Validation {
            message,
            field: None,
            value: None,
            source: Some(err),
            correlation_id: Some(generate_correlation_id()),
        }
    }
}

/// Result type alias for our application
/// Note: AppError is intentionally large to provide comprehensive error context
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = AppError::configuration("Test configuration error");
        assert!(matches!(err, AppError::Configuration { .. }));
        assert!(err.correlation_id().is_some());
    }

    #[test]
    fn test_error_with_context() {
        let err = AppError::database("Connection failed").with_context("During startup");
        if let AppError::Database { message, .. } = err {
            assert!(message.contains("During startup"));
            assert!(message.contains("Connection failed"));
        } else {
            panic!("Expected Database error");
        }
    }

    #[test]
    fn test_user_message() {
        let err = AppError::command_line_with_suggestion(
            "Invalid argument",
            "web start",
            "Try 'imkitchen --help'",
        );
        let msg = err.user_message();
        assert!(msg.contains("Invalid argument"));
        assert!(msg.contains("web start"));
        assert!(msg.contains("Try 'imkitchen --help'"));
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(!AppError::configuration("Test").is_recoverable());
        assert!(AppError::database("Test").is_recoverable());
        assert!(!AppError::security("Test", SecuritySeverity::Critical).is_recoverable());
        assert!(!AppError::security("Test", SecuritySeverity::High).is_recoverable());
    }

    #[test]
    fn test_correlation_id() {
        let err1 = AppError::internal("Test 1");
        let err2 = AppError::internal("Test 2");

        assert!(err1.correlation_id().is_some());
        assert!(err2.correlation_id().is_some());
        assert_ne!(err1.correlation_id(), err2.correlation_id());
    }

    #[test]
    fn test_from_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::FileSystem { .. }));

        let anyhow_err = anyhow::anyhow!("Test anyhow error");
        let app_err: AppError = anyhow_err.into();
        assert!(matches!(app_err, AppError::Internal { .. }));
    }
}
