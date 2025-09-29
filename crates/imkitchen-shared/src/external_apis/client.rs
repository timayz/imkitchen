use super::config::{ApiConfig, ApiCredentials};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use validator::Validate;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error: {0}")]
    HttpError(u16),

    #[error("Timeout error")]
    TimeoutError,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Retry attempts exhausted")]
    RetryExhausted,

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl ApiResponse {
    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn is_client_error(&self) -> bool {
        self.status >= 400 && self.status < 500
    }

    pub fn is_server_error(&self) -> bool {
        self.status >= 500
    }
}

#[derive(Debug)]
struct RateLimiter {
    max_requests: u32,
    time_window: Duration,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    fn new(max_requests: u32, time_window: Duration) -> Self {
        Self {
            max_requests,
            time_window,
            requests: VecDeque::new(),
        }
    }

    fn can_make_request(&mut self) -> bool {
        let now = Instant::now();

        // Remove old requests outside the time window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.time_window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        self.requests.len() < self.max_requests as usize
    }

    fn record_request(&mut self) {
        self.requests.push_back(Instant::now());
    }
}

/// Generic API client with retry logic and error handling
pub struct ApiClient {
    config: ApiConfig,
    credentials: ApiCredentials,
    rate_limiter: Mutex<RateLimiter>,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: ApiConfig, credentials: ApiCredentials) -> Result<Self, ApiError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| ApiError::ConfigurationError(format!("Invalid config: {}", e)))?;

        credentials
            .validate()
            .map_err(|e| ApiError::ConfigurationError(format!("Invalid credentials: {}", e)))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| {
                ApiError::ConfigurationError(format!("Failed to create HTTP client: {}", e))
            })?;

        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute, Duration::from_secs(60));

        Ok(Self {
            config,
            credentials,
            rate_limiter: Mutex::new(rate_limiter),
            client,
        })
    }

    /// Make a GET request to the specified endpoint
    pub async fn get(&self, endpoint: &str) -> Result<ApiResponse, ApiError> {
        self.make_request("GET", endpoint, None).await
    }

    /// Make a POST request to the specified endpoint
    pub async fn post(
        &self,
        endpoint: &str,
        body: Option<String>,
    ) -> Result<ApiResponse, ApiError> {
        self.make_request("POST", endpoint, body).await
    }

    /// Make a PUT request to the specified endpoint
    pub async fn put(&self, endpoint: &str, body: Option<String>) -> Result<ApiResponse, ApiError> {
        self.make_request("PUT", endpoint, body).await
    }

    /// Make a DELETE request to the specified endpoint
    pub async fn delete(&self, endpoint: &str) -> Result<ApiResponse, ApiError> {
        self.make_request("DELETE", endpoint, None).await
    }

    async fn make_request(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<String>,
    ) -> Result<ApiResponse, ApiError> {
        // Check rate limiting
        {
            let mut rate_limiter = self.rate_limiter.lock().await;
            if !rate_limiter.can_make_request() {
                return Err(ApiError::RateLimitExceeded);
            }
            rate_limiter.record_request();
        }

        let url = format!("{}{}", self.config.base_url, endpoint);
        let mut attempts = 0;

        while attempts <= self.config.retry_attempts {
            match self.execute_request(method, &url, body.as_deref()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempts += 1;

                    // Don't retry client errors (4xx)
                    if let ApiError::HttpError(status) = &e {
                        if *status >= 400 && *status < 500 {
                            return Err(e);
                        }
                    }

                    // Don't retry authentication failures
                    if matches!(e, ApiError::AuthenticationFailed) {
                        return Err(e);
                    }

                    // If this was the last attempt, return the error
                    if attempts > self.config.retry_attempts {
                        return Err(ApiError::RetryExhausted);
                    }

                    // Wait before retrying (exponential backoff)
                    let delay = Duration::from_millis(1000 * 2_u64.pow(attempts - 1));
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(ApiError::RetryExhausted)
    }

    async fn execute_request(
        &self,
        method: &str,
        url: &str,
        body: Option<&str>,
    ) -> Result<ApiResponse, ApiError> {
        let mut request = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            _ => {
                return Err(ApiError::ConfigurationError(format!(
                    "Unsupported method: {}",
                    method
                )))
            }
        };

        // Add authentication headers
        request = request.header(
            "Authorization",
            format!("Bearer {}", self.credentials.api_key),
        );

        // Add additional headers
        for (key, value) in &self.credentials.additional_headers {
            request = request.header(key, value);
        }

        // Add body if provided
        if let Some(body_content) = body {
            request = request
                .header("Content-Type", "application/json")
                .body(body_content.to_string());
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::TimeoutError
            } else {
                ApiError::NetworkError(e.to_string())
            }
        })?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response
            .text()
            .await
            .map_err(|e| ApiError::NetworkError(format!("Failed to read response body: {}", e)))?;

        if status >= 400 {
            if status == 401 || status == 403 {
                return Err(ApiError::AuthenticationFailed);
            } else {
                return Err(ApiError::HttpError(status));
            }
        }

        Ok(ApiResponse {
            status,
            headers,
            body,
        })
    }

    /// Check if the API is accessible
    pub async fn health_check(&self) -> Result<bool, ApiError> {
        // Try health endpoint first, fallback to root
        match self.get("/health").await {
            Ok(response) => Ok(response.is_success()),
            Err(_) => {
                // Fallback to root endpoint
                match self.get("/").await {
                    Ok(response) => Ok(response.is_success()),
                    Err(_) => Ok(false),
                }
            }
        }
    }
}
