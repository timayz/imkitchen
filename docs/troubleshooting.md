# Troubleshooting Guide

Comprehensive troubleshooting guide for common issues encountered during IMKitchen development, deployment, and operation.

## Table of Contents

- [Development Environment Issues](#development-environment-issues)
- [Compilation and Build Errors](#compilation-and-build-errors)
- [Database Connection Problems](#database-connection-problems)
- [SMTP Configuration Issues](#smtp-configuration-issues)
- [Template Rendering Errors](#template-rendering-errors)
- [TwinSpark Interaction Issues](#twinspark-interaction-issues)
- [Authentication and Session Problems](#authentication-and-session-problems)
- [Performance Issues](#performance-issues)
- [Deployment Problems](#deployment-problems)
- [General Debugging Tips](#general-debugging-tips)

## Development Environment Issues

### Rust Installation Problems

#### Issue: `rustc` or `cargo` command not found
```bash
$ cargo --version
bash: cargo: command not found
```

**Solutions:**
1. **Reload shell configuration:**
   ```bash
   # For bash
   source ~/.bashrc
   
   # For zsh (macOS default)
   source ~/.zshrc
   ```

2. **Add Rust to PATH manually:**
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   ```

3. **Reinstall Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

#### Issue: Permission denied during Rust installation
```bash
error: could not create directory '/home/user/.cargo'
```

**Solutions:**
1. **Check home directory permissions:**
   ```bash
   ls -la ~/
   chmod 755 ~
   ```

2. **Install with different permissions:**
   ```bash
   # Install to custom location
   export CARGO_HOME=/tmp/cargo
   export RUSTUP_HOME=/tmp/rustup
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Node.js and npm Issues

#### Issue: `npm install -g` permission denied
```bash
$ npm install -g @tailwindcss/cli
npm ERR! Error: EACCES: permission denied
```

**Solutions:**
1. **Use a Node version manager (recommended):**
   ```bash
   # Install nvm
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   
   # Install and use latest Node.js
   nvm install node
   nvm use node
   ```

2. **Configure npm global directory:**
   ```bash
   mkdir ~/.npm-global
   npm config set prefix '~/.npm-global'
   echo 'export PATH=~/.npm-global/bin:$PATH' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Use npx instead of global install:**
   ```bash
   npx @tailwindcss/cli -i input.css -o output.css
   ```

## Compilation and Build Errors

### Common Rust Compilation Errors

#### Issue: SQLx compile-time verification failed
```bash
error: error occurred while preparing the query:
table `users` doesn't exist in database
```

**Solutions:**
1. **Run database migrations first:**
   ```bash
   cargo sqlx migrate run --database-url sqlite:imkitchen.db
   ```

2. **Generate query metadata for offline compilation:**
   ```bash
   cargo sqlx prepare --database-url sqlite:imkitchen.db
   ```

3. **Use offline mode in CI/CD:**
   ```bash
   export SQLX_OFFLINE=true
   cargo build
   ```

#### Issue: Missing dependency errors
```bash
error[E0432]: unresolved import `imkitchen_shared::types::UserId`
```

**Solutions:**
1. **Check Cargo.toml dependencies:**
   ```toml
   [dependencies]
   imkitchen-shared = { path = "../imkitchen-shared" }
   ```

2. **Verify workspace configuration:**
   ```toml
   # Root Cargo.toml
   [workspace]
   members = [
       "crates/imkitchen-shared",
       "crates/imkitchen-user",
   ]
   ```

3. **Clean and rebuild:**
   ```bash
   cargo clean
   cargo build --workspace
   ```

#### Issue: Askama template compilation errors
```bash
error: failed to derive template: path `recipe_detail.html` not found
```

**Solutions:**
1. **Check template file exists:**
   ```bash
   ls -la crates/imkitchen-web/templates/recipes/recipe_detail.html
   ```

2. **Verify template path in derive macro:**
   ```rust
   #[derive(Template)]
   #[template(path = "recipes/recipe_detail.html")]  // Correct path
   pub struct RecipeDetailTemplate { }
   ```

3. **Check template syntax:**
   ```html
   <!-- Ensure proper Askama syntax -->
   <h1>{{ recipe.title|e }}</h1>
   {% for ingredient in recipe.ingredients %}
   <li>{{ ingredient.name|e }}</li>
   {% endfor %}
   ```

### Linking and Library Errors

#### Issue: OpenSSL linking errors on Linux
```bash
error: failed to run custom build command for `openssl-sys`
```

**Solutions:**
1. **Install development packages:**
   ```bash
   # Ubuntu/Debian
   sudo apt update
   sudo apt install pkg-config libssl-dev build-essential
   
   # CentOS/RHEL/Fedora
   sudo dnf install pkgconfig openssl-devel gcc
   
   # Arch Linux
   sudo pacman -S base-devel openssl pkg-config
   ```

2. **Use system OpenSSL:**
   ```bash
   export OPENSSL_DIR=/usr/lib/ssl
   cargo build
   ```

#### Issue: SQLite linking errors
```bash
error: linking with `cc` failed: SQLite not found
```

**Solutions:**
1. **Install SQLite development libraries:**
   ```bash
   # Ubuntu/Debian
   sudo apt install libsqlite3-dev
   
   # CentOS/RHEL/Fedora
   sudo dnf install sqlite-devel
   
   # macOS
   brew install sqlite
   ```

2. **Use bundled SQLite:**
   ```toml
   [dependencies]
   sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls", "bundled"] }
   ```

## Database Connection Problems

### SQLite Database Issues

#### Issue: Database file locked
```bash
Error: database is locked
```

**Solutions:**
1. **Check for running processes:**
   ```bash
   # Find processes using the database
   lsof imkitchen.db
   
   # Kill hanging processes
   pkill -f imkitchen
   ```

2. **Close SQLite connections:**
   ```bash
   # If using sqlite3 CLI
   sqlite3 imkitchen.db ".quit"
   ```

3. **Check file permissions:**
   ```bash
   ls -la imkitchen.db
   chmod 644 imkitchen.db
   ```

#### Issue: Database corruption detected
```bash
Error: database disk image is malformed
```

**Solutions:**
1. **Check database integrity:**
   ```bash
   sqlite3 imkitchen.db "PRAGMA integrity_check;"
   ```

2. **Recover from backup:**
   ```bash
   # If you have a backup
   cp imkitchen.db.backup imkitchen.db
   ```

3. **Recreate database from migrations:**
   ```bash
   # Remove corrupted database
   rm imkitchen.db
   
   # Run migrations to recreate
   cargo sqlx migrate run --database-url sqlite:imkitchen.db
   ```

### Migration Issues

#### Issue: Migration already applied error
```bash
Error: Migration 20251129100000 has already been applied
```

**Solutions:**
1. **Check migration status:**
   ```bash
   cargo sqlx migrate info --database-url sqlite:imkitchen.db
   ```

2. **Force rerun migration:**
   ```bash
   # Remove from migration table
   sqlite3 imkitchen.db "DELETE FROM _sqlx_migrations WHERE version = 20251129100000;"
   
   # Rerun migrations
   cargo sqlx migrate run --database-url sqlite:imkitchen.db
   ```

3. **Reset migration state:**
   ```bash
   # Drop migration table
   sqlite3 imkitchen.db "DROP TABLE IF EXISTS _sqlx_migrations;"
   
   # Rerun all migrations
   cargo sqlx migrate run --database-url sqlite:imkitchen.db
   ```

## SMTP Configuration Issues

### SMTP Connection Problems

#### Issue: SMTP authentication failed
```
Error: authentication failed
Status: 535 5.7.8 Username and Password not accepted
```

**Solutions:**
1. **Verify credentials:**
   ```bash
   # Test SMTP connection manually
   telnet smtp.gmail.com 587
   # Use correct username and app password
   ```

2. **Use app-specific passwords (Gmail):**
   - Enable 2-Factor Authentication
   - Generate App Password in Google Account settings
   - Use app password instead of regular password

3. **Check SMTP settings:**
   ```bash
   # Verify environment variables
   echo $SMTP_HOST
   echo $SMTP_PORT
   echo $SMTP_USERNAME
   # Don't echo password for security
   ```

#### Issue: SMTP connection timeout
```
Error: connection timed out
```

**Solutions:**
1. **Check network connectivity:**
   ```bash
   # Test connectivity to SMTP server
   telnet smtp.gmail.com 587
   nc -zv smtp.gmail.com 587
   ```

2. **Verify firewall settings:**
   ```bash
   # Check if port 587 is blocked
   sudo ufw status
   sudo iptables -L
   ```

3. **Try different ports:**
   ```bash
   # Try SSL port instead of STARTTLS
   SMTP_PORT=465
   SMTP_SECURITY=ssl
   ```

#### Issue: SMTP certificate verification failed
```
Error: certificate verify failed
```

**Solutions:**
1. **Update certificates:**
   ```bash
   # Ubuntu/Debian
   sudo apt update && sudo apt install ca-certificates
   
   # CentOS/RHEL
   sudo dnf update ca-certificates
   ```

2. **Disable certificate verification (development only):**
   ```rust
   // In SMTP configuration (NOT for production)
   .danger_accept_invalid_certs(true)
   ```

## Template Rendering Errors

### Askama Template Issues

#### Issue: Template not found error
```bash
Error: template `recipe_detail.html` not found
```

**Solutions:**
1. **Check file path:**
   ```bash
   find . -name "recipe_detail.html"
   ls -la crates/imkitchen-web/templates/recipes/
   ```

2. **Verify template configuration:**
   ```rust
   #[derive(Template)]
   #[template(path = "recipes/recipe_detail.html")]  // Path relative to templates/
   pub struct RecipeDetailTemplate { }
   ```

3. **Check Cargo.toml template configuration:**
   ```toml
   [package.metadata.askama]
   dirs = ["templates"]
   ```

#### Issue: Template variable not found
```bash
Error: variable `recipe.title` not found in scope
```

**Solutions:**
1. **Check template data structure:**
   ```rust
   #[derive(Template)]
   #[template(path = "recipe_detail.html")]
   pub struct RecipeDetailTemplate {
       pub recipe: Recipe,  // Make sure field exists
   }
   ```

2. **Verify field access in template:**
   ```html
   <!-- Correct -->
   <h1>{{ recipe.title|e }}</h1>
   
   <!-- If recipe is Option<Recipe> -->
   {% if let Some(recipe) = recipe %}
   <h1>{{ recipe.title|e }}</h1>
   {% endif %}
   ```

### XSS and Security Issues

#### Issue: Unescaped HTML in templates
```html
<!-- Dangerous - XSS vulnerability -->
<div>{{ user_input }}</div>

<!-- Safe - escaped output -->
<div>{{ user_input|e }}</div>
```

**Solutions:**
1. **Always escape user input:**
   ```html
   <h1>{{ recipe.title|e }}</h1>
   <p>{{ recipe.description|e|nl2br }}</p>
   ```

2. **Use safe filter only for trusted content:**
   ```html
   <!-- Only for sanitized HTML -->
   <div>{{ admin_content|safe }}</div>
   ```

## TwinSpark Interaction Issues

### TwinSpark JavaScript Errors

#### Issue: TwinSpark not loading
```
Uncaught ReferenceError: TwinSpark is not defined
```

**Solutions:**
1. **Check TwinSpark script inclusion:**
   ```html
   <script src="/static/js/twinspark.js"></script>
   ```

2. **Download TwinSpark library:**
   ```bash
   curl -o ./crates/imkitchen-web/static/js/twinspark.js https://unpkg.com/twinspark@1/dist/twinspark.js
   ```

3. **Verify static file serving:**
   ```bash
   curl http://localhost:3000/static/js/twinspark.js
   ```

#### Issue: TwinSpark requests failing
```
POST http://localhost:3000/recipes 422 (Unprocessable Entity)
```

**Solutions:**
1. **Check CSRF token:**
   ```html
   <form ts-req="/recipes" ts-target="#result">
       <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
       <!-- form fields -->
   </form>
   ```

2. **Verify form data format:**
   ```html
   <!-- Correct form encoding -->
   <form enctype="application/x-www-form-urlencoded">
   ```

3. **Check request headers:**
   ```javascript
   // Browser developer tools > Network tab
   // Verify Content-Type: application/x-www-form-urlencoded
   ```

### Form Validation Issues

#### Issue: Real-time validation not working
```html
<input 
    name="email" 
    ts-req="/validate/email" 
    ts-trigger="blur" 
    ts-target="#email-error">
```

**Solutions:**
1. **Check validation endpoint:**
   ```bash
   curl -X POST http://localhost:3000/validate/email -d "email=test@example.com"
   ```

2. **Verify target element exists:**
   ```html
   <div id="email-error"></div>  <!-- Make sure this exists -->
   ```

3. **Check TwinSpark attributes:**
   ```html
   <input 
       name="email"
       ts-req="/validate/email"
       ts-trigger="blur"          <!-- Event to trigger -->
       ts-target="#email-error"   <!-- Where to put response -->
       ts-swap="innerHTML">       <!-- How to replace content -->
   ```

## Authentication and Session Problems

### Session Management Issues

#### Issue: User session not persisting
```
Error: session expired or invalid
```

**Solutions:**
1. **Check session secret configuration:**
   ```bash
   # Make sure SESSION_SECRET is set and long enough
   echo $SESSION_SECRET | wc -c  # Should be 32+ characters
   ```

2. **Verify cookie settings:**
   ```rust
   // Check cookie configuration
   SessionLayer::new(session_store, session_secret.as_bytes())
       .with_cookie_name("session_id")
       .with_cookie_domain(cookie_domain)
       .with_same_site_policy(SameSite::Strict)
       .with_secure(true)  // Only if using HTTPS
   ```

3. **Check browser cookies:**
   ```javascript
   // Browser console
   document.cookie
   // Should show session_id cookie
   ```

#### Issue: CSRF token validation failing
```
Error: CSRF token mismatch
```

**Solutions:**
1. **Ensure CSRF token in forms:**
   ```html
   <form method="POST">
       <input type="hidden" name="csrf_token" value="{{ csrf_token }}">
       <!-- form fields -->
   </form>
   ```

2. **Check token generation:**
   ```rust
   // In template data
   pub struct TemplateData {
       pub csrf_token: String,  // Generated per request
   }
   ```

### Authentication Middleware Issues

#### Issue: Protected routes accessible without login
```bash
curl http://localhost:3000/profile
# Should return 401/redirect, not profile page
```

**Solutions:**
1. **Check middleware order:**
   ```rust
   let app = Router::new()
       .route("/profile", get(profile_handler))
       .layer(AuthRequiredLayer)  // Auth middleware
       .layer(SessionLayer::new(/* ... */));  // Session layer first
   ```

2. **Verify route protection:**
   ```rust
   async fn profile_handler(
       user: UserSession,  // This should require authentication
   ) -> Result<Html<String>, AppError> {
       // Handler implementation
   }
   ```

## Performance Issues

### Slow Application Startup

#### Issue: Application takes long time to start
```bash
$ cargo run -- web start
# Takes 30+ seconds to start
```

**Solutions:**
1. **Check database connection pooling:**
   ```rust
   // Reduce initial connections for development
   SqlitePoolOptions::new()
       .max_connections(2)  // Lower for development
       .min_connections(1)
       .connect_timeout(Duration::from_secs(5))
   ```

2. **Profile startup time:**
   ```bash
   # Add timing logs
   RUST_LOG=debug cargo run -- web start
   ```

3. **Check for blocking operations:**
   ```rust
   // Avoid blocking operations in startup
   tokio::task::spawn_blocking(|| {
       // Move blocking work to thread pool
   }).await?;
   ```

### Slow Database Queries

#### Issue: Database queries taking too long
```
Query took 2000ms to execute
```

**Solutions:**
1. **Add database indexes:**
   ```sql
   CREATE INDEX idx_recipes_user_id ON recipes(user_id);
   CREATE INDEX idx_recipes_cuisine ON recipes(cuisine_type);
   ```

2. **Analyze query performance:**
   ```sql
   EXPLAIN QUERY PLAN 
   SELECT * FROM recipes WHERE user_id = ? AND cuisine_type = ?;
   ```

3. **Use connection pooling:**
   ```rust
   // Configure appropriate pool size
   SqlitePoolOptions::new()
       .max_connections(10)
       .acquire_timeout(Duration::from_secs(3))
   ```

## Deployment Problems

### Docker Build Issues

#### Issue: Docker build failing
```bash
ERROR: failed to build: Could not find Cargo.toml
```

**Solutions:**
1. **Check Dockerfile context:**
   ```dockerfile
   # Make sure COPY paths are correct
   COPY Cargo.toml Cargo.lock ./
   COPY crates/ ./crates/
   COPY src/ ./src/
   ```

2. **Verify build context:**
   ```bash
   # Build from project root
   docker build -t imkitchen .
   ```

3. **Check .dockerignore:**
   ```
   target/
   .git/
   .env
   *.db
   ```

### Container Runtime Issues

#### Issue: Application not accessible in container
```bash
curl http://localhost:3000
# Connection refused
```

**Solutions:**
1. **Bind to all interfaces:**
   ```bash
   # In container, use 0.0.0.0, not 127.0.0.1
   SERVER_HOST=0.0.0.0 cargo run -- web start --port 3000
   ```

2. **Check port mapping:**
   ```bash
   docker run -p 3000:3000 imkitchen
   ```

3. **Verify container networking:**
   ```bash
   docker exec -it container_id netstat -tlnp
   ```

## General Debugging Tips

### Logging and Tracing

1. **Enable detailed logging:**
   ```bash
   RUST_LOG=debug cargo run -- web start
   RUST_LOG=imkitchen=trace,sqlx=debug cargo run
   ```

2. **Add tracing to handlers:**
   ```rust
   #[tracing::instrument(skip(app_state))]
   async fn create_recipe_handler(
       State(app_state): State<AppState>,
       user: UserSession,
       Form(request): Form<CreateRecipeRequest>,
   ) -> Result<Html<String>, AppError> {
       tracing::info!("Creating recipe for user {}", user.user_id);
       // Handler implementation
   }
   ```

### Development Tools

1. **Use cargo watch for auto-rebuild:**
   ```bash
   cargo watch -x "run -- web start --port 3000"
   ```

2. **Check with cargo clippy:**
   ```bash
   cargo clippy --workspace --all-targets
   ```

3. **Format code consistently:**
   ```bash
   cargo fmt --all
   ```

### Browser Developer Tools

1. **Check network requests:**
   - Open F12 Developer Tools
   - Network tab to see failed requests
   - Look for 4XX/5XX status codes

2. **Inspect form submissions:**
   - Check request payload in Network tab
   - Verify Content-Type headers
   - Look for CSRF token inclusion

3. **Debug TwinSpark issues:**
   - Console tab for JavaScript errors
   - Network tab for AJAX requests
   - Elements tab to verify DOM updates

### Environment Verification

1. **Check all environment variables:**
   ```bash
   printenv | grep -E "(DATABASE|SMTP|SESSION)"
   ```

2. **Verify file permissions:**
   ```bash
   ls -la imkitchen.db
   ls -la crates/imkitchen-web/static/
   ```

3. **Test individual components:**
   ```bash
   # Test database connection
   sqlite3 imkitchen.db ".tables"
   
   # Test SMTP (if configured)
   telnet smtp.gmail.com 587
   ```

## Getting Additional Help

If you're still experiencing issues after trying these solutions:

1. **Check the logs:**
   ```bash
   RUST_LOG=debug cargo run -- web start 2>&1 | tee debug.log
   ```

2. **Create a minimal reproduction:**
   - Isolate the issue to smallest possible case
   - Document exact steps to reproduce

3. **Gather system information:**
   ```bash
   rustc --version
   cargo --version
   uname -a
   sqlite3 --version
   ```

4. **Check project issues:**
   - Look at GitHub issues for similar problems
   - Search documentation for relevant sections

For more help:
- [Development Setup Guide](development/setup.md)
- [API Documentation](api/README.md)
- [Database Documentation](database/README.md)
- [Deployment Guide](deployment/README.md)