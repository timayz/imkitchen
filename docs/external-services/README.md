# External Service Integration Guide

This guide covers the integration of external services in IMKitchen, including current SMTP email services and preparation for future API integrations.

## Table of Contents

- [Overview](#overview)
- [SMTP Email Integration](#smtp-email-integration)
- [Environment Variable Security](#environment-variable-security)
- [Development Fallback Configuration](#development-fallback-configuration)
- [Future API Integration Preparation](#future-api-integration-preparation)
- [Service Discovery and Health Checks](#service-discovery-and-health-checks)
- [Monitoring and Observability](#monitoring-and-observability)
- [Troubleshooting](#troubleshooting)

## Overview

IMKitchen uses a layered approach to external service integration:

- **Email Services**: SMTP integration for user notifications and system alerts
- **Future API Services**: Prepared framework for third-party integrations
- **Service Abstraction**: Clean interfaces for easy service swapping
- **Fallback Mechanisms**: Graceful degradation when services are unavailable

### Current External Services

| Service Type | Provider | Purpose | Status |
|--------------|----------|---------|--------|
| Email (SMTP) | Configurable | User notifications, alerts | ✅ Implemented |
| File Storage | Local/Future Cloud | Asset management | 🔄 Planned |
| Payment Processing | Future Integration | Transaction handling | 📋 Prepared |
| Analytics | Future Integration | Usage metrics | 📋 Prepared |

## SMTP Email Integration

### Supported SMTP Providers

IMKitchen supports any SMTP-compliant email service. Here are tested configurations:

#### Gmail (Personal/Business)
```bash
# For Gmail with App Password (recommended)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=your-email@gmail.com
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true
```

#### Gmail with OAuth2 (Enterprise)
```bash
# OAuth2 configuration (future enhancement)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_AUTH_TYPE=oauth2
SMTP_CLIENT_ID=your-client-id
SMTP_CLIENT_SECRET=your-client-secret
SMTP_REFRESH_TOKEN=your-refresh-token
```

#### Amazon SES
```bash
# Amazon Simple Email Service
SMTP_HOST=email-smtp.us-east-1.amazonaws.com
SMTP_PORT=587
SMTP_USERNAME=your-ses-access-key
SMTP_PASSWORD=your-ses-secret-key
SMTP_FROM_EMAIL=verified-email@yourdomain.com
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true
```

#### SendGrid
```bash
# SendGrid SMTP
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_FROM_EMAIL=verified-email@yourdomain.com
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true
```

#### Mailgun
```bash
# Mailgun SMTP
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@your-domain.mailgun.org
SMTP_PASSWORD=your-mailgun-password
SMTP_FROM_EMAIL=noreply@your-domain.com
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true
```

#### Microsoft 365/Outlook
```bash
# Microsoft 365 Business
SMTP_HOST=smtp.office365.com
SMTP_PORT=587
SMTP_USERNAME=your-email@yourdomain.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=your-email@yourdomain.com
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true
```

### SMTP Configuration Steps

#### Step 1: Choose Your Provider
Select an SMTP provider based on your needs:
- **Development**: Gmail with App Password (free, easy setup)
- **Small Production**: SendGrid or Mailgun (reliable, affordable)
- **Enterprise**: Amazon SES or Microsoft 365 (scalable, integrated)

#### Step 2: Obtain Credentials
Follow provider-specific instructions:

**Gmail App Password Setup:**
1. Enable 2-Factor Authentication
2. Go to Google Account settings
3. Security → App passwords
4. Generate password for "Mail"
5. Use generated password in `SMTP_PASSWORD`

**SendGrid API Key:**
1. Create SendGrid account
2. Go to Settings → API Keys
3. Create new API key with "Mail Send" permissions
4. Use API key as `SMTP_PASSWORD`

#### Step 3: Configure Environment Variables
Add to your `.env` file:
```bash
# Required SMTP settings
SMTP_HOST=your-smtp-host
SMTP_PORT=587
SMTP_USERNAME=your-username
SMTP_PASSWORD=your-password
SMTP_FROM_EMAIL=your-from-email
SMTP_FROM_NAME="IMKitchen"
SMTP_USE_TLS=true

# Optional SMTP settings
SMTP_TIMEOUT_SECONDS=30
SMTP_RETRY_ATTEMPTS=3
SMTP_RETRY_DELAY_SECONDS=5
```

#### Step 4: Test Configuration
Use the built-in email test command:
```bash
# Test SMTP configuration
cargo run --bin imkitchen email test-smtp

# Send test email to specific address
cargo run --bin imkitchen email test-smtp --to test@example.com
```

### Email Template System

IMKitchen uses Askama templates for email content:

```rust
// Email template example
#[derive(Template)]
#[template(path = "emails/welcome.html")]
struct WelcomeEmailTemplate {
    user_name: String,
    verification_link: String,
    app_url: String,
}

// Usage in email service
let template = WelcomeEmailTemplate {
    user_name: user.display_name.clone(),
    verification_link: generate_verification_link(&user.id),
    app_url: env::var("APP_URL").unwrap_or_default(),
};

email_service.send_template_email(
    &user.email,
    "Welcome to IMKitchen",
    &template,
).await?;
```

### Email Queue and Reliability

For production deployments, consider implementing:

```rust
// Email queue configuration (future enhancement)
[email]
queue_enabled = true
queue_backend = "redis"  # or "database"
max_retries = 3
retry_backoff = "exponential"
batch_size = 10
```

## Environment Variable Security

### Security Best Practices

#### 1. Never Commit Secrets
```bash
# Add to .gitignore
.env
.env.local
.env.production
*.key
*.pem
config/secrets.toml
```

#### 2. Use Environment-Specific Files
```bash
# Development
.env.development

# Staging  
.env.staging

# Production
.env.production
```

#### 3. Validate Required Variables
```rust
// Environment validation on startup
pub fn validate_environment() -> Result<(), ConfigError> {
    let required_vars = [
        "DATABASE_URL",
        "SESSION_SECRET",
        "SMTP_HOST",
        "SMTP_USERNAME",
        "SMTP_PASSWORD",
    ];
    
    for var in required_vars {
        env::var(var)
            .map_err(|_| ConfigError::MissingVariable(var.to_string()))?;
    }
    
    Ok(())
}
```

#### 4. Use Strong Secrets
```bash
# Generate secure random secrets
openssl rand -hex 32  # For SESSION_SECRET
openssl rand -base64 32  # Alternative format

# Use password managers for SMTP credentials
# Never use personal passwords for service accounts
```

#### 5. Implement Secret Rotation
```rust
// Future enhancement: Secret rotation support
#[derive(Debug, Clone)]
pub struct RotatingSecret {
    current: String,
    previous: Option<String>,
    rotation_interval: Duration,
    last_rotation: SystemTime,
}
```

### Environment Variable Encryption

For sensitive deployments, consider encrypting environment variables:

```bash
# Using sops (Secrets OPerationS)
# Install: https://github.com/mozilla/sops

# Encrypt .env file
sops -e .env > .env.enc

# Decrypt for use
sops -d .env.enc > .env
```

### Docker Secrets Management

For containerized deployments:

```yaml
# docker-compose.yml
version: '3.8'
services:
  imkitchen:
    image: imkitchen:latest
    secrets:
      - smtp_password
      - session_secret
    environment:
      SMTP_PASSWORD_FILE: /run/secrets/smtp_password
      SESSION_SECRET_FILE: /run/secrets/session_secret

secrets:
  smtp_password:
    file: ./secrets/smtp_password.txt
  session_secret:
    file: ./secrets/session_secret.txt
```

```rust
// Reading Docker secrets in Rust
pub fn read_secret_file(env_var: &str) -> Result<String, Box<dyn Error>> {
    let file_path = env::var(format!("{}_FILE", env_var))?;
    let secret = fs::read_to_string(file_path)?.trim().to_string();
    Ok(secret)
}
```

## Development Fallback Configuration

### SMTP Fallback for Development

When SMTP is not configured, IMKitchen gracefully degrades:

```rust
// Email service fallback behavior
impl EmailService {
    pub async fn send_email(&self, email: &Email) -> Result<(), EmailError> {
        match &self.smtp_config {
            Some(config) => {
                // Send via SMTP
                self.send_via_smtp(email, config).await
            }
            None => {
                // Fallback: log email content
                tracing::warn!("SMTP not configured, logging email instead");
                self.log_email_content(email).await;
                Ok(())
            }
        }
    }
    
    async fn log_email_content(&self, email: &Email) {
        tracing::info!(
            to = %email.to,
            subject = %email.subject,
            "Email would be sent: {}",
            email.body
        );
    }
}
```

### Development SMTP Server

Use a local SMTP server for development:

#### Option 1: MailHog (Recommended)
```bash
# Install MailHog
# macOS
brew install mailhog

# Linux
go install github.com/mailhog/MailHog@latest

# Windows
# Download from: https://github.com/mailhog/MailHog/releases

# Run MailHog
mailhog

# Configuration for MailHog
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_USERNAME=""
SMTP_PASSWORD=""
SMTP_FROM_EMAIL=dev@imkitchen.local
SMTP_FROM_NAME="IMKitchen Dev"
SMTP_USE_TLS=false
```

#### Option 2: smtp4dev
```bash
# Run with Docker
docker run -p 3000:80 -p 2525:25 rnwood/smtp4dev

# Configuration for smtp4dev
SMTP_HOST=localhost
SMTP_PORT=2525
SMTP_USERNAME=""
SMTP_PASSWORD=""
SMTP_FROM_EMAIL=dev@imkitchen.local
SMTP_FROM_NAME="IMKitchen Dev"
SMTP_USE_TLS=false
```

### Feature Flags for External Services

Control external service usage with feature flags:

```rust
// Feature flag configuration
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub email_enabled: bool,
    pub email_queue_enabled: bool,
    pub external_apis_enabled: bool,
    pub analytics_enabled: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            email_enabled: env::var("FEATURE_EMAIL")
                .unwrap_or_default()
                .parse()
                .unwrap_or(true),
            email_queue_enabled: env::var("FEATURE_EMAIL_QUEUE")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
            external_apis_enabled: env::var("FEATURE_EXTERNAL_APIS")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
            analytics_enabled: env::var("FEATURE_ANALYTICS")
                .unwrap_or_default()
                .parse()
                .unwrap_or(false),
        }
    }
}
```

## Future API Integration Preparation

### API Integration Framework

IMKitchen is prepared for future third-party API integrations:

```rust
// Generic API client trait
#[async_trait]
pub trait ApiClient: Send + Sync {
    type Config: Clone + Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn new(config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;
        
    async fn health_check(&self) -> Result<(), Self::Error>;
    
    async fn authenticate(&mut self) -> Result<(), Self::Error>;
    
    async fn request<T>(&self, request: ApiRequest) -> Result<T, Self::Error>
    where
        T: serde::DeserializeOwned;
}

// API request abstraction
#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<serde_json::Value>,
    pub timeout: Option<Duration>,
}
```

### Prepared Integration Points

#### 1. Payment Processing APIs
```rust
// Payment provider abstraction
#[async_trait]
pub trait PaymentProvider: ApiClient {
    async fn create_payment_intent(
        &self,
        amount: u64,
        currency: &str,
        metadata: &PaymentMetadata,
    ) -> Result<PaymentIntent, Self::Error>;
    
    async fn confirm_payment(
        &self,
        payment_id: &str,
    ) -> Result<PaymentStatus, Self::Error>;
    
    async fn refund_payment(
        &self,
        payment_id: &str,
        amount: Option<u64>,
    ) -> Result<RefundStatus, Self::Error>;
}

// Stripe implementation (example)
pub struct StripeClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

#[async_trait]
impl PaymentProvider for StripeClient {
    // Implementation details...
}
```

#### 2. File Storage APIs
```rust
// Storage provider abstraction
#[async_trait]
pub trait StorageProvider: ApiClient {
    async fn upload_file(
        &self,
        file_data: &[u8],
        file_name: &str,
        content_type: &str,
    ) -> Result<StorageUrl, Self::Error>;
    
    async fn download_file(
        &self,
        file_id: &str,
    ) -> Result<Vec<u8>, Self::Error>;
    
    async fn delete_file(
        &self,
        file_id: &str,
    ) -> Result<(), Self::Error>;
    
    async fn generate_presigned_url(
        &self,
        file_id: &str,
        expires_in: Duration,
    ) -> Result<String, Self::Error>;
}
```

#### 3. Analytics APIs
```rust
// Analytics provider abstraction
#[async_trait]
pub trait AnalyticsProvider: ApiClient {
    async fn track_event(
        &self,
        user_id: Option<&str>,
        event_name: &str,
        properties: &serde_json::Value,
    ) -> Result<(), Self::Error>;
    
    async fn track_page_view(
        &self,
        user_id: Option<&str>,
        page_url: &str,
        referrer: Option<&str>,
    ) -> Result<(), Self::Error>;
    
    async fn identify_user(
        &self,
        user_id: &str,
        traits: &serde_json::Value,
    ) -> Result<(), Self::Error>;
}
```

### Configuration Management for APIs

```toml
# Future API configuration (config/external_apis.toml)
[payment]
provider = "stripe"  # or "square", "paypal"
environment = "sandbox"  # or "production"
webhook_secret = "${PAYMENT_WEBHOOK_SECRET}"

[storage]
provider = "s3"  # or "gcs", "azure"
bucket = "imkitchen-assets"
region = "us-east-1"

[analytics]
provider = "mixpanel"  # or "amplitude", "segment"
project_token = "${ANALYTICS_PROJECT_TOKEN}"
```

### API Rate Limiting and Circuit Breakers

```rust
// Rate limiting for API calls
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

pub struct RateLimitedApiClient<T> {
    inner: T,
    rate_limiter: RateLimiter<governor::clock::DefaultClock, governor::middleware::NoOpMiddleware>,
}

impl<T> RateLimitedApiClient<T>
where
    T: ApiClient,
{
    pub fn new(inner: T, requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        let rate_limiter = RateLimiter::direct(quota);
        
        Self {
            inner,
            rate_limiter,
        }
    }
}

// Circuit breaker pattern
use circuit_breaker::CircuitBreaker;

pub struct CircuitBreakerApiClient<T> {
    inner: T,
    circuit_breaker: CircuitBreaker,
}
```

## Service Discovery and Health Checks

### Health Check Framework

```rust
// Health check trait for all external services
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self) -> HealthStatus;
    fn service_name(&self) -> &str;
}

#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { message: String },
    Unhealthy { error: String },
}

// Health check aggregator
pub struct HealthChecker {
    services: Vec<Box<dyn HealthCheck>>,
}

impl HealthChecker {
    pub async fn check_all(&self) -> HealthReport {
        let mut checks = Vec::new();
        
        for service in &self.services {
            let status = service.check_health().await;
            checks.push(ServiceHealth {
                name: service.service_name().to_string(),
                status,
                checked_at: SystemTime::now(),
            });
        }
        
        HealthReport { checks }
    }
}
```

### Service Registry

```rust
// Service registry for dynamic service discovery
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub endpoint: String,
    pub health_check_path: String,
    pub last_health_check: Option<SystemTime>,
    pub status: HealthStatus,
}

impl ServiceRegistry {
    pub async fn register_service(&self, info: ServiceInfo) {
        let mut services = self.services.write().await;
        services.insert(info.name.clone(), info);
    }
    
    pub async fn get_healthy_service(&self, name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(name)
            .filter(|info| matches!(info.status, HealthStatus::Healthy))
            .cloned()
    }
}
```

## Monitoring and Observability

### Metrics Collection

```rust
// Metrics for external service calls
use prometheus::{Counter, Histogram, Gauge};

#[derive(Clone)]
pub struct ExternalServiceMetrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: Gauge,
    pub errors_total: Counter,
}

impl ExternalServiceMetrics {
    pub fn new(service_name: &str) -> Self {
        Self {
            requests_total: Counter::new(
                format!("{}_requests_total", service_name),
                format!("Total {} API requests", service_name),
            ).unwrap(),
            request_duration: Histogram::new(
                format!("{}_request_duration_seconds", service_name),
                format!("{} API request duration", service_name),
            ).unwrap(),
            active_connections: Gauge::new(
                format!("{}_active_connections", service_name),
                format!("Active {} connections", service_name),
            ).unwrap(),
            errors_total: Counter::new(
                format!("{}_errors_total", service_name),
                format!("Total {} API errors", service_name),
            ).unwrap(),
        }
    }
}
```

### Distributed Tracing

```rust
// Tracing integration for external service calls
use tracing::{instrument, Span};

impl EmailService {
    #[instrument(
        name = "email_service.send",
        fields(
            to = %email.to,
            subject = %email.subject,
            provider = %self.provider_name()
        )
    )]
    pub async fn send_email(&self, email: &Email) -> Result<(), EmailError> {
        let span = Span::current();
        span.record("email.message_id", &email.id);
        
        // Send email with full tracing context
        self.send_via_smtp(email).await
    }
}
```

## Troubleshooting

### Common SMTP Issues

#### Problem: Authentication Failed
```
Error: SMTP authentication failed
```

**Solutions:**
1. **Check credentials**: Verify username/password are correct
2. **App passwords**: Use app-specific passwords for Gmail/Outlook
3. **2FA**: Ensure 2-Factor Authentication is properly configured
4. **Test connection**:
   ```bash
   # Test SMTP connection manually
   telnet smtp.gmail.com 587
   ```

#### Problem: Connection Timeout
```
Error: Connection to SMTP server timed out
```

**Solutions:**
1. **Check firewall**: Ensure SMTP ports (25, 465, 587) are open
2. **Network connectivity**: Test basic connectivity to SMTP server
3. **Increase timeout**:
   ```bash
   SMTP_TIMEOUT_SECONDS=60
   ```
4. **Try different port**:
   ```bash
   # Try port 465 for SSL
   SMTP_PORT=465
   SMTP_USE_SSL=true
   ```

#### Problem: TLS/SSL Issues
```
Error: TLS handshake failed
```

**Solutions:**
1. **Check TLS setting**:
   ```bash
   SMTP_USE_TLS=true  # For port 587
   SMTP_USE_SSL=true  # For port 465
   ```
2. **Update certificates**: Ensure system certificates are up to date
3. **Disable certificate verification** (development only):
   ```bash
   SMTP_VERIFY_CERTIFICATES=false
   ```

### External Service Debugging

#### Enable Debug Logging
```bash
# Enable detailed logging for external services
RUST_LOG=imkitchen_external=debug,reqwest=debug

# Run with debug logging
cargo run --bin imkitchen
```

#### Network Diagnostics
```bash
# Test DNS resolution
nslookup smtp.gmail.com

# Test port connectivity
nc -zv smtp.gmail.com 587

# Test HTTP APIs
curl -v https://api.example.com/health
```

#### Service Health Monitoring
```bash
# Check all service health
cargo run --bin imkitchen health-check

# Check specific service
cargo run --bin imkitchen health-check --service smtp

# Continuous monitoring
watch -n 30 "cargo run --bin imkitchen health-check"
```

### Recovery Procedures

#### SMTP Service Recovery
1. **Check service status**: Verify SMTP provider status page
2. **Rotate credentials**: Generate new API keys/passwords
3. **Failover to backup**: Switch to alternative SMTP provider
4. **Queue management**: Handle queued emails after recovery

#### API Service Recovery
1. **Circuit breaker**: Automatic failover after consecutive failures
2. **Exponential backoff**: Gradual retry with increasing delays
3. **Fallback providers**: Switch to alternative service providers
4. **Data consistency**: Ensure no data loss during outages

### Support Resources

- **Email Issues**: Check provider documentation and status pages
- **API Integration**: Review API provider developer documentation
- **Security**: Follow OWASP guidelines for API security
- **Performance**: Monitor service response times and error rates

For additional support, consult the main [troubleshooting guide](../troubleshooting.md).