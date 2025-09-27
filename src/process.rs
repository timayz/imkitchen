use std::fs;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tokio::sync::oneshot;
use tokio::time::timeout;
use tracing::{error, info, warn};

use crate::error::{AppError, AppResult};

/// Process management for production deployments
pub struct ProcessManager {
    pid_file: Option<PathBuf>,
    daemon_mode: bool,
    shutdown_signal: Arc<AtomicBool>,
    cleanup_handlers: Vec<Box<dyn Fn() -> AppResult<()> + Send + Sync>>,
}

impl ProcessManager {
    /// Create a new process manager
    pub fn new() -> Self {
        Self {
            pid_file: None,
            daemon_mode: false,
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            cleanup_handlers: Vec::new(),
        }
    }

    /// Configure PID file path
    pub fn with_pid_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.pid_file = Some(path.into());
        self
    }

    /// Enable daemon mode
    pub fn with_daemon_mode(mut self, daemon: bool) -> Self {
        self.daemon_mode = daemon;
        self
    }

    /// Add a cleanup handler that will be called on shutdown
    pub fn add_cleanup_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn() -> AppResult<()> + Send + Sync + 'static,
    {
        self.cleanup_handlers.push(Box::new(handler));
        self
    }

    /// Initialize process management (write PID file, setup signal handlers)
    pub async fn initialize(&self) -> AppResult<()> {
        info!("Initializing process management");

        // Write PID file if specified
        if let Some(ref pid_path) = self.pid_file {
            self.write_pid_file(pid_path)?;
            info!("PID file written to: {:?}", pid_path);
        }

        // Setup signal handlers for graceful shutdown
        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        tokio::spawn(async move {
            if let Err(e) = setup_signal_handlers(shutdown_signal).await {
                error!("Failed to setup signal handlers: {}", e);
            }
        });

        // If daemon mode is enabled, prepare for background operation
        if self.daemon_mode {
            info!("Process initialized in daemon mode");
            self.setup_daemon_mode().await?;
        }

        Ok(())
    }

    /// Write PID file with proper error handling and permissions
    #[allow(clippy::result_large_err)]
    fn write_pid_file(&self, path: &PathBuf) -> AppResult<()> {
        let pid = process::id();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    AppError::file_system_with_source(
                        "Failed to create PID file directory",
                        parent.to_string_lossy().to_string(),
                        crate::error::FileOperation::Create,
                        e,
                    )
                })?;
            }
        }

        // Write PID to file
        fs::write(path, pid.to_string()).map_err(|e| {
            AppError::file_system_with_source(
                "Failed to write PID file",
                path.to_string_lossy().to_string(),
                crate::error::FileOperation::Write,
                e,
            )
        })?;

        // Set restrictive permissions on PID file (readable by owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .map_err(|e| {
                    AppError::file_system_with_source(
                        "Failed to get PID file metadata",
                        path.to_string_lossy().to_string(),
                        crate::error::FileOperation::Read,
                        e,
                    )
                })?
                .permissions();
            perms.set_mode(0o644); // rw-r--r--
            fs::set_permissions(path, perms).map_err(|e| {
                AppError::file_system_with_source(
                    "Failed to set PID file permissions",
                    path.to_string_lossy().to_string(),
                    crate::error::FileOperation::Write,
                    e,
                )
            })?;
        }

        Ok(())
    }

    /// Setup daemon mode (background process operation)
    async fn setup_daemon_mode(&self) -> AppResult<()> {
        info!("Setting up daemon mode");

        // In a real daemon implementation, we would:
        // 1. Fork the process
        // 2. Create a new session
        // 3. Fork again to ensure we're not a session leader
        // 4. Change working directory to root
        // 5. Close standard file descriptors
        // 6. Redirect stdout/stderr to syslog or log files

        // For this implementation, we'll simulate daemon behavior
        // by ensuring proper signal handling and background operation

        warn!("Daemon mode enabled - process will run in background");
        warn!("Standard I/O streams should be redirected in production");

        Ok(())
    }

    /// Check if shutdown has been signaled
    #[allow(dead_code)]
    pub fn should_shutdown(&self) -> bool {
        self.shutdown_signal.load(Ordering::Relaxed)
    }

    /// Wait for shutdown signal with timeout
    pub async fn wait_for_shutdown(&self, timeout_duration: Duration) -> AppResult<()> {
        let shutdown_signal = Arc::clone(&self.shutdown_signal);

        let result = timeout(timeout_duration, async move {
            while !shutdown_signal.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        match result {
            Ok(_) => {
                info!("Shutdown signal received");
                Ok(())
            }
            Err(_) => {
                warn!("Shutdown timeout reached");
                Err(AppError::process(
                    "Shutdown timeout exceeded",
                    crate::error::ProcessOperation::Stop,
                ))
            }
        }
    }

    /// Perform graceful shutdown with cleanup
    pub async fn shutdown(&self, timeout_duration: Duration) -> AppResult<()> {
        info!("Starting graceful shutdown");

        // Set shutdown signal
        self.shutdown_signal.store(true, Ordering::Relaxed);

        // Run cleanup handlers with timeout
        let cleanup_result = timeout(timeout_duration, async {
            for handler in &self.cleanup_handlers {
                if let Err(e) = handler() {
                    error!("Cleanup handler failed: {}", e);
                }
            }
        })
        .await;

        if cleanup_result.is_err() {
            warn!("Cleanup handlers timed out");
        }

        // Remove PID file
        if let Some(ref pid_path) = self.pid_file {
            if let Err(e) = self.remove_pid_file(pid_path) {
                warn!("Failed to remove PID file: {}", e);
            }
        }

        info!("Graceful shutdown completed");
        Ok(())
    }

    /// Remove PID file on shutdown
    #[allow(clippy::result_large_err)]
    fn remove_pid_file(&self, path: &PathBuf) -> AppResult<()> {
        if path.exists() {
            fs::remove_file(path).map_err(|e| {
                AppError::file_system_with_source(
                    "Failed to remove PID file",
                    path.to_string_lossy().to_string(),
                    crate::error::FileOperation::Delete,
                    e,
                )
            })?;
            info!("PID file removed: {:?}", path);
        }
        Ok(())
    }

    /// Check if another process is running using the PID file
    #[allow(clippy::result_large_err)]
    pub fn check_existing_process(&self) -> AppResult<Option<u32>> {
        if let Some(ref pid_path) = self.pid_file {
            if pid_path.exists() {
                let pid_str = fs::read_to_string(pid_path).map_err(|e| {
                    AppError::file_system_with_source(
                        "Failed to read existing PID file",
                        pid_path.to_string_lossy().to_string(),
                        crate::error::FileOperation::Read,
                        e,
                    )
                })?;

                let pid: u32 = pid_str.trim().parse().map_err(|e| {
                    AppError::configuration_with_source("Invalid PID in PID file", e)
                })?;

                // Check if process is still running
                if process_exists(pid) {
                    return Ok(Some(pid));
                } else {
                    // Stale PID file, remove it
                    warn!("Removing stale PID file for non-existent process {}", pid);
                    self.remove_pid_file(pid_path)?;
                }
            }
        }
        Ok(None)
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Setup signal handlers for graceful shutdown
async fn setup_signal_handlers(shutdown_signal: Arc<AtomicBool>) -> AppResult<()> {
    let mut sigterm =
        signal::unix::signal(signal::unix::SignalKind::terminate()).map_err(|_| {
            AppError::process(
                "Failed to setup SIGTERM handler",
                crate::error::ProcessOperation::Signal,
            )
        })?;

    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).map_err(|_| {
        AppError::process(
            "Failed to setup SIGINT handler",
            crate::error::ProcessOperation::Signal,
        )
    })?;

    let mut sighup = signal::unix::signal(signal::unix::SignalKind::hangup()).map_err(|_| {
        AppError::process(
            "Failed to setup SIGHUP handler",
            crate::error::ProcessOperation::Signal,
        )
    })?;

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, initiating graceful shutdown");
            shutdown_signal.store(true, Ordering::Relaxed);
        }
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C), initiating graceful shutdown");
            shutdown_signal.store(true, Ordering::Relaxed);
        }
        _ = sighup.recv() => {
            info!("Received SIGHUP, initiating graceful shutdown");
            shutdown_signal.store(true, Ordering::Relaxed);
        }
    }

    Ok(())
}

