# ImKitchen Configuration Guide

This document provides comprehensive information about configuring ImKitchen for different environments.

## Environment Variables

### Required Variables

These environment variables must be set for the application to run:

#### `DATABASE_URL`
**Required**: Yes  
**Example**: `postgresql://postgres:password@localhost:5432/imkitchen_dev`  
**Description**: PostgreSQL connection string including credentials, host, port, and database name.

#### `JWT_SECRET`
**Required**: Yes  
**Example**: `your_jwt_secret_key_change_in_production_min_32_chars`  
**Description**: Secret key for JWT token signing. Must be at least 32 characters long for security.

### Server Configuration

#### `SERVER_HOST`
**Required**: No  
**Default**: `0.0.0.0`  
**Example**: `127.0.0.1`  
**Description**: Host address for the HTTP server to bind to.

#### `SERVER_PORT`
**Required**: No  
**Default**: `3000`  
**Example**: `8080`  
**Description**: Port number for the HTTP server.

### Database Configuration

#### `DATABASE_MAX_CONNECTIONS`
**Required**: No  
**Default**: `20`  
**Example**: `50`  
**Description**: Maximum number of concurrent database connections in the pool.

#### `DATABASE_MIN_CONNECTIONS`
**Required**: No  
**Default**: `5`  
**Example**: `10`  
**Description**: Minimum number of database connections to maintain in the pool.

### Cache Configuration

#### `REDIS_URL`
**Required**: No  
**Default**: `redis://localhost:6379`  
**Example**: `redis://redis:6379/1`  
**Description**: Redis connection string for session storage and caching.

### Application Configuration

#### `ENVIRONMENT`
**Required**: No  
**Default**: `development`  
**Options**: `development`, `staging`, `production`  
**Description**: Application environment affecting logging, error handling, and feature flags.

#### `JWT_EXPIRES_IN`
**Required**: No  
**Default**: `3600` (1 hour)  
**Example**: `86400` (24 hours)  
**Description**: JWT token expiration time in seconds.

#### `UPLOAD_PATH`
**Required**: No  
**Default**: `./uploads`  
**Example**: `/var/lib/imkitchen/uploads`  
**Description**: Directory path for storing uploaded files (recipe images, etc.).

#### `MAX_FILE_SIZE`
**Required**: No  
**Default**: `10485760` (10MB)  
**Example**: `52428800` (50MB)  
**Description**: Maximum file upload size in bytes.

### Logging Configuration

#### `RUST_LOG`
**Required**: No  
**Default**: `info`  
**Example**: `debug,tower_http=debug,sqlx=info`  
**Description**: Rust logging configuration using env_filter syntax.

#### `LOG_FILE`
**Required**: No  
**Default**: None (console only)  
**Example**: `/var/log/imkitchen/app.log`  
**Description**: Optional file path for application logs.

## Environment-Specific Configuration

### Development

Recommended `.env` file for development:

```env
# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/imkitchen_dev
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=2

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=development_jwt_secret_key_change_in_production_please_32_chars_minimum
JWT_EXPIRES_IN=86400

# Application
ENVIRONMENT=development
UPLOAD_PATH=./uploads
MAX_FILE_SIZE=10485760

# Logging
RUST_LOG=debug,tower_http=debug,sqlx=info
```

### Staging

Recommended configuration for staging environment:

```env
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database
DATABASE_URL=postgresql://imkitchen_user:secure_password@db-staging:5432/imkitchen_staging
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# Redis
REDIS_URL=redis://redis-staging:6379

# JWT
JWT_SECRET=staging_jwt_secret_key_32_characters_minimum_secure
JWT_EXPIRES_IN=3600

# Application
ENVIRONMENT=staging
UPLOAD_PATH=/app/uploads
MAX_FILE_SIZE=20971520

# Logging
RUST_LOG=info
LOG_FILE=/var/log/imkitchen/staging.log
```

### Production

Recommended configuration for production environment:

```env
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database
DATABASE_URL=postgresql://imkitchen_prod:very_secure_password@db-prod:5432/imkitchen_production
DATABASE_MAX_CONNECTIONS=50
DATABASE_MIN_CONNECTIONS=10

# Redis
REDIS_URL=redis://redis-prod:6379

# JWT
JWT_SECRET=production_jwt_secret_key_64_characters_minimum_very_secure_string
JWT_EXPIRES_IN=3600

# Application
ENVIRONMENT=production
UPLOAD_PATH=/var/lib/imkitchen/uploads
MAX_FILE_SIZE=52428800

# Logging
RUST_LOG=warn,imkitchen=info
LOG_FILE=/var/log/imkitchen/production.log
```

## Security Considerations

### JWT Secret

- **Development**: Can use a simple string but should still be 32+ characters
- **Staging/Production**: Use a cryptographically secure random string
- **Generation**: Use `openssl rand -base64 32` or similar tools
- **Rotation**: Plan for secret rotation in production environments

### Database Credentials

- Use strong, unique passwords
- Consider using connection pooling and SSL
- Implement proper backup and recovery procedures
- Use database user with minimal required permissions

### File Uploads

- Validate file types and sizes
- Store uploads outside the web root
- Consider using cloud storage for production
- Implement virus scanning for uploaded files

## Configuration Validation

The application performs configuration validation on startup:

1. **Required variables**: Ensures all required environment variables are set
2. **JWT secret length**: Validates JWT secret is at least 32 characters
3. **Database connections**: Ensures min_connections ≤ max_connections
4. **Port validity**: Validates server port is a valid number

If validation fails, the application will exit with a descriptive error message.

## Docker Configuration

### Development

Use `docker-compose.dev.yml` with development-friendly settings:
- Volume mounts for hot-reload
- Relaxed security settings
- Debug logging enabled

### Production

Use `docker-compose.prod.yml` with production settings:
- Optimized build process
- Security hardened containers
- Resource limits configured
- Health checks enabled

## Environment File Security

**Important**: Never commit actual environment files to version control.

- Use `.env.example` as a template
- Add `.env` to `.gitignore`
- Use secure secret management in production
- Consider using tools like Docker secrets or Kubernetes secrets

## Troubleshooting

### Common Configuration Issues

1. **Database connection failed**
   - Verify DATABASE_URL format
   - Check database server is running
   - Verify credentials and permissions

2. **JWT secret too short**
   - Ensure JWT_SECRET is at least 32 characters
   - Generate secure secret for production

3. **Redis connection failed**
   - Verify Redis server is running
   - Check REDIS_URL format
   - Test Redis connection manually

4. **Port already in use**
   - Change SERVER_PORT to available port
   - Check for other services using the port

### Validation Errors

The application provides detailed error messages for configuration issues. Check the startup logs for specific validation failures and their solutions.