// Common utility functions

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Generate a new correlation ID for request tracing
pub fn generate_correlation_id() -> Uuid {
    Uuid::new_v4()
}

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Convert seconds to human readable duration
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}