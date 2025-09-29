# SMTP Setup Guide for IMKitchen

This guide provides step-by-step instructions for configuring SMTP email delivery in the IMKitchen application.

## Overview

The IMKitchen notification system supports sending emails for:
- User registration confirmations
- Password reset requests  
- System notifications and alerts

The system uses the Lettre SMTP library with support for common email providers and secure authentication.

## Configuration Requirements

### Environment Variables

Create a `.env` file in your project root with the following SMTP configuration:

```bash
# SMTP Server Configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM_EMAIL=noreply@yourdomain.com
SMTP_FROM_NAME=IMKitchen
SMTP_SECURITY=starttls
SMTP_TIMEOUT_SECONDS=30
```

### Configuration Options

| Variable | Description | Example | Required |
|----------|-------------|---------|----------|
| `SMTP_HOST` | SMTP server hostname | `smtp.gmail.com` | Yes |
| `SMTP_PORT` | SMTP server port | `587`, `465`, `25` | Yes |
| `SMTP_USERNAME` | SMTP authentication username | `user@gmail.com` | Yes |
| `SMTP_PASSWORD` | SMTP authentication password | `app-password` | Yes |
| `SMTP_FROM_EMAIL` | Sender email address | `noreply@myapp.com` | Yes |
| `SMTP_FROM_NAME` | Sender display name | `My App` | Yes |
| `SMTP_SECURITY` | Security method | `starttls`, `ssl`, `none` | Yes |
| `SMTP_TIMEOUT_SECONDS` | Connection timeout | `30` | Yes |

## Provider-Specific Setup

### Gmail

1. **Enable 2-Factor Authentication** on your Google account
2. **Generate an App Password**:
   - Go to Google Account settings
   - Security → 2-Step Verification → App passwords
   - Generate a new app password for "Mail"
3. **Configuration**:
   ```bash
   SMTP_HOST=smtp.gmail.com
   SMTP_PORT=587
   SMTP_USERNAME=your-email@gmail.com
   SMTP_PASSWORD=your-16-char-app-password
   SMTP_SECURITY=starttls
   ```

### SendGrid

1. **Create a SendGrid account** and obtain an API key
2. **Configuration**:
   ```bash
   SMTP_HOST=smtp.sendgrid.net
   SMTP_PORT=587
   SMTP_USERNAME=apikey
   SMTP_PASSWORD=SG.your-api-key-here
   SMTP_SECURITY=starttls
   ```

### Mailgun

1. **Create a Mailgun account** and configure your domain
2. **Get SMTP credentials** from the Mailgun dashboard
3. **Configuration**:
   ```bash
   SMTP_HOST=smtp.mailgun.org
   SMTP_PORT=587
   SMTP_USERNAME=postmaster@mg.yourdomain.com
   SMTP_PASSWORD=your-mailgun-password
   SMTP_SECURITY=starttls
   ```

### Custom SMTP Server

For custom SMTP servers, use the appropriate host, port, and credentials:

```bash
SMTP_HOST=mail.yourdomain.com
SMTP_PORT=587
SMTP_USERNAME=noreply@yourdomain.com
SMTP_PASSWORD=your-password
SMTP_SECURITY=starttls
```

## Security Configuration

### Encryption Methods

- **StartTLS** (Recommended): Use `SMTP_SECURITY=starttls` with port 587
- **SSL/TLS**: Use `SMTP_SECURITY=ssl` with port 465
- **None**: Use `SMTP_SECURITY=none` with port 25 (not recommended for production)

### Security Best Practices

1. **Use App Passwords**: Never use your main account password
2. **Secure Storage**: Store credentials in environment variables, not in code
3. **Regular Rotation**: Rotate passwords and API keys regularly
4. **Monitoring**: Monitor email delivery logs for suspicious activity

## Development Setup

For development and testing, you can use a local SMTP server or the built-in development fallback:

### Local SMTP Server (MailCatcher)

