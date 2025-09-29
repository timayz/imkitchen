# SMTP Troubleshooting Guide

This guide helps diagnose and resolve common SMTP configuration and delivery issues in IMKitchen.

## Quick Diagnostic Steps

### 1. Check Configuration

Verify your SMTP configuration is loaded correctly:

```bash
# Run configuration validation test
cd crates/imkitchen-notification
cargo test test_smtp_config_validation -- --nocapture
```

### 2. Test Connection

Test basic SMTP connectivity:

```bash
# Run connection tests
cargo test test_smtp_connection -- --nocapture
```

### 3. Check Logs

Enable debug logging and check for errors:

```bash
RUST_LOG=imkitchen_notification=debug cargo run
```

## Common Error Messages

### Authentication Errors

#### `SmtpConnectionFailed: Authentication failed`

**Cause**: Invalid username or password

**Solutions**:
1. **Gmail Users**: Ensure you're using an App Password, not your regular password
   - Go to Google Account → Security → 2-Step Verification → App passwords
   - Generate a new 16-character app password
   
2. **Other Providers**: Verify credentials in provider dashboard
   - Check username format (email vs username)
   - Verify password is correct
   - Check for special characters that might need escaping

3. **Test with telnet**:
   ```bash
   telnet smtp.gmail.com 587
   EHLO localhost
   STARTTLS
   # Verify connection works manually
   ```

#### `SmtpConnectionFailed: TLS handshake failed`

**Cause**: SSL/TLS configuration mismatch

**Solutions**:
1. **Check Port and Security Settings**:
   - Port 587: Use `SMTP_SECURITY=starttls`
   - Port 465: Use `SMTP_SECURITY=ssl`
   - Port 25: Use `SMTP_SECURITY=none`

2. **Update Configuration**:
   ```bash
   # For Gmail
   SMTP_PORT=587
   SMTP_SECURITY=starttls
   
   # For providers requiring SSL
   SMTP_PORT=465
   SMTP_SECURITY=ssl
   ```

### Connection Errors

#### `SmtpConnectionFailed: Connection timeout`

**Cause**: Network connectivity issues

**Solutions**:
1. **Check Network Connectivity**:
   ```bash
   # Test if SMTP server is reachable
   telnet smtp.gmail.com 587
   
   # Test DNS resolution
   nslookup smtp.gmail.com
   ```

2. **Firewall Issues**:
   - Check corporate firewall settings
   - Verify outbound ports 25, 587, 465 are allowed
   - Try different ports if blocked

3. **Increase Timeout**:
   ```bash
   SMTP_TIMEOUT_SECONDS=60
   ```

#### `SmtpConnectionFailed: Connection refused`

**Cause**: SMTP server not accepting connections

**Solutions**:
1. **Verify Server Settings**:
   - Double-check SMTP host and port
   - Ensure server is operational
   - Check provider status page

2. **Try Alternative Servers**:
   ```bash
   # Gmail alternatives
   SMTP_HOST=smtp.gmail.com
   # or
   SMTP_HOST=aspmx.l.google.com
   ```

### Rate Limiting Errors

#### `RateLimitExceeded`

**Cause**: Sending too many emails too quickly

**Solutions**:
1. **Check Provider Limits**:
   - Gmail: 500 emails/day (free), 2000/day (workspace)
   - SendGrid: Varies by plan
   - Mailgun: Varies by plan

2. **Implement Delays**:
   ```rust
   // Add delay between sends in your application
   tokio::time::sleep(Duration::from_millis(100)).await;
   ```

3. **Upgrade Provider Plan**:
   - Consider paid plans for higher limits
   - Implement queue system for high volume

### Delivery Errors

#### `DeliveryFailed: Message rejected`

**Cause**: Content or recipient issues

**Solutions**:
1. **Check Email Content**:
   - Verify recipient email format
   - Check for spam-like content
   - Ensure sender domain is not blacklisted

2. **Validate Email Templates**:
   ```bash
   cargo test test_email_template_validation -- --nocapture
   ```

3. **Check SPF/DKIM Records**:
   - Configure SPF records for your domain
   - Set up DKIM signing if required

## Provider-Specific Issues

### Gmail Issues

#### "Less secure app access" Error
**Solution**: Use App Passwords instead of regular password

#### 2FA Required Error
**Solution**: Enable 2FA and generate app password

#### Daily Limit Exceeded
**Solution**: 
- Wait 24 hours for reset
- Upgrade to Google Workspace
- Use alternative provider for high volume

### SendGrid Issues

#### API Key Format Error
```bash
# Correct format
SMTP_USERNAME=apikey
SMTP_PASSWORD=SG.your-actual-api-key-here
```

#### Domain Verification Required
**Solution**: Complete domain verification in SendGrid dashboard

### Mailgun Issues

#### Sandbox Domain Limitations
**Solution**: Add authorized recipients or upgrade to production domain

#### EU vs US Servers
**Solution**: Use correct SMTP host:
- US: `smtp.mailgun.org`
- EU: `smtp.eu.mailgun.org`

