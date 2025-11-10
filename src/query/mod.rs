mod admin_user;
mod contact;
mod global_stat;

pub use admin_user::*;
pub use contact::*;
pub use global_stat::*;

use time::OffsetDateTime;

pub fn format_relative_time(timestamp: u64) -> String {
    let now = OffsetDateTime::now_utc().unix_timestamp() as u64;

    if timestamp > now {
        return "just now".to_string();
    }

    let diff = now - timestamp;
    let minutes = diff / 60;
    let hours = diff / 3600;
    let days = diff / 86400;

    match diff {
        s if s < 60 => "just now".to_string(),
        s if s < 3600 => {
            if minutes == 1 {
                "1 minute ago".to_string()
            } else {
                format!("{} minutes ago", minutes)
            }
        }
        s if s < 86400 => {
            if hours == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{} hours ago", hours)
            }
        }
        s if s < 172800 => "yesterday".to_string(), // 2 days in seconds
        s if s < 604800 => format!("{} days ago", days), // 7 days
        s if s < 2592000 => {
            let weeks = days / 7;
            if weeks == 1 {
                "1 week ago".to_string()
            } else {
                format!("{} weeks ago", weeks)
            }
        }
        s if s < 31536000 => {
            let months = days / 30;
            if months == 1 {
                "1 month ago".to_string()
            } else {
                format!("{} months ago", months)
            }
        }
        _ => {
            let years = days / 365;
            if years == 1 {
                "1 year ago".to_string()
            } else {
                format!("{} years ago", years)
            }
        }
    }
}