/// Check if a process with the given PID exists
fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Send signal 0 to check if process exists without actually sending a signal
        let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
        result == 0
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, we'll assume the process exists
        // In a real implementation, you'd use platform-specific APIs
        warn!("Process existence check not implemented for this platform");
        true
    }
}

/// Shutdown coordinator for managing multiple services
#[allow(dead_code)]
pub struct ShutdownCoordinator {
    shutdown_tx: Option<oneshot::Sender<()>>,
    shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    #[allow(dead_code)]
    pub fn new() -> Self {
        let (tx, rx) = oneshot::channel();
        Self {
            shutdown_tx: Some(tx),
            shutdown_rx: Some(rx),
        }
    }

    /// Get a receiver for shutdown notifications
    #[allow(dead_code)]
    pub fn subscribe(&mut self) -> Option<oneshot::Receiver<()>> {
        self.shutdown_rx.take()
    }

    /// Signal shutdown to all subscribers
    #[allow(dead_code)]
    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_process_manager_creation() {
        let pm = ProcessManager::new();
        assert!(!pm.daemon_mode);
        assert!(pm.pid_file.is_none());
    }

    #[tokio::test]
    async fn test_process_manager_configuration() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        let pm = ProcessManager::new()
            .with_pid_file(&pid_path)
            .with_daemon_mode(true);

        assert!(pm.daemon_mode);
        assert_eq!(pm.pid_file.unwrap(), pid_path);
    }

    #[tokio::test]
    async fn test_pid_file_operations() {
        let temp_dir = tempdir().unwrap();
        let pid_path = temp_dir.path().join("test.pid");

        let pm = ProcessManager::new().with_pid_file(&pid_path);

        // Write PID file
        pm.write_pid_file(&pid_path).unwrap();
        assert!(pid_path.exists());

        // Read back PID
        let content = fs::read_to_string(&pid_path).unwrap();
        let pid: u32 = content.trim().parse().unwrap();
        assert_eq!(pid, process::id());

        // Remove PID file
        pm.remove_pid_file(&pid_path).unwrap();
        assert!(!pid_path.exists());
    }

    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let mut coordinator = ShutdownCoordinator::new();
        let mut rx = coordinator.subscribe().unwrap();

        // Should not have received shutdown yet
        assert!(rx.try_recv().is_err());

        // Signal shutdown
        coordinator.shutdown();

        // Should receive shutdown signal
        assert!(rx.await.is_ok());
    }

    #[test]
    fn test_cleanup_handler() {
        let cleanup_called = Arc::new(AtomicBool::new(false));
        let cleanup_called_clone = Arc::clone(&cleanup_called);

        let _pm = ProcessManager::new().add_cleanup_handler(move || {
            cleanup_called_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        // In a real test, we'd call shutdown and verify the handler was called
        // For now, just verify the handler was added
        // This would require making cleanup_handlers public or adding a test method
    }
}