1. **Install MailCatcher**:
   ```bash
   gem install mailcatcher
   mailcatcher
   ```

2. **Configuration**:
   ```bash
   SMTP_HOST=localhost
   SMTP_PORT=1025
   SMTP_USERNAME=test@example.com
   SMTP_PASSWORD=password
   SMTP_FROM_EMAIL=noreply@imkitchen.local
   SMTP_FROM_NAME=IMKitchen Dev
   SMTP_SECURITY=none
   SMTP_TIMEOUT_SECONDS=10
   ```

3. **View Emails**: Open http://localhost:1080 to view captured emails

### Development Fallback

If no SMTP configuration is provided, the system uses development fallback settings:

```rust
SmtpConfig {
    host: "localhost".to_string(),
    port: 1025,
    username: "test@example.com".to_string(),
    password: "password".to_string(),
    from_email: "noreply@imkitchen.local".to_string(),
    from_name: "IMKitchen Development".to_string(),
    security: SmtpSecurity::None,
    timeout_seconds: 10,
}
```

## Testing Configuration

### Verify SMTP Settings

Run the SMTP integration tests to verify your configuration:

```bash
cd crates/imkitchen-notification
cargo test smtp_integration_tests -- --nocapture
```

### Send Test Email

Use the application's health check endpoint to test email delivery:

```bash
curl -X POST http://localhost:3000/health/email \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com"}'
```

## Troubleshooting

### Common Issues

1. **Authentication Failed**
   - Verify username and password are correct
   - For Gmail, ensure you're using an app password, not your main password
   - Check if 2FA is enabled and properly configured

2. **Connection Timeout**
   - Verify SMTP host and port are correct
   - Check firewall settings
   - Increase `SMTP_TIMEOUT_SECONDS` if needed

3. **TLS/SSL Errors**
   - Verify the security setting matches the port
   - Port 587 typically uses StartTLS
   - Port 465 typically uses SSL
   - Port 25 typically uses no encryption

4. **Rate Limiting**
   - Check provider's sending limits
   - Implement delays between sends if necessary
   - Consider upgrading to a higher tier plan

### Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `SmtpConnectionFailed` | Cannot connect to SMTP server | Check host, port, and network connectivity |
| `AuthenticationFailed` | Invalid credentials | Verify username and password |
| `RateLimitExceeded` | Too many emails sent | Wait or upgrade plan |
| `DeliveryFailed` | Email rejected by server | Check recipient address and content |

### Debugging

Enable debug logging to see detailed SMTP communication:

```bash
RUST_LOG=imkitchen_notification=debug cargo run
```

## Monitoring and Maintenance

### Health Checks

The application provides SMTP health check endpoints:

```bash
# Check SMTP connection health
curl http://localhost:3000/health/smtp

# Get email delivery statistics
curl http://localhost:3000/health/email-stats
```

### Log Monitoring

Monitor application logs for email delivery issues:

```bash
tail -f logs/imkitchen.log | grep -i smtp
```

### Performance Metrics

Track these metrics for email delivery:
- Delivery success rate
- Average delivery time  
- Connection pool usage
- Rate limit hits

## Production Deployment

### Environment Configuration

1. **Set environment variables** in your deployment system
2. **Use secrets management** for sensitive credentials
3. **Configure monitoring** for email delivery failures
4. **Set up alerting** for SMTP connection issues

### Scaling Considerations

- **Connection Pooling**: Adjust pool size based on email volume
- **Rate Limiting**: Configure limits to respect provider restrictions
- **Retry Logic**: Configure appropriate retry attempts and delays
- **Failover**: Consider backup SMTP providers for high availability

## Support

For additional help:

1. Check the [troubleshooting section](#troubleshooting) above
2. Review application logs for specific error messages
3. Test with the [development setup](#development-setup) first
4. Consult your SMTP provider's documentation

## Security Notice

⚠️ **Important**: Never commit SMTP credentials to version control. Always use environment variables or secure secret management systems for production deployments.