# Docker Setup for Local Development

This document describes the Docker Compose setup for local development of imkitchen.

## Services

### MailDev - Email Testing

MailDev is an SMTP server with a web interface for testing email functionality during development.

- **Web UI**: http://localhost:1080
- **SMTP Server**: localhost:1025
- **Image**: maildev/maildev:latest

All emails sent by the application are captured by MailDev and can be viewed in the web interface. No emails are actually sent to external addresses.

## Quick Start

### 1. Start MailDev

```bash
docker-compose up -d maildev
```

### 2. Verify Configuration

The default configuration in `config/default.toml` is already set up for MailDev:

```toml
[email]
smtp_host = "localhost"
smtp_port = 1025
from_email = "noreply@imkitchen.local"
from_name = "imkitchen"
base_url = "http://localhost:3000"
```

No additional configuration needed!

### 3. Run Database Migrations

```bash
cargo run -- migrate
```

### 4. Start the Application

```bash
cargo run -- serve
```

### 5. Access MailDev Web UI

Open http://localhost:1080 in your browser to view captured emails.

## Testing Password Reset Flow

1. Start MailDev: `docker-compose up -d maildev`
2. Start the app: `cargo run -- serve`
3. Navigate to http://localhost:3000/login
4. Click "Forgot Password?"
5. Enter an email address (any registered user)
6. Check MailDev at http://localhost:1080 to see the password reset email
7. Click the reset link in the email to complete the flow

## Stopping Services

Stop all services:

```bash
docker-compose down
```

Stop and remove volumes:

```bash
docker-compose down -v
```

## Configuration

### Default Configuration (`config/default.toml`)

The application uses TOML configuration files. The `config/default.toml` file contains sensible defaults for local development.

### Override Configuration

You can override any configuration value using environment variables with the pattern:

```
IMKITCHEN__SECTION__KEY
```

Examples:

```bash
# Override SMTP host
export IMKITCHEN__EMAIL__SMTP_HOST=smtp.example.com

# Override SMTP port
export IMKITCHEN__EMAIL__SMTP_PORT=587

# Override JWT secret (IMPORTANT for production!)
export IMKITCHEN__JWT__SECRET=your-super-secret-jwt-key-32-chars

# Override base URL
export IMKITCHEN__EMAIL__BASE_URL=https://imkitchen.app
```

### Legacy Environment Variables

For backward compatibility, these environment variables are also supported:

- `DATABASE_URL` - Database connection string
- `JWT_SECRET` - JWT signing secret

### Production Configuration

For production, you can:

1. **Create a custom config file:**
   ```bash
   cargo run -- --config config/production.toml serve
   ```

2. **Use environment variables:**
   ```bash
   export IMKITCHEN__EMAIL__SMTP_HOST=smtp.sendgrid.net
   export IMKITCHEN__EMAIL__SMTP_PORT=587
   export IMKITCHEN__EMAIL__SMTP_USERNAME=apikey
   export IMKITCHEN__EMAIL__SMTP_PASSWORD=SG.your_api_key_here
   export IMKITCHEN__EMAIL__FROM_EMAIL=noreply@imkitchen.app
   export IMKITCHEN__EMAIL__BASE_URL=https://imkitchen.app
   export IMKITCHEN__JWT__SECRET=$(openssl rand -base64 32)

   cargo run --release -- serve
   ```

## Environment Variable Reference

### Email Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `IMKITCHEN__EMAIL__SMTP_HOST` | localhost | SMTP server hostname |
| `IMKITCHEN__EMAIL__SMTP_PORT` | 1025 | SMTP server port |
| `IMKITCHEN__EMAIL__SMTP_USERNAME` | (empty) | SMTP username |
| `IMKITCHEN__EMAIL__SMTP_PASSWORD` | (empty) | SMTP password |
| `IMKITCHEN__EMAIL__FROM_EMAIL` | noreply@imkitchen.local | From email address |
| `IMKITCHEN__EMAIL__FROM_NAME` | imkitchen | From display name |
| `IMKITCHEN__EMAIL__BASE_URL` | http://localhost:3000 | Base URL for reset links |

### Database Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | sqlite:imkitchen.db | Database connection string |
| `IMKITCHEN__DATABASE__MAX_CONNECTIONS` | 5 | Max database connections |

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `IMKITCHEN__SERVER__HOST` | 127.0.0.1 | Server bind address |
| `IMKITCHEN__SERVER__PORT` | 3000 | Server port |

### JWT Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` or `IMKITCHEN__JWT__SECRET` | development_secret... | JWT signing secret (min 32 chars) |

## Future Services

The docker-compose.yml includes commented-out configurations for:

- **PostgreSQL**: Production database (currently using SQLite)
- **OpenTelemetry Collector**: Distributed tracing and observability

Uncomment these sections when ready to use them.

## Troubleshooting

### MailDev not receiving emails

1. Check MailDev is running: `docker-compose ps`
2. Check logs: `docker-compose logs maildev`
3. Verify config in `config/default.toml` matches MailDev ports
4. Ensure no firewall blocking port 1025

### Port conflicts

If port 1080 or 1025 is already in use, edit `docker-compose.yml`:

```yaml
ports:
  - "8080:1080"  # Change left side to different port
  - "2025:1025"  # Change left side to different port
```

Then override the SMTP port:

```bash
export IMKITCHEN__EMAIL__SMTP_PORT=2025
cargo run -- serve
```

Or edit `config/default.toml` directly.

## Example: Using SendGrid in Production

Create `config/production.toml`:

```toml
[email]
smtp_host = "smtp.sendgrid.net"
smtp_port = 587
smtp_username = "apikey"
smtp_password = ""  # Set via IMKITCHEN__EMAIL__SMTP_PASSWORD
from_email = "noreply@imkitchen.app"
from_name = "imkitchen"
base_url = "https://imkitchen.app"

[jwt]
secret = ""  # Set via IMKITCHEN__JWT__SECRET
```

Then run:

```bash
export IMKITCHEN__EMAIL__SMTP_PASSWORD=SG.your_api_key
export IMKITCHEN__JWT__SECRET=$(openssl rand -base64 32)
cargo run --release -- --config config/production.toml serve
```