## Environment-Specific Issues

### Development Environment

#### MailCatcher Not Receiving Emails

**Check MailCatcher Status**:
```bash
# Verify MailCatcher is running
curl http://localhost:1080

# Restart MailCatcher
mailcatcher --ip 0.0.0.0
```

**Configuration**:
```bash
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_SECURITY=none
```

#### Docker Development Issues

**Network Connectivity**:
```bash
# Use host networking for MailCatcher
docker run --net=host mailcatcher

# Or use container name
SMTP_HOST=mailcatcher
```

### Production Environment

#### Environment Variables Not Loading

**Check Environment Setup**:
```bash
# Verify environment variables
env | grep SMTP

# Check .env file location
ls -la .env
```

#### Container Deployment Issues

**Docker Configuration**:
```dockerfile
# Ensure environment variables are passed
ENV SMTP_HOST=${SMTP_HOST}
ENV SMTP_PORT=${SMTP_PORT}
# ... other variables
```

## Debugging Tools

### Enable Debug Logging

```bash
# Full debug output
RUST_LOG=debug cargo run

# SMTP-specific debugging
RUST_LOG=imkitchen_notification=debug cargo run

# Lettre library debugging
RUST_LOG=lettre=debug cargo run
```

### Test Email Sending

Create a test script to isolate issues:

```rust
use imkitchen_notification::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SmtpConfig::from_env()?;
    let connection_manager = SmtpConnectionManager::new(config, 1).await?;
    let template_renderer = EmailTemplateRenderer::new();
    let service = EmailDeliveryService::new(connection_manager, template_renderer)?;
    
    let data = RegistrationEmailData {
        user_name: "Test User".to_string(),
        verification_url: "https://example.com/verify".to_string(),
        app_name: "Test App".to_string(),
    };
    
    match service.send_registration_email("test@example.com", &data).await {
        Ok(status) => println!("✓ Email sent: {:?}", status),
        Err(e) => println!("✗ Email failed: {}", e),
    }
    
    Ok(())
}
```

### Network Diagnostics

```bash
# Test SMTP server connectivity
telnet smtp.gmail.com 587

# Check DNS resolution
dig smtp.gmail.com

# Test with openssl for TLS
openssl s_client -connect smtp.gmail.com:587 -starttls smtp

# Test with curl (if supported)
curl -v --url 'smtps://smtp.gmail.com:465' --mail-from 'from@example.com' --mail-rcpt 'to@example.com'
```

## Performance Issues

### Slow Email Delivery

**Diagnostic Steps**:
1. Check connection pool size
2. Monitor timeout settings
3. Verify network latency

**Solutions**:
```bash
# Increase connection pool
SMTP_CONNECTION_POOL_SIZE=5

# Reduce timeout for faster failures
SMTP_TIMEOUT_SECONDS=10

# Enable connection reuse
SMTP_KEEP_ALIVE=true
```

### High Memory Usage

**Check Connection Pool**:
- Reduce pool size if not needed
- Ensure connections are properly released
- Monitor for connection leaks

## Security Issues

### Certificate Verification Errors

**Solutions**:
1. **Update System Certificates**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update && sudo apt-get install ca-certificates
   
   # CentOS/RHEL
   sudo yum update ca-certificates
   ```

2. **Provider Certificate Issues**:
   - Check if provider uses valid certificates
   - Consider certificate pinning for production

### Credential Exposure

**Prevention**:
1. Never log passwords or API keys
2. Use environment variables
3. Implement credential rotation
4. Monitor for credential leaks

## Getting Help

### Collect Diagnostic Information

Before seeking help, collect:

1. **Configuration** (sanitized):
   ```bash
   env | grep SMTP | sed 's/PASSWORD=.*/PASSWORD=***HIDDEN***/'
   ```

2. **Error Messages**:
   ```bash
   RUST_LOG=debug cargo test 2>&1 | grep -i error
   ```

3. **Network Information**:
   ```bash
   # Test basic connectivity
   telnet smtp.gmail.com 587
   ```

4. **Application Version**:
   ```bash
   cargo --version
   rustc --version
   ```

### Support Channels

1. **Check Documentation**:
   - [SMTP Setup Guide](smtp-setup-guide.md)
   - Provider documentation

2. **Test with Minimal Setup**:
   - Use development fallback first
   - Isolate configuration issues

3. **Community Resources**:
   - Lettre documentation
   - Provider support forums
   - Stack Overflow with specific error messages

## Prevention

### Best Practices

1. **Test in Development First**:
   - Use MailCatcher for local testing
   - Validate configuration before production

2. **Monitor Email Delivery**:
   - Set up health checks
   - Track delivery success rates
   - Alert on failures

3. **Regular Maintenance**:
   - Rotate credentials periodically
   - Update SMTP libraries
   - Review provider limits

4. **Documentation**:
   - Document provider-specific settings
   - Keep troubleshooting logs
   - Share solutions with team