use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApiConfig {
    #[validate(url(message = "Base URL must be a valid URL"))]
    pub base_url: String,

    #[validate(length(min = 1, message = "API key is required"))]
    pub api_key: String,

    #[validate(range(
        min = 1,
        max = 300,
        message = "Timeout must be between 1 and 300 seconds"
    ))]
    pub timeout_seconds: u64,

    #[validate(range(
        min = 1,
        max = 10000,
        message = "Rate limit must be between 1 and 10000 per minute"
    ))]
    pub rate_limit_per_minute: u32,

    #[validate(range(min = 0, max = 10, message = "Retry attempts must be between 0 and 10"))]
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ApiCredentials {
    #[validate(length(min = 1, message = "API key is required"))]
    pub api_key: String,

    pub secret_key: Option<String>,

    pub additional_headers: HashMap<String, String>,
}

impl ApiConfig {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            timeout_seconds: 30,
            rate_limit_per_minute: 60,
            retry_attempts: 3,
        }
    }

    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    pub fn with_rate_limit(mut self, rate_limit_per_minute: u32) -> Self {
        self.rate_limit_per_minute = rate_limit_per_minute;
        self
    }

    pub fn with_retry_attempts(mut self, retry_attempts: u32) -> Self {
        self.retry_attempts = retry_attempts;
        self
    }
}

impl ApiCredentials {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            secret_key: None,
            additional_headers: HashMap::new(),
        }
    }

    pub fn with_secret_key(mut self, secret_key: String) -> Self {
        self.secret_key = Some(secret_key);
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.additional_headers.insert(key, value);
        self
    }
}
