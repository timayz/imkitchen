use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LogFormat {
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "compact")]
    Compact,
}

/// Sets up comprehensive monitoring with structured logging, correlation IDs, and log rotation
#[allow(clippy::result_large_err)]
pub fn setup_monitoring(
    log_level: Option<&String>,
    log_format: &LogFormat,
    log_dir: &Option<PathBuf>,
    log_rotation: Rotation,
) -> AppResult<()> {
    // Build environment filter from CLI args or RUST_LOG
    let env_filter = if let Some(level) = log_level {
        EnvFilter::new(level)
    } else {
        EnvFilter::from_default_env()
            .add_directive("imkitchen=info".parse().map_err(|e| {
                AppError::configuration_with_source("Invalid logging directive for imkitchen", e)
            })?)
            .add_directive("sqlx=warn".parse().map_err(|e| {
                AppError::configuration_with_source("Invalid logging directive for sqlx", e)
            })?)
            .add_directive("tokio=warn".parse().map_err(|e| {
                AppError::configuration_with_source("Invalid logging directive for tokio", e)
            })?)
            .add_directive("hyper=warn".parse().map_err(|e| {
                AppError::configuration_with_source("Invalid logging directive for hyper", e)
            })?)
    };

    match log_dir {
        Some(log_dir) => {
            // File output - choose format
            match log_format {
                LogFormat::Json => setup_file_logging_json(log_dir, &log_rotation, env_filter)?,
                LogFormat::Pretty => setup_file_logging_pretty(log_dir, &log_rotation, env_filter)?,
                LogFormat::Compact => {
                    setup_file_logging_compact(log_dir, &log_rotation, env_filter)?
                }
            }
        }
        None => {
            // Stdout output - choose format
            match log_format {
                LogFormat::Json => setup_stdout_logging_json(env_filter)?,
                LogFormat::Pretty => setup_stdout_logging_pretty(env_filter)?,
                LogFormat::Compact => setup_stdout_logging_compact(env_filter)?,
            }
        }
    }

    info!(
        "Monitoring stack initialized with format: {:?}, directory: {:?}, rotation: {:?}",
        log_format, log_dir, log_rotation
    );

    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_stdout_logging_json(env_filter: EnvFilter) -> AppResult<()> {
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .json()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_current_span(false)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();
    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_stdout_logging_pretty(env_filter: EnvFilter) -> AppResult<()> {
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .pretty()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_line_number(true)
                .with_file(true)
                .with_thread_ids(true)
                .with_thread_names(true),
        )
        .init();
    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_stdout_logging_compact(env_filter: EnvFilter) -> AppResult<()> {
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .compact()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_target(false)
                .with_thread_ids(false),
        )
        .init();
    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_file_logging_json(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> AppResult<()> {
    fs::create_dir_all(log_dir).map_err(|e| {
        AppError::file_system_with_source(
            "Failed to create log directory",
            log_dir.to_string_lossy().to_string(),
            crate::error::FileOperation::Create,
            e,
        )
    })?;

    let file_appender = RollingFileAppender::new(rotation.clone(), log_dir, "imkitchen.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    // Also output to stdout for immediate feedback
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .json()
                .with_span_events(fmt::format::FmtSpan::CLOSE),
        )
        .with(
            fmt::layer()
                .json()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    std::mem::forget(_guard); // Keep the guard alive for the duration of the program
    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_file_logging_pretty(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> AppResult<()> {
    fs::create_dir_all(log_dir).map_err(|e| {
        AppError::file_system_with_source(
            "Failed to create log directory",
            log_dir.to_string_lossy().to_string(),
            crate::error::FileOperation::Create,
            e,
        )
    })?;

    let file_appender = RollingFileAppender::new(rotation.clone(), log_dir, "imkitchen.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .pretty()
                .with_span_events(fmt::format::FmtSpan::CLOSE),
        )
        .with(
            fmt::layer()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    std::mem::forget(_guard);
    Ok(())
}

#[allow(clippy::result_large_err)]
fn setup_file_logging_compact(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> AppResult<()> {
    fs::create_dir_all(log_dir).map_err(|e| {
        AppError::file_system_with_source(
            "Failed to create log directory",
            log_dir.to_string_lossy().to_string(),
            crate::error::FileOperation::Create,
            e,
        )
    })?;

    let file_appender = RollingFileAppender::new(rotation.clone(), log_dir, "imkitchen.log");
    let (non_blocking, _guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .compact()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_target(false),
        )
        .with(
            fmt::layer()
                .compact()
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_writer(non_blocking)
                .with_target(false)
                .with_ansi(false),
        )
        .init();

    std::mem::forget(_guard);
    Ok(())
}

/// Create a correlation ID for the current request/operation
#[allow(dead_code)]
pub fn create_correlation_id() -> String {
    Uuid::new_v4().to_string()
}

/// Enhanced error context with correlation for debugging
#[allow(dead_code)]
#[allow(clippy::result_large_err)]
pub fn with_error_correlation<T>(
    result: AppResult<T>,
    operation: &str,
    correlation_id: &str,
) -> AppResult<T> {
    result.map_err(|err| {
        err.with_context(format!(
            "Operation '{}' failed [correlation_id={}]",
            operation, correlation_id
        ))
    })
}
