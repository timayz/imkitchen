use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::info;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use uuid::Uuid;

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
pub fn setup_monitoring(
    log_level: Option<&String>,
    log_format: &LogFormat,
    log_dir: &Option<PathBuf>,
    log_rotation: Rotation,
) -> Result<()> {
    // Build environment filter from CLI args or RUST_LOG
    let env_filter = if let Some(level) = log_level {
        EnvFilter::new(level)
    } else {
        EnvFilter::from_default_env()
            .add_directive("imkitchen=info".parse()?)
            .add_directive("sqlx=warn".parse()?)
            .add_directive("tokio=warn".parse()?)
            .add_directive("hyper=warn".parse()?)
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

fn setup_stdout_logging_json(env_filter: EnvFilter) -> Result<()> {
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

fn setup_stdout_logging_pretty(env_filter: EnvFilter) -> Result<()> {
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

fn setup_stdout_logging_compact(env_filter: EnvFilter) -> Result<()> {
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

fn setup_file_logging_json(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> Result<()> {
    fs::create_dir_all(log_dir).context("Failed to create log directory")?;

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

fn setup_file_logging_pretty(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> Result<()> {
    fs::create_dir_all(log_dir).context("Failed to create log directory")?;

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

fn setup_file_logging_compact(
    log_dir: &PathBuf,
    rotation: &Rotation,
    env_filter: EnvFilter,
) -> Result<()> {
    fs::create_dir_all(log_dir).context("Failed to create log directory")?;

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
pub fn with_error_correlation<T>(
    result: Result<T>,
    operation: &str,
    correlation_id: &str,
) -> Result<T> {
    result.with_context(|| {
        format!(
            "Operation '{}' failed [correlation_id={}]",
            operation, correlation_id
        )
    })
}
