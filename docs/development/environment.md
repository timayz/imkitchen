# Environment Configuration Guide

Complete reference for configuring IMKitchen environment variables across development, staging, and production environments.

## Table of Contents

- [Environment Files](#environment-files)
- [Required Variables](#required-variables)
- [Optional Variables](#optional-variables)
- [SMTP Configuration](#smtp-configuration)
- [Security Best Practices](#security-best-practices)
- [Environment-Specific Configurations](#environment-specific-configurations)
- [Validation and Testing](#validation-and-testing)

## Environment Files

### Development (.env)
```bash
# Create from template
cp .env.example .env
```

### Production
```bash
# Use environment variables directly
# Never commit .env files to version control
```

### Docker
```yaml
# docker-compose.yml
environment:
  - DATABASE_URL=sqlite:/data/imkitchen.db
  - SESSION_SECRET=${SESSION_SECRET}
```

## Required Variables

These variables must be set for the application to function:

### Application Configuration

```bash
# Database connection string
DATABASE_URL=sqlite:imkitchen.db

# Logging level (trace, debug, info, warn, error)
RUST_LOG=info

# Server binding configuration
SERVER_HOST=0.0.0.0    # 127.0.0.1 for localhost only
SERVER_PORT=3000       # Port number (1024-65535)
```

### Security Configuration

```bash
# Session encryption secret (32+ characters, randomly generated)
SESSION_SECRET=your-very-secure-32-character-secret-key-here

# Password hashing strength (8-15, recommended: 12)
PASSWORD_SALT_ROUNDS=12
```

#### Generating Secure Session Secret

```bash
# Using OpenSSL
openssl rand -hex 32

# Using Python
python3 -c "import secrets; print(secrets.token_hex(32))"

# Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"

# Manual generation (Linux/macOS)
head -c 32 /dev/urandom | base64
```

## Optional Variables

### SMTP Email Configuration

```bash
# SMTP server settings
SMTP_HOST=smtp.gmail.com           # SMTP server hostname
SMTP_PORT=587                      # 587 (STARTTLS), 465 (SSL), 25 (plain)
SMTP_USERNAME=your-email@gmail.com # Authentication username
SMTP_PASSWORD=your-app-password    # Authentication password
SMTP_FROM_EMAIL=noreply@imkitchen.com  # Sender email address
SMTP_FROM_NAME=IMKitchen          # Sender display name

# Security and connection settings
SMTP_SECURITY=starttls            # starttls, ssl, none
SMTP_TIMEOUT=30                   # Connection timeout in seconds
```

### Feature Flags

```bash
# Application features
ENABLE_REGISTRATION=true          # Allow new user registration
ENABLE_EMAIL_VERIFICATION=true    # Require email verification
ENABLE_COMMUNITY_FEATURES=true    # Enable social features
```

### Development Configuration

```bash
# Development-specific settings
RUST_BACKTRACE=1                  # Full stack traces in development
CARGO_WATCH_IGNORE="*.db,*.log"   # Files to ignore in watch mode
DATABASE_MAX_CONNECTIONS=5        # Database connection pool size
```

## SMTP Configuration

### Provider-Specific Settings

#### Gmail Configuration

```bash
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-specific-password
SMTP_SECURITY=starttls
```

**Setup Steps:**
1. Enable 2-Factor Authentication on Google account
2. Generate App Password: Google Account → Security → App passwords
3. Use the generated password (not your regular password)

#### SendGrid Configuration

```bash
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=your-sendgrid-api-key
SMTP_SECURITY=starttls
```

**Setup Steps:**
1. Create SendGrid account
2. Generate API key: Settings → API Keys
3. Use 'apikey' as username and API key as password

#### Mailgun Configuration

```bash
SMTP_HOST=smtp.mailgun.org
SMTP_PORT=587
SMTP_USERNAME=postmaster@your-domain.mailgun.org
SMTP_PASSWORD=your-mailgun-smtp-password
SMTP_SECURITY=starttls
```

#### Custom SMTP Server

```bash
SMTP_HOST=mail.your-domain.com
SMTP_PORT=587                     # or 465 for SSL
SMTP_USERNAME=username@your-domain.com
SMTP_PASSWORD=your-smtp-password
SMTP_SECURITY=starttls           # or ssl
```

### Development SMTP Options

#### Option 1: Disable SMTP (Local Development)
```bash
# Comment out or remove all SMTP_* variables
# Application will log emails to console instead
```

#### Option 2: MailHog (Local SMTP Testing)
```bash
# Install MailHog
go install github.com/mailhog/MailHog@latest

# Run MailHog
MailHog

# Configure application
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_SECURITY=none
# Leave username/password empty
```

#### Option 3: Mailtrap (Development Testing)
```bash
SMTP_HOST=smtp.mailtrap.io
SMTP_PORT=2525
SMTP_USERNAME=your-mailtrap-username
SMTP_PASSWORD=your-mailtrap-password
SMTP_SECURITY=starttls
```

## Security Best Practices

### Environment Variable Security

1. **Never commit secrets to version control**
   ```bash
   # Add to .gitignore
   .env
   .env.local
   .env.production
   ```

2. **Use strong, unique secrets**
   ```bash
   # Generate unique secrets for each environment
   SESSION_SECRET=$(openssl rand -hex 32)
   ```

3. **Rotate secrets regularly**
   ```bash
   # Change secrets periodically
   # Update all instances simultaneously
   ```

4. **Use environment-specific configurations**
   ```bash
   # Development
   RUST_LOG=debug
   
   # Production
   RUST_LOG=warn
   ```

### Production Security

1. **Restrict network access**
   ```bash
   SERVER_HOST=127.0.0.1  # Internal only
   ```

2. **Use secure SMTP connections**
   ```bash
   SMTP_SECURITY=starttls  # Never use 'none' in production
   ```

3. **Enable all security features**
   ```bash
   ENABLE_EMAIL_VERIFICATION=true
   ```

## Environment-Specific Configurations

### Development Environment

```bash
# .env (development)
DATABASE_URL=sqlite:imkitchen.db
RUST_LOG=debug
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
SESSION_SECRET=dev-secret-32-chars-replace-in-prod
PASSWORD_SALT_ROUNDS=10

# Optional SMTP for testing
# SMTP_HOST=localhost
# SMTP_PORT=1025

ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=false
RUST_BACKTRACE=1
```

### Staging Environment

```bash
# Staging environment variables
DATABASE_URL=sqlite:/app/data/imkitchen.db
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
SESSION_SECRET=${STAGING_SESSION_SECRET}
PASSWORD_SALT_ROUNDS=12

# Real SMTP for testing
SMTP_HOST=smtp.mailtrap.io
SMTP_PORT=2525
SMTP_USERNAME=${STAGING_SMTP_USERNAME}
SMTP_PASSWORD=${STAGING_SMTP_PASSWORD}
SMTP_SECURITY=starttls

ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
```

### Production Environment

```bash
# Production environment variables
DATABASE_URL=sqlite:/app/data/imkitchen.db
RUST_LOG=warn
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
SESSION_SECRET=${PRODUCTION_SESSION_SECRET}
PASSWORD_SALT_ROUNDS=12

# Production SMTP
SMTP_HOST=smtp.sendgrid.net
SMTP_PORT=587
SMTP_USERNAME=apikey
SMTP_PASSWORD=${PRODUCTION_SENDGRID_API_KEY}
SMTP_FROM_EMAIL=noreply@imkitchen.com
SMTP_FROM_NAME=IMKitchen
SMTP_SECURITY=starttls
SMTP_TIMEOUT=30

ENABLE_REGISTRATION=true
ENABLE_EMAIL_VERIFICATION=true
ENABLE_COMMUNITY_FEATURES=true
```

## Validation and Testing

### Environment Variable Validation

The application automatically validates environment variables on startup:

```bash
# Check configuration
cargo run -- web start --port 3000

# Look for validation errors in logs:
# ERROR: SESSION_SECRET must be at least 32 characters
# ERROR: Invalid SMTP_PORT: must be between 1 and 65535
```

### Testing SMTP Configuration

```bash
# Test SMTP connectivity
cargo run -- test-smtp

# Send test email
cargo run -- send-test-email user@example.com
```

### Environment Validation Script

Create a validation script for your environment:

```bash
#!/bin/bash
# validate-env.sh

echo "Validating environment configuration..."

# Check required variables
required_vars=("DATABASE_URL" "SESSION_SECRET" "SERVER_PORT")
for var in "${required_vars[@]}"; do
    if [[ -z "${!var}" ]]; then
        echo "ERROR: $var is not set"
        exit 1
    fi
done

# Validate SESSION_SECRET length
if [[ ${#SESSION_SECRET} -lt 32 ]]; then
    echo "ERROR: SESSION_SECRET must be at least 32 characters"
    exit 1
fi

# Validate PORT range
if [[ $SERVER_PORT -lt 1 || $SERVER_PORT -gt 65535 ]]; then
    echo "ERROR: SERVER_PORT must be between 1 and 65535"
    exit 1
fi

echo "Environment validation passed!"
```

### Docker Environment Validation

```dockerfile
# Dockerfile environment validation
ENV REQUIRED_VARS="DATABASE_URL SESSION_SECRET SERVER_PORT"
RUN for var in $REQUIRED_VARS; do \
      if [ -z "$(eval echo \$$var)" ]; then \
        echo "ERROR: $var is not set"; \
        exit 1; \
      fi; \
    done
```

## Troubleshooting

### Common Environment Issues

#### 1. Session Secret Too Short
```
Error: SESSION_SECRET must be at least 32 characters
```
**Solution:** Generate a longer secret using `openssl rand -hex 32`

#### 2. SMTP Authentication Failed
```
Error: SMTP authentication failed
```
**Solutions:**
- Verify username/password combination
- Use app-specific passwords for Gmail
- Check SMTP server settings

#### 3. Database Connection Failed
```
Error: Failed to connect to database
```
**Solutions:**
- Verify DATABASE_URL format
- Check file permissions for SQLite
- Ensure database directory exists

#### 4. Port Already in Use
```
Error: Address already in use
```
**Solutions:**
- Change SERVER_PORT to different value
- Stop other services using the port
- Use `lsof -i :3000` to find conflicting processes

### Environment Debugging

```bash
# Check environment variables
printenv | grep -E "(DATABASE|SMTP|SESSION)"

# Validate .env file loading
cargo run -- config-dump

# Test individual components
cargo run -- test-database
cargo run -- test-smtp
```

## Migration Guide

### Updating Environment Configuration

When updating environment variables:

1. **Update .env.example**
   ```bash
   # Add new variable to template
   NEW_FEATURE_ENABLED=false
   ```

2. **Update documentation**
   ```bash
   # Document the new variable
   # Update this file and setup.md
   ```

3. **Add validation**
   ```rust
   // Add validation in config.rs
   ```

4. **Communicate changes**
   ```bash
   # Notify team of required environment updates
   # Update deployment scripts
   ```

For more configuration help, see our [Setup Guide](setup.md) and [Troubleshooting Guide](../troubleshooting.md).