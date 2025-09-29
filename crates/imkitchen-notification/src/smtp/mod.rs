pub mod client;
pub mod connection_manager;

pub use client::{SmtpClient, SmtpConnectionError};
pub use connection_manager::{ConnectionStats, RetryConfig, SmtpConnectionManager};
